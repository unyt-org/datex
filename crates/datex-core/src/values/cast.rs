use crate::{
    prelude::*,
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
        value::Value,
        value_container::ValueContainer,
    },
};

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
    u16,
    {
       CoreValue::TypedInteger(TypedInteger::U16(value)) => Ok(value),
    }
);
derive_try_from_chain!(
    u32,
    {
       CoreValue::TypedInteger(TypedInteger::U32(value)) => Ok(value),
    }
);
derive_try_from_chain!(
    u64,
    {
       CoreValue::TypedInteger(TypedInteger::U64(value)) => Ok(value),
    }
);
derive_try_from_chain!(
    usize,
    {
       CoreValue::TypedInteger(TypedInteger::U64(value)) => Ok(value as usize),
    }
);
derive_try_from_chain!(
    i8,
    {
       CoreValue::TypedInteger(TypedInteger::I8(value)) => Ok(value),
    }
);
derive_try_from_chain!(
    i16,
    {
       CoreValue::TypedInteger(TypedInteger::I16(value)) => Ok(value),
    }
);
derive_try_from_chain!(
    i32,
    {
       CoreValue::TypedInteger(TypedInteger::I32(value)) => Ok(value),
    }
);
derive_try_from_chain!(
    i64,
    {
       CoreValue::TypedInteger(TypedInteger::I64(value)) => Ok(value),
    }
);
derive_try_from_chain!(
    isize,
    {
       CoreValue::TypedInteger(TypedInteger::I64(value)) => Ok(value as isize),
    }
);
derive_try_from_chain!(
    f32,
    {
       CoreValue::TypedDecimal(TypedDecimal::F32(value)) => Ok(value.into()),
    }
);
derive_try_from_chain!(
    f64,
    {
       CoreValue::TypedDecimal(TypedDecimal::F64(value)) => Ok(value.into()),
    }
);
derive_try_from_chain!(
    char,
    {
        CoreValue::Text(Text(value)) if value.len() == 1 => Ok(value.chars().next().unwrap()),
    }
);

derive_try_from_chain!(
    String,
    {
        CoreValue::Text(Text(value)) => Ok(value),
    }
);
