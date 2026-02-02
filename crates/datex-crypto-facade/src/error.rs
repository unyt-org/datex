use alloc::string::String;
use core::fmt::Display;

#[derive(Debug, Clone)]
pub enum CryptoError {
    Other(String),
    KeyGeneration,
    KeyExport,
    KeyImport,
    Encryption,
    Decryption,
    Signing,
    Verification,
}

impl Display for CryptoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CryptoError::Other(msg) => core::write!(f, "Crypto: {}", msg),
            CryptoError::KeyGeneration => {
                core::write!(f, "CryptoError: Key generation failed")
            }
            CryptoError::KeyExport => {
                core::write!(f, "CryptoError: Key export failed")
            }
            CryptoError::KeyImport => {
                core::write!(f, "CryptoError: Key import failed")
            }
            CryptoError::Encryption => {
                core::write!(f, "CryptoError: Encryption failed")
            }
            CryptoError::Decryption => {
                core::write!(f, "CryptoError: Decryption failed")
            }
            CryptoError::Signing => {
                core::write!(f, "CryptoError: Signing failed")
            }
            CryptoError::Verification => {
                core::write!(f, "CryptoError: Verification failed")
            }
        }
    }
}
