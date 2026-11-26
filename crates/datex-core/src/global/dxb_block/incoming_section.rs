use crate::{
    channel::mpsc::UnboundedReceiver, global::dxb_block::DXBBlock,
    values::core_values::endpoint::Endpoint,
};

pub type IncomingContextId = u32;
pub type IncomingSectionIndex = u16;
pub type IncomingBlockNumber = u16;
pub type OutgoingContextId = u32;
pub type OutgoingSectionIndex = u16;
pub type OutgoingBlockNumber = u16;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum IncomingSection {
    /// a single block
    SingleBlock((Option<DXBBlock>, IncomingEndpointContextSectionId)),
    /// a stream of blocks
    /// the stream is finished when a block has the end_of_section flag set
    BlockStream(
        (
            Option<UnboundedReceiver<DXBBlock>>,
            IncomingEndpointContextSectionId,
        ),
    ),
}

impl IncomingSection {
    pub async fn next(&mut self) -> Option<DXBBlock> {
        match self {
            IncomingSection::SingleBlock((block, _)) => block.take(),
            IncomingSection::BlockStream((blocks, _)) => {
                if let Some(receiver) = blocks {
                    receiver.next().await
                } else {
                    None // No blocks to receive
                }
            }
        }
    }

    pub async fn drain(&mut self) -> Vec<DXBBlock> {
        let mut blocks = Vec::new();
        while let Some(block) = self.next().await {
            blocks.push(block);
        }
        blocks
    }
}

impl IncomingSection {
    pub fn get_section_index(&self) -> IncomingSectionIndex {
        self.get_section_context_id().section_index
    }

    pub fn get_sender(&self) -> Endpoint {
        self.get_section_context_id()
            .endpoint_context_id
            .sender
            .clone()
    }

    pub fn get_section_context_id(&self) -> &IncomingEndpointContextSectionId {
        match self {
            IncomingSection::SingleBlock((_, section_context_id))
            | IncomingSection::BlockStream((_, section_context_id)) => {
                section_context_id
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IncomingEndpointContextId {
    pub sender: Endpoint,
    pub context_id: IncomingContextId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IncomingEndpointContextSectionId {
    pub endpoint_context_id: IncomingEndpointContextId,
    pub section_index: IncomingSectionIndex,
}

impl IncomingEndpointContextSectionId {
    pub fn new(
        endpoint_context_id: IncomingEndpointContextId,
        section_index: IncomingSectionIndex,
    ) -> Self {
        IncomingEndpointContextSectionId {
            endpoint_context_id,
            section_index,
        }
    }
}
