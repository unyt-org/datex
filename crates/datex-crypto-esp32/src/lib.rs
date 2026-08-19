#![no_std]
#![feature(thread_local)]

#[cfg(test)]
extern crate std;

extern crate alloc;
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
use alloc::{format, string::String, vec, vec::Vec};
use datex_crypto_facade::crypto::{AsyncCryptoResult, Crypto};

use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use datex_crypto_facade::{
    crypto::{AsyncCryptoResult, Crypto, CryptoVault, PQCrypto},
    error::BackendError,
};

use aes::cipher::{KeyIvInit, StreamCipher};
use aes_kw::KekAes256;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

use ml_dsa::{
    Generate, Keypair, MlDsa44, Signer as ml_dsa_signer,
    SigningKey as ml_dsa_signing_key, Verifier as ml_dsa_verifier,
};
use ml_kem::{
    FromSeed, MlKem512, TryKeyInit,
    kem::{Decapsulate, Encapsulate, Kem, KeyExport},
};

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
mod hal {
    use esp_hal::rng::Rng;
    use spin::{Mutex, MutexGuard, Once, Spin};
    use static_cell::StaticCell;

    static RNG: StaticCell<Mutex<Rng>> = StaticCell::new();
    static INIT: Once<&'static Mutex<Rng>> = Once::new();

    pub fn rng() -> MutexGuard<'static, Rng, Spin> {
        let m = INIT.call_once(|| RNG.init(Mutex::new(Rng::new())));

        m.lock()
    }
}

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
pub use hal::rng;

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
use esp_hal::rng::Rng;

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(
    dest: *mut u8,
    len: usize,
) -> Result<(), getrandom::Error> {
    unsafe { esp_hal::rng::Rng::new().read_into_raw(dest, len) };
    Ok(())
}

struct InfallibleRng;
impl InfallibleRng {
    fn read(&mut self, _: &mut [u8]) {
        panic!("RNG not supported on this platform");
    }
}

#[cfg(not(any(target_arch = "xtensa", target_arch = "riscv32")))]
fn rng() -> InfallibleRng {
    InfallibleRng
}

#[derive(Debug, Clone)]
pub struct CryptoEsp32;

impl Crypto for CryptoEsp32 {
    fn create_uuid() -> String {
        // TODO #705: use uuid crate?
        let mut bytes = [0u8; 16];
        rng().read(&mut bytes);

        // set version to 4 -- random
        bytes[6] = (bytes[6] & 0x0F) | 0x40;
        // set variant to RFC 4122
        bytes[8] = (bytes[8] & 0x3F) | 0x80;
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u16::from_be_bytes([bytes[4], bytes[5]]),
            u16::from_be_bytes([bytes[6], bytes[7]]),
            u16::from_be_bytes([bytes[8], bytes[9]]),
            u64::from_be_bytes([
                bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                bytes[15], 0, 0
            ]) >> 16
        )
    }

    fn random_bytes(length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        rng().read(&mut bytes);
        bytes
    }

    fn hash_sha256<'a>(
        to_digest: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::Sha256Error> {
        Box::pin(async move {
            let hash: [u8; 32] = Sha256::digest(to_digest).into();
            Ok(hash)
        })
    }

    fn hkdf_sha256<'a>(
        ikm: &'a [u8],
        _salt: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::HkdfError> {
        Box::pin(async move {
            let mut okm = [0u8; 32];
            let ctx = Hkdf::<Sha256>::new(None, ikm);
            ctx.expand(b"", &mut okm).map_err(|_| {
                Self::HkdfError::Backend(BackendError::Unavailable("hkdf ctx"))
            })?;
            Ok(okm)
        })
    }

    fn gen_ed25519<'a>()
    -> AsyncCryptoResult<'a, ([u8; 32], [u8; 32]), Self::Ed25519GenError> {
        Box::pin(async move {
            let key: [u8; 32] =
                Self::random_bytes(32).try_into().map_err(|_| {
                    Self::Ed25519GenError::Backend(BackendError::Unavailable(
                        "ed25519 key gen rng",
                    ))
                })?;
            let pri_key = SigningKey::from_bytes(&key);
            let pub_key = pri_key.verifying_key().to_bytes();
            Ok((pub_key, pri_key.to_bytes()))
        })
    }

    fn sig_ed25519<'a>(
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, [u8; 64], Self::Ed25519SignError> {
        Box::pin(async move {
            let prepped_key: [u8; 32] =
                pri_key.to_vec().try_into().map_err(|_| {
                    Self::Ed25519SignError::Backend(BackendError::Unavailable(
                        "ed25519 private key format",
                    ))
                })?;

            Ok(SigningKey::from_bytes(&prepped_key).sign(data).to_bytes())
        })
    }

    fn ver_ed25519<'a>(
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> AsyncCryptoResult<'a, bool, Self::Ed25519VerifyError> {
        Box::pin(async move {
            let sign: [u8; 64] = sig
                .try_into()
                .map_err(|_| Self::Ed25519VerifyError::InvalidSignature)?;
            let prepped_key: [u8; 32] = pub_key
                .to_vec()
                .try_into()
                .map_err(|_| Self::Ed25519VerifyError::InvalidPublicKey)?;
            let ver = VerifyingKey::from_bytes(&prepped_key).map_err(|_| {
                Self::Ed25519VerifyError::Backend(BackendError::Unavailable(
                    "ed 25519 verify",
                ))
            })?;
            Ok(ver.verify(data, &Signature::from_bytes(&sign)).is_ok())
        })
    }

    fn aes_ctr_encrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        plaintext: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
        Box::pin(async move {
            type Aes128Ctr64LE = ctr::Ctr64LE<aes::Aes256>;
            let mut msg = plaintext.to_vec();
            let mut cipher = Aes128Ctr64LE::new(key.into(), iv.into());
            cipher.apply_keystream(msg.as_mut_slice());
            Ok(msg)
        })
    }

    fn aes_ctr_decrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        cipher: &'a [u8],
    ) -> AsyncCryptoResult<'a, Vec<u8>, Self::AesCtrError> {
        Self::aes_ctr_encrypt(key, iv, cipher)
    }

    fn key_wrap_rfc3394<'a>(
        kek: &'a [u8; 32],
        key_to_wrap: &'a [u8; 32],
    ) -> AsyncCryptoResult<'a, [u8; 40], Self::KeyWrapError> {
        Box::pin(async move {
            let x = KekAes256::new(kek.into());
            let mut buf = [0u8; 40];
            x.wrap(key_to_wrap.as_slice(), &mut buf).map_err(|_| {
                Self::KeyWrapError::Backend(BackendError::Unavailable("aes-kw"))
            })?;
            Ok(buf)
        })
    }

    fn key_unwrap_rfc3394<'a>(
        kek: &'a [u8; 32],
        wrapped: &'a [u8; 40],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::KeyUnwrapError> {
        Box::pin(async move {
            let x = KekAes256::new(kek.into());
            let mut buf = [0u8; 32];
            let _ = x.unwrap(wrapped.as_slice(), &mut buf).map_err(|_| {
                Self::KeyWrapError::Backend(BackendError::Unavailable("aes-kw"))
            });
            Ok(buf)
        })
    }

    fn gen_x25519<'a>()
    -> AsyncCryptoResult<'a, ([u8; 32], [u8; 32]), Self::X25519GenError> {
        Box::pin(async move {
            let key: [u8; 32] =
                Self::random_bytes(32).try_into().map_err(|_| {
                    Self::X25519GenError::Backend(BackendError::Unavailable(
                        "x25519 key gen rng",
                    ))
                })?;
            let pri_key = StaticSecret::from(key);
            let pub_key = PublicKey::from(&pri_key).to_bytes();
            Ok((pub_key, pri_key.to_bytes()))
        })
    }

    fn derive_x25519<'a>(
        pri_key: &'a [u8; 32],
        peer_pub: &'a [u8; 32],
    ) -> AsyncCryptoResult<'a, [u8; 32], Self::X25519DeriveError> {
        Box::pin(async move {
            let x: [u8; 32] = pri_key.to_vec().try_into().map_err(|_| {
                Self::X25519DeriveError::Backend(BackendError::Unavailable(
                    "x25519 private key (shared secret derivation)",
                ))
            })?;
            let y: [u8; 32] = peer_pub.to_vec().try_into().map_err(|_| {
                Self::X25519DeriveError::Backend(BackendError::Unavailable(
                    "x25519 public key (shared secret derivation)",
                ))
            })?;
            let private_key = StaticSecret::from(x);
            let public_key = PublicKey::from(y);
            Ok(private_key.diffie_hellman(&public_key).to_bytes())
        })
    }
}

impl PQCrypto for CryptoEsp32 {
    // hack around async implementation
    fn gen_ed25519_cheat() -> Result<([u8; 32], [u8; 32]), BackendError> {
        let key: [u8; 32] = Self::random_bytes(32)
            .try_into()
            .map_err(|_| BackendError::Unavailable("ed25519 key gen rng"))?;
        let pri_key = SigningKey::from_bytes(&key);
        let pub_key = pri_key.verifying_key().to_bytes();
        Ok((pub_key, pri_key.to_bytes()))
    }

    fn gen_x25519_cheat() -> Result<([u8; 32], [u8; 32]), BackendError> {
        let key: [u8; 32] = Self::random_bytes(32)
            .try_into()
            .map_err(|_| BackendError::Unavailable("x25519 key gen rng"))?;
        let pri_key = StaticSecret::from(key);
        let pub_key = PublicKey::from(&pri_key).to_bytes();
        Ok((pub_key, pri_key.to_bytes()))
    }

    fn derive_x25519_cheat(
        pri_key: &[u8; 32],
        peer_pub: &[u8; 32],
    ) -> Result<[u8; 32], BackendError> {
        let x: [u8; 32] = pri_key.to_vec().try_into().map_err(|_| {
            BackendError::Unavailable(
                "x25519 private key (shared secret derivation)",
            )
        })?;
        let y: [u8; 32] = peer_pub.to_vec().try_into().map_err(|_| {
            BackendError::Unavailable(
                "x25519 public key (shared secret derivation)",
            )
        })?;
        let private_key = StaticSecret::from(x);
        let public_key = PublicKey::from(y);
        Ok(private_key.diffie_hellman(&public_key).to_bytes())
    }

    fn hkdf_cheat(ikm: &[u8], salt: &[u8]) -> Result<[u8; 32], BackendError> {
        let mut okm = [0u8; 32];
        let ctx = Hkdf::<Sha256>::new(None, ikm);
        ctx.expand(b"", &mut okm)
            .map_err(|_| BackendError::Unavailable("hkdf ctx"))?;
        Ok(okm)
    }

    fn aes_cheat(
        key: &[u8; 32],
        iv: &[u8; 16],
        data: &[u8],
    ) -> Result<Vec<u8>, BackendError> {
        type Aes128Ctr64LE = ctr::Ctr64LE<aes::Aes256>;
        let mut msg = data.to_vec();
        let mut cipher = Aes128Ctr64LE::new(key.into(), iv.into());
        cipher.apply_keystream(msg.as_mut_slice());
        Ok(msg)
    }

    fn aes_kw_wrap_cheat(
        kek: &[u8; 32],
        key_to_wrap: &[u8; 32],
    ) -> Result<[u8; 40], BackendError> {
        let x = KekAes256::new(kek.into());
        let mut buf = [0u8; 40];
        x.wrap(key_to_wrap.as_slice(), &mut buf)
            .map_err(|_| BackendError::Unavailable("aes-kw"))?;
        Ok(buf)
    }

    fn aes_kw_unwrap_cheat(
        kek: &[u8; 32],
        wrapped: &[u8; 40],
    ) -> Result<[u8; 32], BackendError> {
        let x = KekAes256::new(kek.into());
        let mut buf = [0u8; 32];
        let _ = x
            .unwrap(wrapped.as_slice(), &mut buf)
            .map_err(|_| BackendError::Unavailable("aes-kw"));
        Ok(buf)
    }
}

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
pub fn now_ms() -> u64 {
    let rtc = esp_hal::rtc_cntl::Rtc::new(unsafe {
        esp_hal::peripherals::Peripherals::steal()
            .LPWR
            .clone_unchecked()
    });
    rtc.current_time_us() / 1000
}
