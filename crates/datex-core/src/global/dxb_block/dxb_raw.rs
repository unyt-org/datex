use crate::{
    global::{
        dxb_block::{SIZE_BYTE_POSITION, error::DXBBlockParseError},
        protocol_structures::{
            block_header::BlockHeader, routing_header::RoutingHeader,
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
pub struct RawDXB {
    pub routing_header: RoutingHeader,
    pub block_header: BlockHeader,

    #[serde(skip)]
    pub body: Vec<u8>,
}

impl RawDXB {
    fn adjust_block_length(mut bytes: Vec<u8>) -> Vec<u8> {
        let size = bytes.len() as u32;
        write_u16(&mut bytes, &mut SIZE_BYTE_POSITION.clone(), size as u16);
        bytes
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        self.routing_header.write(&mut writer).unwrap();
        self.block_header.write(&mut writer).unwrap();
        let mut bytes = writer.into_inner();

        bytes.extend_from_slice(&self.body);
        RawDXB::adjust_block_length(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<RawDXB, DXBBlockParseError> {
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

        Ok(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn raw_dxb_default() {
        let dxb = RawDXB::default();
        println!("RawDXB: {:?}", dxb)
    }

    #[test]
    pub fn raw_dxb_from_to_bytes_roundtrip() {
        let original_dxb = RawDXB::default();
        let bytes = original_dxb.to_bytes();
        let bytes_copy = RawDXB::from_bytes(&bytes).unwrap().to_bytes();

        assert_eq!(bytes, bytes_copy)
    }
}
