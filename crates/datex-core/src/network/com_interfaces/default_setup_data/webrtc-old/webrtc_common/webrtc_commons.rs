use crate::{
    core_compiler::{
        core_compilation_context::CompileInput,
        value_compiler::compile_value_container,
    },
    datex_proxy::DatexValueContainerProxySerialize,
    runtime::pointer_availability_lookup::PointerAvailabilityLookup,
    values::core_values::endpoint::Endpoint,
};
use alloc::collections::VecDeque;
use datex_macros_internal::Datex;
use log::error;
use serde::{Deserialize, Serialize};

use super::structures::{RTCIceCandidateInitDX, RTCIceServer};

pub struct WebRTCCommon {
    pub endpoint: Endpoint,
    pub ice_servers: Vec<RTCIceServer>,
    pub candidates: VecDeque<Vec<u8>>,
    pub is_remote_description_set: bool,
    pub on_ice_candidate: Option<Box<dyn Fn(Vec<u8>)>>,
    pub on_connect: Option<Box<dyn Fn()>>,
}

impl WebRTCCommon {
    pub fn reset(&mut self) {
        self.is_remote_description_set = false;
        self.candidates.clear();
        self.on_ice_candidate = None;
    }
    pub fn new(endpoint: impl Into<Endpoint>) -> Self {
        WebRTCCommon {
            endpoint: endpoint.into(),
            candidates: VecDeque::new(),
            is_remote_description_set: false,
            on_ice_candidate: None,
            on_connect: None,
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            }],
        }
    }
    pub fn on_ice_candidate(&self, candidate: RTCIceCandidateInitDX) {
        if let Some(ref on_ice_candidate) = self.on_ice_candidate {
            if let Ok(candidate) = candidate.try_to_value_container() {
                let pointer_lookup = PointerAvailabilityLookup::default();
                let compile_input = CompileInput::new(&pointer_lookup, &vec![]);
                on_ice_candidate(
                    compile_value_container(candidate, compile_input).dxb,
                );
            } else {
                error!("Failed to serialize candidate");
            }
        } else {
            error!("No on_ice_candidate callback set");
        }
    }
}

#[derive(Datex)]
#[datex(structural)]
pub struct WebRTCInterfaceSetupData {
    pub peer_endpoint: Endpoint,
    pub ice_servers: Option<Vec<RTCIceServer>>,
}
