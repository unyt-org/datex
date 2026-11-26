use binrw::io::Cursor;
use binrw::{BinRead, BinWrite};
use serde::{Deserialize, Serialize};
use datex_facades::crypto::Crypto;
use crate::crypto::CryptoImpl;

/// Contains the endpoint's own public/private key pair
#[derive(Clone, Debug, BinRead, BinWrite, PartialEq, Eq, Hash)]
#[brw(little)]
pub struct OwnEndpointKeys {
    pub pub_key: [u8; 32],
    pub pri_key: [u8; 32],
}

impl Serialize for OwnEndpointKeys {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.serialize_to_base58().map_err(serde::ser::Error::custom)?.as_str())
    }
}

impl<'de> Deserialize<'de> for OwnEndpointKeys {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let base58_str = String::deserialize(deserializer)?;
        OwnEndpointKeys::deserialize_from_base58(&base58_str).map_err(serde::de::Error::custom)
    }
}


impl OwnEndpointKeys {
    pub fn deserialize_from_bytes(bytes: &[u8]) -> Result<Self, binrw::Error> {
        let mut cursor = binrw::io::Cursor::new(bytes);
        OwnEndpointKeys::read(&mut cursor)
    }

    pub fn serialize_to_bytes(&self) -> Result<Vec<u8>, binrw::Error> {
        let mut cursor = Cursor::new(Vec::new());
        self.write(&mut cursor)?;
        let bytes = cursor.into_inner();
        Ok(bytes)
    }
    
    pub fn serialize_to_base58(&self) -> Result<String, binrw::Error> {
        let bytes = self.serialize_to_bytes()?;
        Ok(CryptoImpl::enc_b58(&bytes))
    }
    
    pub fn deserialize_from_base58(base58: &str) -> Result<Self, binrw::Error> {
        let bytes = CryptoImpl::dec_b58(base58).map_err(|e| binrw::Error::Custom {
            pos: 0,
            err: Box::new(e),
        })?;
        Self::deserialize_from_bytes(&bytes)
    }
}
