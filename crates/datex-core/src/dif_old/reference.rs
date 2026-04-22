use crate::{
    dif::{r#type::DIFTypeDefinition, value::DIFValueContainer},
    shared_values::shared_containers::{
        ReferencedSharedContainer, SharedContainer, SharedContainerMutability,
        mutability_as_int,
    },
};
use serde::{Deserialize, Serialize};
use crate::dif::r#type::DIFType;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DIFReference {
    pub value: DIFValueContainer,
    pub allowed_type: DIFType,
    #[serde(rename = "mut")]
    #[serde(with = "mutability_as_int")]
    pub mutability: SharedContainerMutability,
}

impl DIFReference {
    pub fn from_reference(reference: &ReferencedSharedContainer) -> Self {
        let value = DIFValueContainer::from_value_container(
            &reference.value_container(),
        );
        let allowed_type = DIFType::from_type(
            &*reference.allowed_type(),
        );
        DIFReference {
            value,
            allowed_type,
            mutability: reference.container_mutability(),
        }
    }
}
