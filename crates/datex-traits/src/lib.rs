#![no_std]
#![feature(associated_type_defaults)]
#[cfg(test)]
extern crate std;

extern crate alloc;
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
