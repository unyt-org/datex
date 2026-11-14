use core::fmt::Display;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum B58DecodeError {
    InvalidBase58,
    WrongLength { expected: usize, got: usize },
}
impl Display for B58DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            B58DecodeError::InvalidBase58 => write!(f, "Invalid Base58 string"),
            B58DecodeError::WrongLength { expected, got } => write!(
                f,
                "Invalid length for Base58 decoded data: expected {} bytes, got {} bytes",
                expected, got
            ),
        }
    }
}

// crate::error.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    /// The backend/platform cannot do this operation (algo not available, feature off).
    Unsupported(&'static str),
    /// The backend could do it, but is not currently usable (not initialized, no entropy, etc).
    Unavailable(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RandomBytesError {
    Backend(BackendError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sha256Error {
    Backend(BackendError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HkdfError {
    Backend(BackendError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ed25519GenError {
    Backend(BackendError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ed25519SignError {
    /// Private key bytes not acceptable (wrong length/format).
    InvalidPrivateKey,
    Backend(BackendError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ed25519VerifyError {
    /// Public key bytes not acceptable (wrong length/format).
    InvalidPublicKey,
    /// Signature bytes not acceptable (wrong length/format).
    InvalidSignature,
    Backend(BackendError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AesCtrError {
    Backend(BackendError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyWrapError {
    Backend(BackendError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyUnwrapError {
    IntegrityCheckFailed,
    Backend(BackendError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum X25519GenError {
    Backend(BackendError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum X25519DeriveError {
    InvalidPrivateKey,
    InvalidPeerPublicKey,
    Backend(BackendError),
}
