use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::convert::TryInto;
use log::{debug, error};
use mibpf_common::{
    BinaryFileLayout, HelperAccessListSource, HelperAccessVerification, SuitPullRequest,
    VMConfiguration,
};
use mibpf_elf_utils::extract_allowed_helpers;

use coap_message::{MutableWritableMessage, ReadableMessage};

use crate::{
    infra::suit_storage::{self, SUIT_STORAGE_SLOT_SIZE},
    vm::{middleware::helpers::HelperAccessList, rbpf_vm},
};

use super::util::preprocess_request_raw;

pub struct SuitPullHandler {
    /// Status of the last processed request, if successful it will contain
    /// the name of the SUIT manifest file from where the image was pulled.
    last_request_status: Result<String, String>,
}

impl SuitPullHandler {
    pub fn new() -> Self {
        Self {
            last_request_status: Err("No requests processed yet".to_string()),
        }
    }
}

impl coap_handler::Handler for SuitPullHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        let preprocessing_result: Result<String, u8> = preprocess_request_raw(request);

        let Ok(request_str) = preprocessing_result else {
            return preprocessing_result.err().unwrap();
        };

        let parsed_request = SuitPullRequest::decode(request_str);
        let Ok(request) = parsed_request else {
            return coap_numbers::code::BAD_REQUEST;
        };

        let fetch_result = suit_storage::suit_fetch(
            request.ip.as_str(),
            request.riot_netif.as_str(),
            request.manifest.as_str(),
        );

        if let Ok(()) = fetch_result {
            debug!("SUIT fetch successful.");
        } else {
            debug!("SUIT fetch failed.");
            self.last_request_status = Err("SUIT worker failed to pull new firmware".to_string());
            return coap_numbers::code::BAD_REQUEST;
        }

        let config = VMConfiguration::decode(request.config);

        if config.helper_access_verification == HelperAccessVerification::LoadTime {
            let mut program_buffer = [0; SUIT_STORAGE_SLOT_SIZE];
            let program = suit_storage::load_program(&mut program_buffer, config.suit_slot);

            let helper_idxs: Vec<u32> = match config.helper_access_list_source {
                HelperAccessListSource::ExecuteRequest => HelperAccessList::from(request.helpers)
                    .0
                    .into_iter()
                    .map(|f| f.id as u32)
                    .collect(),
                HelperAccessListSource::BinaryMetadata => {
                    if config.binary_layout == BinaryFileLayout::ExtendedHeader {
                        extract_allowed_helpers(&program)
                            .into_iter()
                            .map(|id| id as u32)
                            .collect()
                    } else {
                        let error_msg = "Tried to extract allowed helper functions from an incompatible binary file.";
                        error!("{}", error_msg);
                        self.last_request_status = Err(error_msg.to_string());
                        suit_storage::suit_erase(config.suit_slot);
                        return coap_numbers::code::BAD_REQUEST;
                    }
                }
            };

            let interpreter = rbpf_vm::map_interpreter(config.binary_layout);

            if let Err(e) = rbpf::check_helpers(program, &helper_idxs, interpreter)
                .map_err(|e| format!("Error when checking helper function access: {:?}", e))
            {
                error!("{}", e);
                self.last_request_status = Err(e);
                suit_storage::suit_erase(config.suit_slot);
                return coap_numbers::code::BAD_REQUEST;
            }
        }

        self.last_request_status = Ok(String::from(request.manifest));
        coap_numbers::code::CHANGED
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(
        &mut self,
        response: &mut impl MutableWritableMessage,
        request: Self::RequestData,
    ) {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());

        let res = match &self.last_request_status {
            Ok(suit_manifest) => {
                format!(
                    "SUIT pull request processed successfully for manifest: {}",
                    suit_manifest
                )
            }
            Err(e) => {
                format!("SUIT pull request failed: {}", e)
            }
        };
        response.set_payload(res.as_bytes());
    }
}
