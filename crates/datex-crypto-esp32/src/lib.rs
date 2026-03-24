#![no_std]
#![feature(thread_local)]

#[cfg(test)]
extern crate std;

extern crate alloc;

use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use core::{future::Future, pin::Pin, result::Result};
use datex_crypto_facade::{
    crypto::{Crypto, CryptoResult},
    error::CryptoError,
};
use esp_hal::rtc_cntl::Rtc;

use aes::cipher::{KeyIvInit, StreamCipher};
use ed25519_dalek::{
    Signature, Signer, SigningKey, Verifier, VerifyingKey,
    pkcs8::{
        DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey,
        spki::der::pem::LineEnding,
    },
};
use hkdf::Hkdf;
use sha2::{Sha256, Digest};
use x25519_dalek::{PublicKey, StaticSecret};

use der::{Decode, Encode, asn1::BitStringRef};
use pkcs8::{AlgorithmIdentifierRef, ObjectIdentifier, PrivateKeyInfo};
use spki::SubjectPublicKeyInfoRef;

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
mod hal {
    use esp_hal::rng::Rng;
    use spin::{Mutex, MutexGuard, Once};
    use static_cell::StaticCell;

    static RNG: StaticCell<Mutex<Rng>> = StaticCell::new();
    static INIT: Once<&'static Mutex<Rng>> = Once::new();

    pub fn rng() -> MutexGuard<'static, Rng> {
        let m = INIT.call_once(|| RNG.init(Mutex::new(Rng::new())));

        m.lock()
    }
}

#[cfg(any(target_arch = "xtensa", target_arch = "riscv32"))]
pub use hal::rng;

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
        // TODO: use uuid crate?
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

    fn hash_sha256<'a>(to_digest: &'a [u8]) -> CryptoResult<'a, [u8; 32]> {
        Box::pin(async move {
            let x = Sha256::digest(to_digest);
            let y: [u8; 32] = x.try_into().unwrap();
            Ok(y)
        })
    }

    fn hkdf_sha256<'a>(
        ikm: &'a [u8],
        salt: &'a [u8],
    ) -> CryptoResult<'a, [u8; 32]> {
        Box::pin(async move {
            let mut okm = [0u8; 32];
            let ctx = Hkdf::<Sha256>::new(None, ikm);
            ctx.expand(b"", &mut okm).unwrap();
            Ok(okm)
        })
    }
    fn sig_ed25519<'a>(
        pri_key: &'a [u8],
        data: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<[u8; 64], CryptoError>> + 'a>> {
        Box::pin(async move {
            let prepped_key: [u8; 48] = pri_key.to_vec().try_into().unwrap();
            Ok(SigningKey::from_pkcs8_der(&prepped_key)
                .unwrap()
                .sign(data)
                .to_bytes())
        })
    }

    fn ver_ed25519<'a>(
        pub_key: &'a [u8],
        sig: &'a [u8],
        data: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<bool, CryptoError>> + 'a>> {
        Box::pin(async move {
            let sign: [u8; 64] = sig.try_into().unwrap();
            let ver = VerifyingKey::from_public_key_der(pub_key).unwrap();
            Ok(ver.verify(data, &Signature::from_bytes(&sign)).is_ok())
        })
    }

    fn aes_ctr_encrypt<'a>(
        key: &'a [u8; 32],
        iv: &'a [u8; 16],
        plaintext: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, CryptoError>> + 'a>> {
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
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, CryptoError>> + 'a>> {
        Self::aes_ctr_encrypt(key, iv, cipher)
    }

    fn key_upwrap<'a>(
        _kek_bytes: &'a [u8; 32],
        _rb: &'a [u8; 32],
    ) -> Pin<Box<dyn Future<Output = Result<[u8; 40], CryptoError>> + 'a>> {
        todo!()
    }

    fn key_unwrap<'a>(
        _kek_bytes: &'a [u8; 32],
        _cipher: &'a [u8; 40],
    ) -> Pin<Box<dyn Future<Output = Result<[u8; 32], CryptoError>> + 'a>> {
        todo!()
    }

    fn derive_x25519<'a>(
        pri_key: &'a [u8; 48],
        peer_pub: &'a [u8; 44],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, CryptoError>> + 'a>> {
        Box::pin(async move {
            /*
            let x = StaticSecret::from(pri_key);
            let shared_sec = x.diffie_hellman(&peer_pub.into()).to_bytes();
            Ok(shared_sec.to_vec());
            */
            let x: [u8; 32] = pri_key[16..].try_into().unwrap();
            let xx = StaticSecret::from(x);
            let y: [u8; 32] = peer_pub[12..].try_into().unwrap();
            let yy = PublicKey::from(y);
            Ok(xx.diffie_hellman(&yy).to_bytes().to_vec())
        })
    }

    fn gen_ed25519<'a>() -> CryptoResult<'a, (Vec<u8>, Vec<u8>)> {
        Box::pin(async move {
            let mut key = [0u8; 32];
            let x = SigningKey::from_bytes(&key);

            let oid = ObjectIdentifier::new("1.3.101.112").unwrap();
            let prepped_key =
                [[4u8, 32u8].to_vec(), x.to_bytes().to_vec()].concat();
            let pri_x = PrivateKeyInfo {
                algorithm: AlgorithmIdentifierRef {
                    oid,
                    parameters: None,
                },
                private_key: &prepped_key,
                public_key: None,
            }
            .to_der()
            .unwrap();
            // note: raw pub key
            // let temp = x.verifying_key().to_bytes();
            let pub_key = x
                .verifying_key()
                .to_public_key_der()
                .unwrap()
                .as_bytes()
                .to_vec();
            Ok((pub_key, pri_x))
        })
    }

    fn gen_x25519<'a>() -> CryptoResult<'a, ([u8; 44], [u8; 48])> {
        Box::pin(async move {
            /*
            let pri_key = StaticSecret::random().to_bytes();
            let pub_key = PublicKey::from(&pri_key).to_bytes();
            Ok((pub_key, pri_key.to_bytes()));
            */
            let pri_key = StaticSecret::from([0u8; 32]);
            let pub_key = PublicKey::from(&pri_key).to_bytes();
            let oid = ObjectIdentifier::new("1.3.101.110").unwrap();

            // For a historical reason the private key in pkcs8 is prefixed twice
            // with the octet string instruction code followed by the length of the octet string
            let prepped_key =
                [[4u8, 34u8].to_vec(), pri_key.to_bytes().to_vec()].concat();
            let pri_x = PrivateKeyInfo {
                algorithm: AlgorithmIdentifierRef {
                    oid,
                    parameters: None,
                },
                private_key: &prepped_key,
                public_key: None,
            }
            .to_der()
            .unwrap();

            // PEM encoding
            /*
            let pri_pem = pkcs8::SecretDocument::from_pkcs8_der(&pri_x)
                .unwrap()
                .to_pem("PRIVATE KEY", LineEnding::default())
                .unwrap()
                .to_ascii_uppercase();
            */

            let pub_spki = SubjectPublicKeyInfoRef {
                algorithm: AlgorithmIdentifierRef {
                    oid,
                    parameters: None,
                },
                subject_public_key: BitStringRef::new(0, &pub_key).unwrap(),
            }
            .to_der()
            .unwrap();

            // sanity check
            let z = PrivateKeyInfo::from_der(pri_x.as_slice()).unwrap();
            assert_eq!(prepped_key.to_vec(), z.private_key.to_vec());

            let public_key: [u8; 44] = pub_spki.try_into().unwrap();
            let private_key: [u8; 48] = pri_x.try_into().unwrap();
            Ok((public_key, private_key))
        })
    }
}

pub fn now_ms() -> u64 {
    let rtc = Rtc::new(unsafe {
        esp_hal::peripherals::Peripherals::steal()
            .LPWR
            .clone_unchecked()
    });
    rtc.current_time_us() / 1000
}
