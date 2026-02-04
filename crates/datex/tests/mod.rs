#![feature(assert_matches)]
#![feature(iter_from_coroutine)]
#![feature(coroutines)]
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

pub mod json;
pub mod parser;
