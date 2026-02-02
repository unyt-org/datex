use crypto::signature;
pub type Sig = ed25519::Signature;

use crate::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct Ed25519Keypair;

impl Ed25519Keypair {
    pub fn generate() -> Result<Self, signature::Error> {
        Ok(Self)
    }

    pub fn public_key_der(&self) -> &[u8] {
        &[]
    }

    pub fn private_key_pkcs8(&self) -> &[u8] {
        &[]
    }
}

impl signature::Signer<Sig> for Ed25519Keypair {
    fn try_sign(&self, _msg: &[u8]) -> Result<Sig, signature::Error> {
        Ed25519Signer::default().try_sign(_msg)
    }
}

impl signature::Verifier<Sig> for Ed25519Keypair {
    fn verify(
        &self,
        msg: &[u8],
        signature: &Sig,
    ) -> Result<(), signature::Error> {
        Ed25519Verifier::new().verify(msg, signature)
    }
}

#[derive(Clone, Default)]
pub struct Ed25519Signer {
    private_key_pkcs8: Vec<u8>,
}

impl Ed25519Signer {
    pub fn new(private_key_pkcs8: impl Into<Vec<u8>>) -> Self {
        Self {
            private_key_pkcs8: private_key_pkcs8.into(),
        }
    }

    pub fn random_bytes(&self, len: usize) -> Vec<u8> {
        vec![0u8; len]
    }
}

#[derive(Clone, Default)]
pub struct Ed25519Verifier;

impl Ed25519Verifier {
    pub fn new() -> Self {
        Self
    }
}

impl signature::Signer<ed25519::Signature> for Ed25519Signer {
    /// Signs a message, returning the signature.
    /// In this stub implementation, signing is not supported and will panic if called.
    fn try_sign(
        &self,
        _msg: &[u8],
    ) -> Result<ed25519::Signature, signature::Error> {
        unreachable!("stub signer: signing is not supported")
    }
}

impl signature::Verifier<ed25519::Signature> for Ed25519Verifier {
    /// Verifies a message against a signature.
    /// In this stub implementation, all signatures are considered valid.
    fn verify(
        &self,
        _msg: &[u8],
        _signature: &ed25519::Signature,
    ) -> Result<(), signature::Error> {
        // Always "OK"
        Ok(())
    }
}
