use alloc::{format, string::String, vec::Vec};
use core::convert::TryInto;
use log::error;
use mibpf_common::{HelperAccessVerification, SuitPullRequest, VMConfiguration, HelperAccessListSource};
use serde::{Deserialize, Serialize};

use coap_message::{MutableWritableMessage, ReadableMessage};
use riot_wrappers::{stdio::println, thread};

use crate::{
    infra::suit_storage::{self, SUIT_STORAGE_SLOT_SIZE},
    vm::{middleware::{ALL_HELPERS, helpers::HelperAccessList}, rbpf_vm},
};

use super::util::preprocess_request_raw;

pub struct SuitPullHandler {
    last_fetched_manifest: Option<String>,
    success: bool,
}

impl SuitPullHandler {
    pub fn new() -> Self {
        Self {
            last_fetched_manifest: None,
            success: true,
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

        suit_storage::suit_fetch(
            request.ip.as_str(),
            request.riot_netif.as_str(),
            request.manifest.as_str(),
        );

        let config = VMConfiguration::decode(request.config);

        if config.helper_access_verification == HelperAccessVerification::LoadTime {
            let mut program_buffer = [0; SUIT_STORAGE_SLOT_SIZE];
            let mut program = suit_storage::load_program(&mut program_buffer, config.suit_slot);

            let helper_idxs: Vec<u32> = if config.helper_access_list_source == HelperAccessListSource::ExecuteRequest {
                HelperAccessList::from(request.helpers).0.into_iter().map(|f| f.id as u32).collect()
            } else {
                // In this case we need to parse the helpers out of the binary.
                alloc::vec![]
            };

            let interpreter = rbpf_vm::map_interpreter(config.binary_layout);

            if let Err(e) = rbpf::check_helpers(program, &helper_idxs, interpreter)
                .map_err(|e| format!("Error when checking helper function access: {:?}", e))
            {
                error!("{}", e);
                self.success = false;
                suit_storage::suit_erase(config.suit_slot);
                return coap_numbers::code::BAD_REQUEST;
            }
        }

        self.last_fetched_manifest = Some(String::from(request.manifest));
        self.success = true;
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
        let res = if self.success {
            format!(
                "SUIT pull request processed successfully for manifest: {}",
                self.last_fetched_manifest.as_ref().unwrap()
            )
        } else {
            format!("SUIT pull request failed, invalid set of helpers specified.")
        };
        response.set_payload(res.as_bytes());
    }
}
