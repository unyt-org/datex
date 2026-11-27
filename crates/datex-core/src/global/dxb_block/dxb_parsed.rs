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
            encrypted_header::EncryptedHeader,
            routing_header::{Receivers, RoutingHeader, SignatureType},
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
pub struct ParsedDXB {
    pub routing_header: RoutingHeader,
    pub block_header: BlockHeader,
    pub encrypted_header: EncryptedHeader,
    pub signature: Option<Vec<u8>>,
    pub body: Vec<u8>,
}

impl PartialEq for ParsedDXB {
    fn eq(&self, other: &Self) -> bool {
        self.routing_header == other.routing_header
            && self.block_header == other.block_header
            && self.encrypted_header == other.encrypted_header
            && self.body == other.body
    }
}

impl Display for ParsedDXB {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let block_type = self.block_header.flags_and_timestamp.block_type();
        let sender = &self.routing_header.sender;
        let receivers = self.receivers();
        core::write!(f, "[{block_type}] {sender} -> {receivers}")?;

        Ok(())
    }
}

impl DxB for ParsedDXB {
    fn adjust_block_length(mut bytes: Vec<u8>) -> Vec<u8> {
        let size = bytes.len() as u32;
        write_u16(&mut bytes, &mut SIZE_BYTE_POSITION.clone(), size as u16);
        bytes
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        self.routing_header.write(&mut writer).unwrap();
        self.block_header.write(&mut writer).unwrap();
        self.encrypted_header.write(&mut writer).unwrap();
        self.signature.write(&mut writer).unwrap();
        let mut bytes = writer.into_inner();

        bytes.extend_from_slice(&self.body);
        ParsedDXB::adjust_block_length(bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Result<DXB, DXBBlockParseError> {
        let mut reader = Cursor::new(bytes);
        let routing_header = RoutingHeader::read(&mut reader)?;
        let block_header = BlockHeader::read(&mut reader)?;
        let encrypted_header = EncryptedHeader::read(&mut reader)?;

        let signature = match routing_header.flags.signature_type() {
            SignatureType::Signed | SignatureType::CryptedSigned => {
                let mut signature = Vec::from([0u8; 64]);
                reader.read_exact(&mut signature).map_err(|e| {
                    DXBBlockParseError::IOError(format!(
                        "Failed to read encrypted signature: {}",
                        e
                    ))
                })?;
                Some(signature)
            }
            _ => None,
        };

        let mut body = Vec::new();
        reader.read_to_end(&mut body).map_err(|e| e.to_string())?;

        let block = ParsedDXB {
            routing_header,
            block_header,
            signature,
            encrypted_header,
            body,
        };

        Ok(DXB::ParsedDXB(block))
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
        if !ParsedDXB::has_dxb_magic_number(dxb) {
            return Err(DxbHeaderParsingError::InvalidMagicNumber);
        }

        // block size is u16 at SIZE_BYTE_POSITION
        let block_size_bytes =
            &dxb[SIZE_BYTE_POSITION..SIZE_BYTE_POSITION + SIZE_BYTES];
        Ok(u16::from_le_bytes(block_size_bytes.try_into().unwrap()))
    }

    /// Get a list of all receiver endpoints from the routing header.
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
        DXB::ParsedDXB(new_block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;

    use crate::{
        global::{
            dxb_block::ParsedDXB,
            protocol_structures::{
                encrypted_header::{self, EncryptedHeader},
                routing_header::{RoutingHeader, SignatureType},
            },
        },
        prelude::*,
        values::core_values::endpoint::Endpoint,
    };

    #[test]
    pub fn parsed_dxb_from_to_bytes_roundtrip() {
        let mut original_dxb = ParsedDXB::default();
        original_dxb.signature = Some(Vec::from([0u8; 64]));
        let bytes = original_dxb.to_bytes();
        let bytes_copy = ParsedDXB::from_bytes(&bytes).unwrap().to_bytes();

        assert_eq!(bytes, bytes_copy)
    }

    #[test]
    pub fn recalculate_parsed_dxb() {
        let mut routing_header = RoutingHeader::default()
            .with_sender(Endpoint::from_str("@test").unwrap())
            .to_owned();
        routing_header
            .flags
            .set_signature_type(SignatureType::Unsigned);
        routing_header.set_size(420);
        let mut block = ParsedDXB {
            body: vec![0x01, 0x02, 0x03],
            encrypted_header: EncryptedHeader {
                flags: encrypted_header::Flags::default()
                    .with_user_agent(encrypted_header::UserAgent::Unused11),
                ..Default::default()
            },
            routing_header,
            ..ParsedDXB::default()
        };

        {
            // invalid block size
            let block_bytes = block.to_bytes();
            let block2 = match ParsedDXB::from_bytes(&block_bytes).unwrap() {
                DXB::ParsedDXB(block) => block,
                DXB::RawDXB(_) => unreachable!("This shouldn't happen."),
            };

            assert_ne!(block, block2);
        }

        {
            // valid block size
            block.recalculate_struct();
            let block_bytes = block.to_bytes();
            let block3 = match ParsedDXB::from_bytes(&block_bytes).unwrap() {
                DXB::ParsedDXB(block) => block,
                DXB::RawDXB(_) => unreachable!("This shouldn't happen."),
            };

            assert_eq!(block, block3);
        }
    }
}
