//! This module contains internal utilities for working with macros in the DATEX system.
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::alloc_instead_of_core)]
#![allow(clippy::std_instead_of_alloc)]

use std::{env, path::PathBuf, str::FromStr};

use proc_macro2::Span;
pub mod entrypoint;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

/// Gets the absolute file path of the source file where the macro is invoked.
pub fn get_absolute_file_path() -> PathBuf {
    let root_path = PathBuf::from_str(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()),
    )
    .unwrap();
    root_path
        .join(Span::call_site().file())
        .canonicalize()
        .unwrap()
}
