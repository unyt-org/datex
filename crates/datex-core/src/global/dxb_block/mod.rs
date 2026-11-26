pub mod dxb_id;
pub mod error;
pub mod incoming_section;

use dxb_id::BlockId;
use error::{DXBBlockParseError, DxbHeaderParsingError};
use incoming_section::IncomingEndpointContextId;

use super::protocol_structures::{
    block_header::BlockHeader,
    encrypted_header::EncryptedHeader,
    routing_header::{EncryptionType, RoutingHeader, SignatureType},
};
use crate::{
    global::protocol_structures::{
        block_header::BlockType, routing_header::Receivers,
    },
    utils::buffers::write_u16,
    values::core_values::endpoint::Endpoint,
};
use binrw::{
    BinRead, BinWrite,
    io::{Cursor, Read},
};
use core::{fmt::Display, result::Result, unimplemented};

use crate::prelude::*;

const SIZE_BYTE_POSITION: usize = 3; // magic number (2 bytes) + version (1 byte)
const SIZE_BYTES: usize = 2;

// TODO #110: RawDXBBlock that is received in com_hub, only containing RoutingHeader, BlockHeader and raw bytes
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct DXBBlock {
    pub routing_header: RoutingHeader,
    pub block_header: BlockHeader,
    pub signature: Option<Vec<u8>>,
    pub encrypted_header: EncryptedHeader,
    pub body: Vec<u8>,

    #[serde(skip)]
    pub raw_bytes: Option<Vec<u8>>,
}

impl PartialEq for DXBBlock {
    fn eq(&self, other: &Self) -> bool {
        self.routing_header == other.routing_header
            && self.block_header == other.block_header
            && self.encrypted_header == other.encrypted_header
            && self.body == other.body
    }
}

impl DXBBlock {
    pub fn new_with_body(body: &[u8]) -> DXBBlock {
        let mut block = DXBBlock {
            body: body.to_vec(),
            ..DXBBlock::default()
        };
        block.recalculate_struct();
        block
    }
    pub fn new(
        routing_header: RoutingHeader,
        block_header: BlockHeader,
        encrypted_header: EncryptedHeader,
        body: Vec<u8>,
    ) -> DXBBlock {
        let mut block = DXBBlock {
            routing_header,
            block_header,
            signature: None,
            encrypted_header,
            body,
            raw_bytes: None,
        };
        block.recalculate_struct();
        block
    }

    // TODO #743: guarantee that all unwraps are safe and no binrw::Error is possible
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        self.routing_header.write(&mut writer).unwrap();
        self.signature.write(&mut writer).unwrap();
        self.block_header.write(&mut writer).unwrap();
        self.encrypted_header.write(&mut writer).unwrap();
        let mut bytes = writer.into_inner();
        bytes.extend_from_slice(&self.body);
        DXBBlock::adjust_block_length(bytes)
    }
    pub fn recalculate_struct(&mut self) -> &mut Self {
        let bytes = self.to_bytes();
        let size = bytes.len() as u16;
        self.routing_header.block_size = size;
        self
    }

    fn adjust_block_length(mut bytes: Vec<u8>) -> Vec<u8> {
        let size = bytes.len() as u32;
        write_u16(&mut bytes, &mut SIZE_BYTE_POSITION.clone(), size as u16);
        bytes
    }

    pub fn has_dxb_magic_number(dxb: &[u8]) -> bool {
        dxb.len() >= 2 && dxb[0] == 0x01 && dxb[1] == 0x64
    }

    pub fn extract_dxb_block_length(
        dxb: &[u8],
    ) -> Result<u16, DxbHeaderParsingError> {
        if dxb.len() < SIZE_BYTE_POSITION + SIZE_BYTES {
            return Err(DxbHeaderParsingError::InsufficientLength);
        }

        // make sure magic number is correct
        if !DXBBlock::has_dxb_magic_number(dxb) {
            return Err(DxbHeaderParsingError::InvalidMagicNumber);
        }

        // block size is u16 at SIZE_BYTE_POSITION
        let block_size_bytes =
            &dxb[SIZE_BYTE_POSITION..SIZE_BYTE_POSITION + SIZE_BYTES];
        Ok(u16::from_le_bytes(block_size_bytes.try_into().unwrap()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<DXBBlock, DXBBlockParseError> {
        let mut reader = Cursor::new(bytes);
        let routing_header = RoutingHeader::read(&mut reader)?;

        let signature = match routing_header.flags.signature_type() {
            SignatureType::CryptedSigned => {
                let mut signature = Vec::from([0u8; 64]);
                reader.read_exact(&mut signature).map_err(|e| {
                    DXBBlockParseError::IOError(format!(
                        "Failed to read encrypted signature: {}",
                        e
                    ))
                })?;

                Some(signature)
            }
            SignatureType::Signed => {
                let mut signature = Vec::from([0u8; 64]);
                reader.read_exact(&mut signature).map_err(|e| {
                    DXBBlockParseError::IOError(format!(
                        "Failed to read unencrypted signature: {}",
                        e
                    ))
                })?;
                Some(signature)
            }
            SignatureType::Unsigned => None,
        };

        let decrypted_bytes = match routing_header.flags.encryption_type() {
            EncryptionType::Encrypted => {
                let mut decrypted_bytes = Vec::new();
                reader.read_to_end(&mut decrypted_bytes).map_err(|e| {
                    DXBBlockParseError::IOError(format!(
                        "Failed to read encrypted body: {}",
                        e
                    ))
                })?;
                decrypted_bytes
            }
            EncryptionType::None => {
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
                bytes
            }
        };

        let mut reader = Cursor::new(decrypted_bytes);
        let block_header = BlockHeader::read(&mut reader)?;

        // invalid state: trace / trace back blocks with a signature
        if signature.is_some()
            && block_header
                .flags_and_timestamp
                .block_type()
                .is_tracing_block()
        {
            return Err(DXBBlockParseError::IllegalState);
        }

        let encrypted_header = EncryptedHeader::read(&mut reader)?;

        let mut body = Vec::new();
        reader.read_to_end(&mut body).map_err(|e| e.to_string())?;

        let block = DXBBlock {
            routing_header,
            block_header,
            signature,
            encrypted_header,
            body,
            raw_bytes: Some(bytes.to_vec()),
        };

        Ok(block)
    }

    /// Get a list of all receiver endpoints from the routing header.
    pub fn receiver_endpoints(&self) -> Vec<Endpoint> {
        match self.routing_header.receivers() {
            Receivers::Endpoints(endpoints) => endpoints,
            Receivers::EndpointsWithKeys(endpoints_with_keys) => {
                endpoints_with_keys.into_iter().map(|(e, _)| e).collect()
            }
            Receivers::PointerId(_) => unimplemented!(),
            _ => Vec::new(),
        }
    }
    pub fn receivers(&self) -> Receivers {
        self.routing_header.receivers()
    }

    /// Update the receivers list in the routing header.
    pub fn set_receivers<T>(&mut self, endpoints: T)
    where
        T: Into<Receivers>,
    {
        self.routing_header.set_receivers(endpoints.into());
    }

    pub fn set_bounce_back(&mut self, bounce_back: bool) {
        self.routing_header.flags.set_is_bounce_back(bounce_back);
    }

    pub fn is_bounce_back(&self) -> bool {
        self.routing_header.flags.is_bounce_back()
    }

    pub fn sender(&self) -> &Endpoint {
        &self.routing_header.sender
    }

    pub fn block_type(&self) -> BlockType {
        self.block_header.flags_and_timestamp.block_type()
    }

    pub fn get_endpoint_context_id(&self) -> IncomingEndpointContextId {
        IncomingEndpointContextId {
            sender: self.routing_header.sender.clone(),
            context_id: self.block_header.context_id,
        }
    }

    pub fn get_block_id(&self) -> BlockId {
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
    pub fn has_exact_receiver_count(&self) -> bool {
        !self
            .receiver_endpoints()
            .iter()
            .any(|e| e.instance_is_broadcast() || e.is_any())
    }

    pub fn clone_with_new_receivers<T>(&self, new_receivers: T) -> DXBBlock
    where
        T: Into<Receivers>,
    {
        let mut new_block = self.clone();
        new_block.set_receivers(new_receivers.into());
        new_block
    }
}

impl Display for DXBBlock {
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
    use core::str::FromStr;

    use crate::{
        global::{
            dxb_block::{DXBBlock, error::DXBBlockParseError},
            protocol_structures::{
                block_header::BlockType,
                encrypted_header::{self, EncryptedHeader},
                routing_header::{RoutingHeader, SignatureType},
            },
        },
        prelude::*,
        values::core_values::endpoint::Endpoint,
    };
    use core::assert_matches;

    #[test]
    pub fn recalculate() {
        let mut routing_header = RoutingHeader::default()
            .with_sender(Endpoint::from_str("@test").unwrap())
            .to_owned();
        routing_header
            .flags
            .set_signature_type(SignatureType::Unsigned);
        routing_header.set_size(420);
        let mut block = DXBBlock {
            body: vec![0x01, 0x02, 0x03],
            encrypted_header: EncryptedHeader {
                flags: encrypted_header::Flags::default()
                    .with_user_agent(encrypted_header::UserAgent::Unused11),
                ..Default::default()
            },
            routing_header,
            ..DXBBlock::default()
        };

        {
            // invalid block size
            let block_bytes = block.to_bytes();
            let block2: DXBBlock = DXBBlock::from_bytes(&block_bytes).unwrap();
            assert_ne!(block, block2);
        }

        {
            // valid block size
            block.recalculate_struct();
            let block_bytes = block.to_bytes();
            let block3: DXBBlock = DXBBlock::from_bytes(&block_bytes).unwrap();
            assert_eq!(block, block3);
        }
    }

    #[test]
    fn illegal_signed_trace_block() {
        let mut block = DXBBlock::new_with_body(&[0x01, 0x02, 0x03]);
        block
            .block_header
            .flags_and_timestamp
            .set_block_type(BlockType::Trace);
        block
            .routing_header
            .flags
            .set_signature_type(SignatureType::Signed);
        block.signature = Some(vec![0u8; 64]);

        let block_bytes = block.to_bytes();
        let parse_result = DXBBlock::from_bytes(&block_bytes);
        assert!(parse_result.is_err());
        assert_matches!(
            parse_result.unwrap_err(),
            DXBBlockParseError::IllegalState
        );
    }
}
