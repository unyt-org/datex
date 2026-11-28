use crate::{
    global::{
        dxb_block::{
            BlockId, DXB,
            error::{DXBBlockParseError, DxbHeaderParsingError},
            incoming_section::IncomingEndpointContextId,
        },
        protocol_structures::{
            block_header::BlockType, routing_header::Receivers,
        },
    },
    values::core_values::endpoint::Endpoint,
};

use crate::prelude::*;

pub trait DxB {
    fn has_dxb_magic_number(dxb: &[u8]) -> bool {
        dxb.len() >= 2 && dxb[0] == 0x01 && dxb[1] == 0x64
    }

    /// Returns true if the block has a fixed number of receivers
    /// without wildcard instances, and no @@any receiver.
    fn has_exact_receiver_count(&self) -> bool {
        !self
            .receiver_endpoints()
            .iter()
            .any(|e| e.instance_is_broadcast() || e.is_any())
    }

    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Result<DXB, DXBBlockParseError>;

    fn recalculate_struct(&mut self) -> &mut Self;

    fn adjust_block_length(bytes: Vec<u8>) -> Vec<u8>;
    fn extract_dxb_block_length(
        dxb: &[u8],
    ) -> Result<u16, DxbHeaderParsingError>;

    fn sender(&self) -> &Endpoint;
    fn receivers(&self) -> Receivers;
    fn receiver_endpoints(&self) -> Vec<Endpoint>;

    /// Update the receivers list in the routing header.
    fn set_receivers<T>(&mut self, endpoints: T)
    where
        T: Into<Receivers>;
    fn clone_with_new_receivers<T>(&self, new_receivers: T) -> DXB
    where
        T: Into<Receivers>;

    fn block_type(&self) -> BlockType;
    fn get_block_id(&self) -> BlockId;
    fn get_endpoint_context_id(&self) -> IncomingEndpointContextId;

    fn is_bounce_back(&self) -> bool;
    fn set_bounce_back(&mut self, bounce_back: bool);
}
