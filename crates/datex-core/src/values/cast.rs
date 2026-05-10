use std::hash::Hash;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use crate::{
    datex_proxy::{
        DatexProxy, DatexProxyDeserialize, DatexProxyInfallibleSerialize,
        DatexProxySerialize,
    },
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

            impl TryFrom<Value> for $type {
                type Error = ();
                fn try_from(value: Value) -> Result<Self, Self::Error> {
                    value.inner.try_into()
                }
            }

            impl<'a> TryFrom<&'a CoreValue> for &'a $type {
                type Error = ();
                fn try_from(value: &'a CoreValue) -> Result<Self, Self::Error> {
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

        impl DatexProxy for $type {}

        impl DatexProxyInfallibleSerialize for $type {
            fn to_value_container(self) -> ValueContainer {
               ValueContainer::from(self)
            }
        }
        impl DatexProxySerialize for $type {
            fn try_to_value_container(self) -> Result<ValueContainer, ()> {
                Ok(ValueContainer::from(self))
            }
        }
        impl DatexProxyDeserialize for $type {
            fn try_from_value_container(
                value: ValueContainer,
            ) -> Result<Self, ()> {
                value.try_into().map_err(|_| ())
            }
        }
    };
}

macro_rules! impl_datex_direct_via_value_container {
    ($($ty:ty),* $(,)?) => {
        $(
            impl DatexProxy for $ty {}

            impl DatexProxyInfallibleSerialize for $ty {
                fn to_value_container(self) -> ValueContainer {
                   ValueContainer::from(self)
                }
            }
            impl DatexProxySerialize for $ty {
                fn try_to_value_container(self) -> Result<ValueContainer, ()> {
                    Ok(ValueContainer::from(self))
                }
            }
            impl DatexProxyDeserialize for $ty {
                fn try_from_value_container(
                    value: ValueContainer,
                ) -> Result<Self, ()> {
                   value.try_as().ok_or(())
                }
            }
        )*
    };
}

// FIXME deserialize<de> is still implemented for Endpoint, we should change it to DeserializeSeed and make the proxy compatible
impl_datex_direct_via_value_container!(
    Endpoint,
    // Map,
    // List,
    // Range,
    // Type,
    // NominalTypeDefinition,
    // Callable
);
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
    String,
    {
        CoreValue::Text(Text(value)) => Ok(value),
    }
);


impl<T: Serialize + DeserializeOwned> DatexProxy for Option<T> {}

impl<T: DatexProxyInfallibleSerialize> DatexProxyInfallibleSerialize for Option<T> {
    fn to_value_container(self) -> ValueContainer {
        match self {
            None => ValueContainer::from(Value::null()),
            Some(v) => v.to_value_container(),
        }
    }
}

impl<T: Serialize + DeserializeOwned> DatexProxy for Vec<T> {}

impl<T: DatexProxyInfallibleSerialize> DatexProxyInfallibleSerialize for Vec<T> {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::from(self.into_iter().map(|v| v.to_value_container()).collect::<Vec<_>>())
    }
}

impl<K: DatexProxy + Serialize + DeserializeOwned + Eq + Hash, V: DatexProxy + Serialize + DeserializeOwned> DatexProxy for HashMap<K, V> {}

impl<K: DatexProxyInfallibleSerialize, V: DatexProxyInfallibleSerialize> DatexProxyInfallibleSerialize for HashMap<K, V> {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::from(
            self.into_iter().map(|(k, v)| (k.to_value_container(), v.to_value_container())).collect::<Map>()
        )
    }
}



// FIXME TBD

// derive_try_from_chain!(
//     usize,
//     {
//        CoreValue::TypedInteger(TypedInteger::U64(value)) => Ok(value as usize),
//     }
// );
// derive_try_from_chain!(
//     isize,
//     {
//        CoreValue::TypedInteger(TypedInteger::I64(value)) => Ok(value as isize),
//     }
// );
// derive_try_from_chain!(
//     char,
//     {
//         CoreValue::Text(Text(value)) if value.len() == 1 => Ok(value.chars().next().unwrap()),
//     }
// );
