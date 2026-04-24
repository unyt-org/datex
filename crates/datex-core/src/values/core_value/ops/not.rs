use crate::{
    libs::core::type_id::{
        CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId,
    },
    prelude::*,
    runtime::memory::Memory,
    traits::{structural_eq::StructuralEq, value_eq::ValueEq},
    types::{
        nominal_type_definition::NominalTypeDefinition,
        shared_container_containing_nominal_type::SharedContainerContainingNominalType,
        r#type::Type,
    },
    values::{
        core_value::CoreValue,
        core_values::{
            boolean::Boolean,
            callable::Callable,
            decimal::{
                Decimal,
                typed_decimal::{DecimalTypeVariant, TypedDecimal},
            },
            endpoint::Endpoint,
            integer::{
                Integer,
                typed_integer::{IntegerTypeVariant, TypedInteger},
            },
            list::List,
            map::Map,
            range::Range,
            text::Text,
        },
        value_container::{ValueContainer, error::ValueError},
    },
};
use core::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Neg, Not, Sub},
    result::Result,
};
use datex_macros_internal::FromCoreValue;

impl Not for CoreValue {
    type Output = Option<CoreValue>;

    fn not(self) -> Self::Output {
        match self {
            CoreValue::Boolean(bool) => Some(CoreValue::Boolean(!bool)),
            _ => None, // Not applicable for other types
        }
    }
}
