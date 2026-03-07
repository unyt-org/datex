use crate::{
    dif::update::DIFUpdate,
    prelude::*,
    shared_values::pointer_source::{
        error::PointerSourceError, resolve_request::ResolveRequest,
    },
};
use async_trait::async_trait;

use crate::{
    shared_values::{observers::TransceiverId, pointer::Pointer},
    types::definition::TypeDefinition,
    values::value_container::ValueContainer,
};
pub mod codec;
pub mod error;
pub mod resolve_request;

#[cfg(feature = "sqlite_pointer_source")]
pub mod sqlite;

#[derive(Debug, Clone)]
pub enum ResolveCompleteness {
    Partial,
    Full,
}

#[derive(Debug, Clone)]
pub struct ResolvedPointer {
    pub value_container: ValueContainer,
    pub allowed_type: Option<TypeDefinition>,
    pub completeness: ResolveCompleteness,
    pub version: Option<u64>,
}

pub trait PointerKey {
    fn storage_key(&self) -> String;
}
#[async_trait(?Send)]
pub trait AsyncPointerSource: Send + Sync + 'static {
    fn id(&self) -> TransceiverId;
    fn name(&self) -> &'static str;

    async fn has_pointer(
        &self,
        pointer: &Pointer,
    ) -> Result<bool, PointerSourceError>;

    async fn resolve_pointer(
        &self,
        pointer: &Pointer,
        request: &ResolveRequest,
    ) -> Result<ResolvedPointer, PointerSourceError>;

    async fn put_pointer(
        &self,
        pointer: &Pointer,
        value: &ValueContainer,
        allowed_type: Option<&TypeDefinition>,
    ) -> Result<(), PointerSourceError>;

    async fn update_pointer(
        &self,
        _pointer: &Pointer,
        _update: &DIFUpdate,
    ) -> Result<(), PointerSourceError> {
        Err(PointerSourceError::Unsupported)
    }
}
