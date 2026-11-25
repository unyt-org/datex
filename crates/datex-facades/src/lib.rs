#![no_std]
#![feature(associated_type_defaults)]
#[cfg(test)]
extern crate std;

// FIXME: add directory module for crypto, move error mod
pub mod crypto;
pub mod error;
pub mod vault;

extern crate alloc;
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
