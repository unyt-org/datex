use crate::{
    global::{
        dxb_block::{
            BlockId, DXB, IncomingEndpointContextId, SIZE_BYTE_POSITION,
            SIZE_BYTES,
            dxb_trait::DxB,
            error::{DXBBlockParseError, DxbHeaderParsingError},
        },
        protocol_structures::{
            block_header::{BlockHeader, BlockType},
            routing_header::{Receivers, RoutingHeader},
        },
    },
    prelude::*,
    utils::buffers::write_u16,
    values::core_values::endpoint::Endpoint,
};

use binrw::{
    BinRead, BinWrite,
    io::{Cursor, Read},
};
use core::fmt::Display;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct RawDXB {
    pub routing_header: RoutingHeader,
    pub block_header: BlockHeader,

    #[serde(skip)]
    pub body: Vec<u8>,
}

impl DxB for RawDXB {
    fn adjust_block_length(mut bytes: Vec<u8>) -> Vec<u8> {
        let size = bytes.len() as u32;
        write_u16(&mut bytes, &mut SIZE_BYTE_POSITION.clone(), size as u16);
        bytes
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        self.routing_header.write(&mut writer).unwrap();
        self.block_header.write(&mut writer).unwrap();
        let mut bytes = writer.into_inner();

        bytes.extend_from_slice(&self.body);
        RawDXB::adjust_block_length(bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Result<DXB, DXBBlockParseError> {
        let mut reader = Cursor::new(bytes);
        let routing_header = RoutingHeader::read(&mut reader)?;
        let block_header = BlockHeader::read(&mut reader)?;

        let mut body = Vec::new();
        reader.read_to_end(&mut body).map_err(|e| e.to_string())?;

        let block = RawDXB {
            routing_header,
            block_header,
            body: body.to_vec(),
        };

        Ok(DXB::RawDXB(block))
    }

    fn recalculate_struct(&mut self) -> &mut Self {
        let bytes = self.to_bytes();
        let size = bytes.len() as u16;
        self.routing_header.block_size = size;
        self
    }

    fn extract_dxb_block_length(
        dxb: &[u8],
    ) -> Result<u16, DxbHeaderParsingError> {
        if dxb.len() < SIZE_BYTE_POSITION + SIZE_BYTES {
            return Err(DxbHeaderParsingError::InsufficientLength);
        }

        // make sure magic number is correct
        if !RawDXB::has_dxb_magic_number(dxb) {
            return Err(DxbHeaderParsingError::InvalidMagicNumber);
        }

        // block size is u16 at SIZE_BYTE_POSITION
        let block_size_bytes =
            &dxb[SIZE_BYTE_POSITION..SIZE_BYTE_POSITION + SIZE_BYTES];
        Ok(u16::from_le_bytes(block_size_bytes.try_into().unwrap()))
    }
    fn receiver_endpoints(&self) -> Vec<Endpoint> {
        match self.routing_header.receivers() {
            Receivers::Endpoints(endpoints) => endpoints,
            Receivers::EndpointsWithKeys(endpoints_with_keys) => {
                endpoints_with_keys.into_iter().map(|(e, _)| e).collect()
            }
            Receivers::PointerId(_) => unimplemented!(),
            _ => Vec::new(),
        }
    }
    fn receivers(&self) -> Receivers {
        self.routing_header.receivers()
    }

    /// Update the receivers list in the routing header.
    fn set_receivers<T>(&mut self, endpoints: T)
    where
        T: Into<Receivers>,
    {
        self.routing_header.set_receivers(endpoints.into());
    }

    fn set_bounce_back(&mut self, bounce_back: bool) {
        self.routing_header.flags.set_is_bounce_back(bounce_back);
    }

    fn is_bounce_back(&self) -> bool {
        self.routing_header.flags.is_bounce_back()
    }

    fn sender(&self) -> &Endpoint {
        &self.routing_header.sender
    }

    fn block_type(&self) -> BlockType {
        self.block_header.flags_and_timestamp.block_type()
    }

    fn get_endpoint_context_id(&self) -> IncomingEndpointContextId {
        IncomingEndpointContextId {
            sender: self.routing_header.sender.clone(),
            context_id: self.block_header.context_id,
        }
    }

    fn get_block_id(&self) -> BlockId {
        BlockId {
            endpoint_context_id: self.get_endpoint_context_id(),
            timestamp: self
                .block_header
                .flags_and_timestamp
                .creation_timestamp(),
            current_section_index: self.block_header.section_index,
            current_block_number: self.block_header.block_number,
        }
    }

    /// Returns true if the block has a fixed number of receivers
    /// without wildcard instances, and no @@any receiver.
    fn has_exact_receiver_count(&self) -> bool {
        !self
            .receiver_endpoints()
            .iter()
            .any(|e| e.instance_is_broadcast() || e.is_any())
    }

    fn clone_with_new_receivers<T>(&self, new_receivers: T) -> DXB
    where
        T: Into<Receivers>,
    {
        let mut new_block = self.clone();
        new_block.set_receivers(new_receivers.into());
        DXB::RawDXB(new_block)
    }
}

impl Display for RawDXB {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let block_type = self.block_header.flags_and_timestamp.block_type();
        let sender = &self.routing_header.sender;
        let receivers = self.receivers();
        core::write!(f, "[{block_type}] {sender} -> {receivers}")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn raw_dxb_from_to_bytes_roundtrip() {
        let original_dxb = RawDXB::default();
        let bytes = original_dxb.to_bytes();
        let bytes_copy = RawDXB::from_bytes(&bytes).unwrap().to_bytes();

        assert_eq!(bytes, bytes_copy)
    }
}
