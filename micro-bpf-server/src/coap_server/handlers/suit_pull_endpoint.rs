use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::convert::TryInto;
use log::{debug, error};
use micro_bpf_common::{
    BinaryFileLayout, HelperAccessListSource, HelperAccessVerification, SuitPullRequest,
    VMConfiguration,
};
use micro_bpf_elf_utils::extract_allowed_helpers;

use coap_message::{MinimalWritableMessage, MutableWritableMessage, ReadableMessage};

use crate::{
    infra::suit_storage::{self, SUIT_STORAGE_SLOT_SIZE},
    vm::{middleware::helpers::HelperAccessList, rbpf_vm},
};

use super::{jit_deploy_handler::GenericRequestError, util::preprocess_request_raw};

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
    type ExtractRequestError = GenericRequestError;
    type BuildResponseError<M: MinimalWritableMessage> = GenericRequestError;

    fn extract_request_data<M: ReadableMessage>(
        &mut self,
        request: &M,
    ) -> Result<Self::RequestData, Self::ExtractRequestError> {
        let preprocessing_result: Result<String, u8> = preprocess_request_raw(request);

        let Ok(request_str) = preprocessing_result else {
            return preprocessing_result.err();
        };

        let parsed_request = SuitPullRequest::decode(request_str);
        let Ok(request) = parsed_request else {
            Err(coap_numbers::code::BAD_REQUEST);
        };

        let config = VMConfiguration::decode(request.config);

        debug!(
            "Received SUIT pull request: {:?}, config: {:?}",
            request, config
        );

        let fetch_result = suit_storage::suit_fetch(
            request.ip.as_str(),
            request.riot_netif.as_str(),
            request.manifest.as_str(),
            config.suit_slot,
            request.erase,
            config.binary_layout,
        );

        if let Ok(()) = fetch_result {
            debug!("SUIT fetch successful.");
        } else {
            let err = format!("SUIT fetch failed: {:?}", fetch_result.err().unwrap());
            debug!("{}", err);
            self.last_request_status = Err(err);
            Err(coap_numbers::code::BAD_REQUEST)?
        }

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
                        let _ = suit_storage::suit_erase(config.suit_slot);
                        Err(coap_numbers::code::BAD_REQUEST)?
                    }
                }
            };

            let interpreter = rbpf_vm::map_interpreter(config.binary_layout);

            if let Err(e) = rbpf::check_helpers(program, &helper_idxs, interpreter)
                .map_err(|e| format!("Helper verification failed: {}", e.error))
            {
                error!("{}", e);
                self.last_request_status = Err(e);
                let _ = suit_storage::suit_erase(config.suit_slot);
                Err(coap_numbers::code::BAD_REQUEST)?
            }
        }

        self.last_request_status = Ok(String::from(request.manifest));
        Ok(coap_numbers::code::CHANGED)
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response<M: MutableWritableMessage>(
        &mut self,
        response: &mut M,
        request: Self::RequestData,
    ) -> Result<(), Self::BuildResponseError<M>> {
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
        Ok(())
    }
}
