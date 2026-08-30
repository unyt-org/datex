pub use crate::{
    datex_proxy::{
        TryFromDatexValueError, TryToDatexValueError,
        serde_compat::{
            try_serde_from_value_container, try_serde_to_value_container,
        },
    },
    datex_registry::{get_impls, get_impls_for},
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    prelude::*,
    runtime::cache::shared_references_cache::{
        SharedReferencesCache, SharedTypeReservation,
    },
    shared_values::{
        SelfOwnedPointerAddress,
        errors::{AccessError, KeyNotFoundError},
    },
    traits::value_access::ValueAccess,
    types::{
        entities::entity_type_definition::EntityTypeDefinition,
        literal_type_definition::LiteralTypeDefinition,
        shared_container_containing_entity_type::SharedContainerContainingEntityType,
        r#type::Type,
        type_definition::{
            TypeDefinition, list::ListTypeDefinition, map::MapTypeDefinition,
            tagged_type::TaggedTypeDefinition, union::UnionTypeDefinition,
        },
    },
    values::{
        borrowed_value_container::{
            AsBorrowed, AsBorrowedMut, BorrowedValueContainer,
            BorrowedValueContainerMut,
        },
        core_value::CoreValue,
        core_values::{list::List, map::Map, native::DatexNative, text::Text},
        value::Value,
        value_container::{ValueContainer, value_key::BorrowedValueKey},
    },
};

#[cfg(feature = "decompiler")]
use crate::{
    ast,
    ast::{
        expressions::DatexExpressionData, expressions::Statements,
        spanned::Spanned,
    },
    traits::to_datex_expression_data::ToDatexExpressionData,
};
