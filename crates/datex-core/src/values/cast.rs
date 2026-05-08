use crate::{
    types::{nominal_type_definition::NominalTypeDefinition, r#type::Type},
    values::{
        core_value::CoreValue,
        core_values::{
            boolean::Boolean,
            callable::Callable,
            decimal::{Decimal, typed_decimal::TypedDecimal},
            endpoint::Endpoint,
            integer::{Integer, typed_integer::TypedInteger},
            list::List,
            map::Map,
            range::Range,
            text::Text,
        },
    },
};
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;
use crate::prelude::*;

macro_rules! impl_try_from_core_value {
    ($($variant:ident => $type:ty),* $(,)?) => {
        $(
            impl TryFrom<CoreValue> for $type {
                type Error = ();
                fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
                    match value { CoreValue::$variant(v) => Ok(v), _ => Err(()) }
                }
            }
        )*
    };
}

// Implement [TryFrom] for each CoreValue variant
impl_try_from_core_value! {
    Integer             => Integer,
    TypedInteger        => TypedInteger,
    Decimal             => Decimal,
    TypedDecimal        => TypedDecimal,
    Boolean             => Boolean,
    Endpoint            => Endpoint,
    Text                => Text,
    List                => List,
    Map                 => Map,
    Type                => Type,
    NominalTypeDefinition => NominalTypeDefinition,
    Range               => Range,
    Callable            => Callable,
}

macro_rules! derive_try_from_chain {
    ($type:ty, { $($core_match:tt)* }) => {
        impl TryFrom<CoreValue> for $type {
            type Error = ();

            fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
               match value {
                    $($core_match)*
                    _ => Err(()),
               }
            }
        }

        impl TryFrom<Value> for $type {
            type Error = ();

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                value.inner.try_into()
            }
        }

        impl TryFrom<ValueContainer> for $type {
            type Error = ();

            fn try_from(value: ValueContainer) -> Result<Self, Self::Error> {
                match value {
                    ValueContainer::Local(value) => value.try_into(),
                    _ => Err(()),
                }
            }
        }
    };
}


derive_try_from_chain!(
    bool,
    {
        CoreValue::Boolean(Boolean(value)) => Ok(value),
    }
);


derive_try_from_chain!(
    u8,
    {
       CoreValue::TypedInteger(TypedInteger::U8(value)) => Ok(value),
    }
);

derive_try_from_chain!(
    String,
    {
        CoreValue::Text(Text(value)) => Ok(value),
    }
);