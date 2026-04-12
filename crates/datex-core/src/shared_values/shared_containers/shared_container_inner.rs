use core::fmt::Display;
use binrw::{BinRead, BinWrite};
use num_enum::TryFromPrimitive;
use serde::Serialize;
use crate::serde::Deserialize;
use crate::shared_values::shared_container::ExternalSharedContainer;
use crate::shared_values::shared_containers::EndpointOwnedSharedContainer;
use crate::shared_values::shared_containers::shared_type_container::SharedTypeContainer;
use crate::shared_values::shared_containers::shared_value_container::SharedValueContainer;

/// Wrapper containing either an endpoint-owned shared container or an external shared container
#[derive(Debug)]
pub enum SharedContainerInner {
    EndpointOwned(EndpointOwnedSharedContainer),
    External(ExternalSharedContainer),
}


/// Wrapper containing either a [SharedValueContainer] or a [SharedTypeContainer]
#[derive(Debug, PartialEq)]
pub enum SharedContainerValueOrType {
    Value(SharedValueContainer),
    Type(SharedTypeContainer),
}