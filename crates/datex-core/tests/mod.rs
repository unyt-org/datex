#![feature(assert_matches)]
#![feature(thread_local)]
#![feature(box_patterns)]
#![feature(gen_blocks)]
#![allow(static_mut_refs)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::alloc_instead_of_core)]
#![allow(clippy::std_instead_of_alloc)]

extern crate alloc;
extern crate core;

// pub mod network;
pub mod values;

#[cfg(all(feature = "compiler", feature = "decompiler"))]
pub mod compiler;
#[cfg(all(feature = "compiler", feature = "decompiler"))]
pub mod json;

#[cfg(feature = "parser")]
pub mod parser;

#[cfg(feature = "compiler")]
pub mod execution;

pub mod network;