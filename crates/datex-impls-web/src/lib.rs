use datex_facades::crypto::{AsyncCryptoResult, Crypto};
use wasm_bindgen::{JsCast, JsError, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AesCtrParams, CryptoKey, CryptoKeyPair, js_sys,
    js_sys::{Array, ArrayBuffer, Object, Reflect, Uint8Array},
};
mod utils;
pub mod vault;
pub mod crypto;


#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
