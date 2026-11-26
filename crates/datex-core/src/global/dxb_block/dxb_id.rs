use crate::global::dxb_block::{
    IncomingEndpointContextId,
    incoming_section::{IncomingBlockNumber, IncomingSectionIndex},
};

/// An identifier that defines a globally unique block
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockId {
    pub endpoint_context_id: IncomingEndpointContextId,
    pub timestamp: u64,
    pub current_section_index: IncomingSectionIndex,
    pub current_block_number: IncomingBlockNumber,
}
