use crate::{
    prelude::*,
    serde::{deserializer::from_bytes, serializer::to_bytes},
    shared_values::pointer_source::error::PointerSourceError,
    types::definition::TypeDefinition,
    values::value_container::ValueContainer,
};
pub trait PointerCodec: Send + Sync + 'static {
    fn encode_value(
        &self,
        value: &ValueContainer,
    ) -> Result<Vec<u8>, PointerSourceError>;
    fn decode_value(
        &self,
        bytes: &[u8],
    ) -> Result<ValueContainer, PointerSourceError>;

    fn encode_type(
        &self,
        ty: &TypeDefinition,
    ) -> Result<Vec<u8>, PointerSourceError>;
    fn decode_type(
        &self,
        bytes: &[u8],
    ) -> Result<TypeDefinition, PointerSourceError>;
}

pub struct BincodeCodec;

impl PointerCodec for BincodeCodec {
    fn encode_value(
        &self,
        value: &ValueContainer,
    ) -> Result<Vec<u8>, PointerSourceError> {
        to_bytes(value).map_err(|e| PointerSourceError::Backend(e.to_string()))
    }

    fn decode_value(
        &self,
        bytes: &[u8],
    ) -> Result<ValueContainer, PointerSourceError> {
        from_bytes(bytes)
            .map_err(|e| PointerSourceError::Backend(e.to_string()))
    }

    fn encode_type(
        &self,
        ty: &TypeDefinition,
    ) -> Result<Vec<u8>, PointerSourceError> {
        unimplemented!()
    }

    fn decode_type(
        &self,
        bytes: &[u8],
    ) -> Result<TypeDefinition, PointerSourceError> {
        unimplemented!()
    }
}
