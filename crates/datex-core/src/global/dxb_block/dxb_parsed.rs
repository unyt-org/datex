use crate::{
    global::{
        dxb_block::{SIZE_BYTE_POSITION, error::DXBBlockParseError},
        protocol_structures::{
            block_header::BlockHeader,
            encrypted_header::EncryptedHeader,
            routing_header::{EncryptionType, RoutingHeader, SignatureType},
        },
    },
    prelude::*,
    utils::buffers::write_u16,
};

use binrw::{
    BinRead, BinWrite,
    io::{Cursor, Read},
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct ParsedDXB {
    pub routing_header: RoutingHeader,
    pub block_header: BlockHeader,
    pub encrypted_header: EncryptedHeader,
    pub signature: Option<Vec<u8>>,
    pub body: Vec<u8>,
}

impl ParsedDXB {
    fn adjust_block_length(mut bytes: Vec<u8>) -> Vec<u8> {
        let size = bytes.len() as u32;
        write_u16(&mut bytes, &mut SIZE_BYTE_POSITION.clone(), size as u16);
        bytes
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        self.routing_header.write(&mut writer).unwrap();
        self.block_header.write(&mut writer).unwrap();
        self.encrypted_header.write(&mut writer).unwrap();
        self.signature.write(&mut writer).unwrap();
        let mut bytes = writer.into_inner();

        bytes.extend_from_slice(&self.body);
        ParsedDXB::adjust_block_length(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<ParsedDXB, DXBBlockParseError> {
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
        Ok(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    pub fn parsed_dxb_default() {
        let dxb = ParsedDXB::default();
        println!("ParsedDXB: {:?}", dxb)
    }

    #[test]
    pub fn parsed_dxb_from_to_bytes_roundtrip() {
        let mut original_dxb = ParsedDXB::default();
        original_dxb.signature = Some(Vec::from([0u8; 64]));
        let bytes = original_dxb.to_bytes();
        let bytes_copy = ParsedDXB::from_bytes(&bytes).unwrap().to_bytes();

        assert_eq!(bytes, bytes_copy)
    }
}
