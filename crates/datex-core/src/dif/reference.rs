use crate::{
    dif::{r#type::DIFTypeDefinition, value::DIFValueContainer},
    shared_values::shared_container::{
        SharedContainer, SharedContainerMutability, mutability_as_int,
    },
    runtime::memory::Memory,
};
use core::{cell::RefCell, prelude::rust_2024::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DIFReference {
    pub value: DIFValueContainer,
    pub allowed_type: DIFTypeDefinition,
    #[serde(rename = "mut")]
    #[serde(with = "mutability_as_int")]
    pub mutability: SharedContainerMutability,
}

impl DIFReference {
    pub fn from_reference(
        reference: &SharedContainer,
    ) -> Self {
        let value = DIFValueContainer::from_value_container(
            &reference.value_container(),
        );
        let allowed_type = DIFTypeDefinition::from_type_definition(
            &reference.allowed_type(),
        );
        DIFReference {
            value,
            allowed_type,
            mutability: reference.mutability(),
        }
    }
}
