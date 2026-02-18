use super::protocol_structures::{
    block_header::BlockHeader,
    encrypted_header::EncryptedHeader,
    routing_header::{EncryptionType, RoutingHeader, SignatureType},
};
use crate::{
    channel::mpsc::UnboundedReceiver,
    crypto::CryptoImpl,
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
use datex_crypto_facade::crypto::Crypto;
use strum::Display;
use thiserror::Error;

use crate::prelude::*;
#[derive(Debug, Display, Error)]
pub enum HeaderParsingError {
    InsufficientLength,
    InvalidMagicNumber,
}

// TODO #110: RawDXBBlock that is received in com_hub, only containing RoutingHeader, BlockHeader and raw bytes

// TODO #429 @Norbert
// Add optional raw signature, and encrypted part
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Debug, Clone, Default)]
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

const SIZE_BYTE_POSITION: usize = 3; // magic number (2 bytes) + version (1 byte)
const SIZE_BYTES: usize = 2;

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
    /// the stream is finished when a block has the end_of_block flag set
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

/// An identifier that defines a globally unique block
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockId {
    pub endpoint_context_id: IncomingEndpointContextId,
    pub timestamp: u64,
    pub current_section_index: IncomingSectionIndex,
    pub current_block_number: IncomingBlockNumber,
}

#[derive(Debug)]
pub enum DXBBlockParseError {
    IOError(String),
    ParseError(binrw::Error),
}

#[derive(Debug)]
pub enum SignatureValidationError {
    /// The block is expected to have a signature, but no signature was found.
    MissingSignature,
    /// The signature could not be parsed correctly (e.g. wrong length).
    SignatureParseError,
    /// The signature is invalid (e.g. does not match the expected value).
    InvalidSignature,
}

impl From<binrw::Error> for DXBBlockParseError {
    fn from(err: binrw::Error) -> Self {
        DXBBlockParseError::ParseError(err)
    }
}

impl From<String> for DXBBlockParseError {
    fn from(err: String) -> Self {
        DXBBlockParseError::IOError(err)
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

    // TODO: guarantee that all unwraps are safe and no binrw::Error is possible
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
    ) -> Result<u16, HeaderParsingError> {
        if dxb.len() < SIZE_BYTE_POSITION + SIZE_BYTES {
            return Err(HeaderParsingError::InsufficientLength);
        }

        // make sure magic number is correct
        if !DXBBlock::has_dxb_magic_number(dxb) {
            return Err(HeaderParsingError::InvalidMagicNumber);
        }

        // block size is u16 at SIZE_BYTE_POSITION
        let block_size_bytes =
            &dxb[SIZE_BYTE_POSITION..SIZE_BYTE_POSITION + SIZE_BYTES];
        Ok(u16::from_le_bytes(block_size_bytes.try_into().unwrap()))
    }

    pub async fn from_bytes(
        bytes: &[u8],
    ) -> Result<DXBBlock, DXBBlockParseError> {
        let mut reader = Cursor::new(bytes);
        let routing_header = RoutingHeader::read(&mut reader)?;

        let signature = match routing_header.flags.signature_type() {
            SignatureType::Encrypted => {
                // extract next 255 bytes as the signature
                let mut signature = Vec::from([0u8; 108]);
                reader.read_exact(&mut signature).map_err(|e| {
                    DXBBlockParseError::IOError(format!(
                        "Failed to read encrypted signature: {}",
                        e
                    ))
                })?;

                // TODO #111: decrypt the signature
                Some(signature)
            }
            SignatureType::Unencrypted => {
                // extract next 255 bytes as the signature
                let mut signature = Vec::from([0u8; 108]);
                reader.read_exact(&mut signature).map_err(|e| {
                    DXBBlockParseError::IOError(format!(
                        "Failed to read unencrypted signature: {}",
                        e
                    ))
                })?;
                Some(signature)
            }
            SignatureType::None => None,
        };

        // TODO #112: validate the signature
        let decrypted_bytes = match routing_header.flags.encryption_type() {
            EncryptionType::Encrypted => {
                // TODO #113: decrypt the body
                let mut decrypted_bytes = Vec::from([0u8; 255]);
                reader.read_exact(&mut decrypted_bytes).map_err(|e| {
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

    /// Validates the signature of the block based on the signature type specified in the routing header.
    /// Returns Ok(()) if the signature is valid, or a SignatureValidationError if the signature is missing, cannot be parsed, or is invalid.
    pub async fn validate_signature(&self) -> Result<(), SignatureValidationError> {

        // if not crypto_enabled, but allow_unsigned_blocks is set, just return Ok(()) for all blocks, as signature validation is not possible
        #[cfg(all(not(feature = "crypto_enabled"), feature = "allow_unsigned_blocks"))]
        {
            return Ok(());
        }

        // TODO #179 check for creation time, withdraw if too old (TBD) or in the future
        let is_valid_signature = match self.routing_header.flags.signature_type() {

            // TODO #180: verify signature and abort if invalid
            // Check if signature is following in some later block and add them to
            // a pool of incoming blocks awaiting some signature

            SignatureType::Encrypted => {
                let raw_sign = self
                    .signature
                    .as_ref()
                    .ok_or(SignatureValidationError::MissingSignature)?;
                let (enc_sign, pub_key) = raw_sign.split_at(64);
                let hash = CryptoImpl::hkdf_sha256(pub_key, &[0u8; 16])
                    .await
                    .map_err(|_| SignatureValidationError::SignatureParseError)?;
                let signature = CryptoImpl::aes_ctr_decrypt(
                    &hash, &[0u8; 16], enc_sign,
                )
                    .await
                    .map_err(|_| SignatureValidationError::SignatureParseError)?;

                let raw_signed =
                    [pub_key, &self.body.clone()].concat();
                let hashed_signed =
                    CryptoImpl::hash_sha256(&raw_signed)
                        .await
                        .map_err(|_| SignatureValidationError::SignatureParseError)?;

                CryptoImpl::ver_ed25519(
                    pub_key,
                    &signature,
                    &hashed_signed,
                )
                    .await
                    .map_err(|_| SignatureValidationError::SignatureParseError)?
            }
            SignatureType::Unencrypted => {
                let raw_sign = self
                    .signature
                    .as_ref()
                    .ok_or(SignatureValidationError::MissingSignature)?;
                let (signature, pub_key) = raw_sign.split_at(64);

                let raw_signed =
                    [pub_key, &self.body.clone()].concat();
                let hashed_signed =
                    CryptoImpl::hash_sha256(&raw_signed)
                        .await
                        .map_err(|_| SignatureValidationError::SignatureParseError)?;

                CryptoImpl::ver_ed25519(
                    pub_key,
                    signature,
                    &hashed_signed,
                )
                    .await
                    .map_err(|_| SignatureValidationError::SignatureParseError)?
            }

            SignatureType::None => {
                cfg_if::cfg_if! {
                    // if unsigned blocks are allowed, return true
                    if #[cfg(feature = "allow_unsigned_blocks")] {
                        true
                    }
                    // otherwise, only allow unsigned Trace and TraceBack blocks,
                    // as they are used for debugging and should not be used in production with real data
                    else {
                        match self.block_type() {
                            BlockType::Trace | BlockType::TraceBack => true,
                            // TODO #181 Check if the sender is trusted (endpoint + interface) connection
                            _ => return Err(SignatureValidationError::MissingSignature)
                        }
                    }
                }
            }
        };

        match is_valid_signature {
            true => Ok(()),
            false => Err(SignatureValidationError::InvalidSignature)
        }
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
            .any(|e| e.is_broadcast() || e.is_any())
    }

    pub fn clone_with_new_receivers<T>(&self, new_receivers: T) -> DXBBlock
    where
        T: Into<Receivers>,
    {
        let mut new_block = self.clone();
        new_block.set_receivers(new_receivers.into());
        new_block
    }

    /// Sets the default signature type based on whether the "crypto_enabled" feature is enabled.
    /// When "crypto_enabled" is enabled, the default signature type is set to Unencrypted
    /// Otherwise, it is set to None.
    pub fn set_default_signature_type(&mut self) {
        #[cfg(not(feature = "crypto_enabled"))]
        {
            self
                .routing_header
                .flags
                .set_signature_type(SignatureType::None);
        }
        #[cfg(feature = "crypto_enabled")]
        {
            self
                .routing_header
                .flags
                .set_signature_type(SignatureType::Unencrypted);
        }
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
        crypto::CryptoImpl,
        global::{
            dxb_block::{DXBBlock, DXBBlockParseError},
            protocol_structures::{
                encrypted_header::{self, EncryptedHeader},
                routing_header::{RoutingHeader, SignatureType},
            },
        },
        prelude::*,
        values::core_values::endpoint::Endpoint,
    };
    use core::assert_matches;
    use datex_crypto_facade::crypto::Crypto;
    use crate::global::dxb_block::SignatureValidationError;

    #[tokio::test]
    pub async fn test_recalculate() {
        let mut routing_header = RoutingHeader::default()
            .with_sender(Endpoint::from_str("@test").unwrap())
            .to_owned();
        routing_header.set_size(420);
        let mut block = DXBBlock {
            body: vec![0x01, 0x02, 0x03],
            encrypted_header: EncryptedHeader {
                flags: encrypted_header::Flags::new()
                    .with_user_agent(encrypted_header::UserAgent::Unused11),
                ..Default::default()
            },
            routing_header,
            ..DXBBlock::default()
        };

        {
            // invalid block size
            let block_bytes = block.to_bytes();
            let block2: DXBBlock =
                DXBBlock::from_bytes(&block_bytes).await.unwrap();
            assert_ne!(block, block2);
        }

        {
            // valid block size
            block.recalculate_struct();
            let block_bytes = block.to_bytes();
            let block3: DXBBlock =
                DXBBlock::from_bytes(&block_bytes).await.unwrap();
            assert_eq!(block, block3);
        }
    }

    #[tokio::test]
    #[cfg(feature = "std")]
    pub async fn signature_to_and_from_bytes() {
        // setup block
        let mut routing_header = RoutingHeader::default()
            .with_sender(Endpoint::from_str("@test").unwrap())
            .to_owned();
        routing_header.set_size(157);
        let mut block = DXBBlock {
            body: vec![0x01, 0x02, 0x03],
            encrypted_header: EncryptedHeader {
                ..Default::default()
            },
            routing_header,
            ..DXBBlock::default()
        };

        // setup correct signature
        block
            .routing_header
            .flags
            .set_signature_type(SignatureType::Unencrypted);

        let (pub_key, pri_key) = CryptoImpl::gen_ed25519().await.unwrap();
        let raw_signed = [pub_key.clone(), block.body.clone()].concat();
        let hashed_signed = CryptoImpl::hash_sha256(&raw_signed).await.unwrap();

        let signature = CryptoImpl::sig_ed25519(&pri_key, &hashed_signed)
            .await
            .unwrap();
        // 64 + 44 = 108
        block.signature = Some([signature.to_vec(), pub_key.clone()].concat());

        let block_bytes = block.to_bytes();
        let block2: DXBBlock =
            DXBBlock::from_bytes(&block_bytes).await.unwrap();
        assert_eq!(block, block2);
        assert_eq!(block.signature, block2.signature);

        // setup faulty signature
        let mut other_sig = signature;
        if other_sig[42] != 42u8 {
            other_sig[42] = 42u8;
        } else {
            other_sig[42] = 43u8;
        }
        block.signature = Some([other_sig.to_vec(), pub_key].concat());
        let block_bytes2 = block.to_bytes();
        let signature_validation = DXBBlock::from_bytes(&block_bytes2).await.unwrap().validate_signature().await;
        assert!(signature_validation.is_err());
        assert_matches!(
            signature_validation.unwrap_err(),
            SignatureValidationError::InvalidSignature
        )
    }
}
