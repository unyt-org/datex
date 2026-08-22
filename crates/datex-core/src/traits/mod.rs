//! This module contains various traits that are shared on different levels such as [apply], [identity], [structural_eq] and [value_eq].
pub mod apply;
pub mod callable;
pub mod child_iterator;
pub mod clone_unsafe;
pub mod identity;
pub mod local_child_path_resolver;
pub mod structural_eq;
pub mod value_eq;
#[cfg(feature = "decompiler")]
pub mod to_datex_expression_data;
#[cfg(feature = "decompiler")]
pub mod to_type_expression_data;
