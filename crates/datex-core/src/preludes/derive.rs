#[doc(hidden)]
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
    traits::datex_native_structural::DatexNativeStructural,
    traits::datex_native_only_structural::DatexNativeOnlyStructural,
    traits::has_classification::HasClassification,
    traits::classification::Classification,
    traits::get_datex_type::GetDatexType,
    traits::get_core_lib_type_id::GetCoreLibTypeId,
    traits::convert_parts::FromParts,
    traits::convert_parts::IntoParts,
    types::{
        entities::entity_type_definition::EntityTypeDefinition,
        literal_type_definition::LiteralTypeDefinition,
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
    utils::{goat::Goat, goat_mut::GoatMut},
    values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut},
};

#[cfg(feature = "decompiler")]
pub use crate::{
    ast,
    ast::{
        expressions::DatexExpressionData, expressions::Statements,
        spanned::Spanned,
    },
    traits::to_datex_expression_data::ToDatexExpressionData,
};
