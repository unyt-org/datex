use crate::{
    shared_values::ReferenceMutability,
    types::type_definition_with_metadata::metadata::TypeMetadata,
};
use core::fmt::Display;

use crate::{
    prelude::*,
    shared_values::{SharedContainerMutability, SharedContainerOwnership},
    types::{type_definition::TypeDefinition, type_match::TypeMatch},
    values::value_container::ValueContainer,
};

impl TypeMatch for TypeMetadata {
    fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (
                TypeMetadata::Local {
                    mutability: mutability1,
                    reference_mutability: reference_mutability1,
                },
                TypeMetadata::Local {
                    mutability: mutability2,
                    reference_mutability: reference_mutability2,
                },
            ) => {
                mutability1 == mutability2
                    && reference_mutability1 == reference_mutability2
            }
            (
                TypeMetadata::Shared {
                    mutability: mutability1,
                    ownership: ownership1,
                },
                TypeMetadata::Shared {
                    mutability: mutability2,
                    ownership: ownership2,
                },
            ) => mutability1 == mutability2 && ownership1 == ownership2,
            _ => false,
        }
    }

    fn matched_by_value(&self, _value: &ValueContainer) -> bool {
        unimplemented!()
    }
}
