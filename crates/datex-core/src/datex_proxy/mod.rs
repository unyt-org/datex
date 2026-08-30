pub mod serde_compat;
pub mod shared;

use crate::{
    core_compiler::core_compilation_context::DXBWithSharedValues,
    datex_proxy::shared::Shared,
    prelude::*,
    runtime::{
        Runtime,
        cache::shared_references_cache::SharedReferencesCache,
        execution::{ExecutionError, context::ScriptExecutionError},
        pointer_address_provider::SelfOwnedPointerAddressProvider,
    },
    shared_values::errors::KeyNotFoundError,
    types::r#type::Type,
    values::{
        core_value::CoreValue, core_values::native::DatexNative, value::Value,
        value_container::ValueContainer,
    },
};

#[derive(Debug, Clone)]
pub struct TryFromDatexValueError(pub String);

#[derive(Debug, Clone)]
pub struct TryToDatexValueError(pub String);