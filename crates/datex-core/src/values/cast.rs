use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
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
use core::hash::Hash;

macro_rules! impl_try_from_core_value {
    ($($variant:ident => $type:ty),* $(,)?) => {
        $(
            impl TryFrom<Value> for $type {
                type Error = TryFromDatexValueError;
                fn try_from(value: Value) -> Result<Self, Self::Error> {
                    value.inner.try_into()
                }
            }

            impl TryFrom<CoreValue> for $type {
                type Error = TryFromDatexValueError;
                fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
                    match value {
                        CoreValue::$variant(v) => Ok(v),
                        _ => Err(TryFromDatexValueError(format!("Cannot cast CoreValue to {}, expected CoreValue::{}", stringify!($type), stringify!($variant)))),
                    }
                }
            }

            impl<'a> TryFrom<&'a CoreValue> for &'a $type {
                type Error = TryFromDatexValueError;
                fn try_from(value: &'a CoreValue) -> Result<Self, Self::Error> {
                    match value {
                        CoreValue::$variant(v) => Ok(v),
                        _ => Err(TryFromDatexValueError(format!("Cannot cast CoreValue to {}, expected CoreValue::{}", stringify!($type), stringify!($variant)))),
                    }
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
            type Error = TryFromDatexValueError;

            fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
               match value {
                    $($core_match)*
                    _ => Err(TryFromDatexValueError(format!("Cannot cast CoreValue to {}", stringify!($type)))),
               }
            }
        }

        impl TryFrom<Value> for $type {
            type Error = TryFromDatexValueError;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                value.inner.try_into()
            }
        }

        impl TryFrom<ValueContainer> for $type {
            type Error = TryFromDatexValueError;

            fn try_from(value: ValueContainer) -> Result<Self, Self::Error> {
                match value {
                    ValueContainer::Local(value) => value.try_into(),
                    _ => Err(TryFromDatexValueError(format!("Cannot cast ValueContainer to {}, expected ValueContainer::Local", stringify!($type)))),
                }
            }
        }

        impl DatexValueProxy for $type {}

        impl DatexValueProxyInfallibleSerialize for $type {
            fn to_value(self) -> Value {
               Value::from(self)
            }
        }
        impl DatexValueProxySerialize for $type {
            fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
                Ok(Value::from(self))
            }
        }
        impl DatexValueProxyDeserialize for $type {
            fn try_from_value(
                value: Value,
            ) -> Result<Self, TryFromDatexValueError> {
                value.try_into()
            }
        }
    };
}

macro_rules! impl_datex_direct_via_value_container {
    ($($ty:ty),* $(,)?) => {
        $(
            impl DatexValueProxy for $ty {}

            impl DatexValueProxyInfallibleSerialize for $ty {
                fn to_value(self) -> Value {
                   Value::from(self)
                }
            }
            impl DatexValueProxySerialize for $ty {
                fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
                    Ok(Value::from(self))
                }
            }
            impl DatexValueProxyDeserialize for $ty {
                fn try_from_value(
                    value: Value,
                ) -> Result<Self, TryFromDatexValueError> {
                   value.try_into().map_err(|_| TryFromDatexValueError(format!("Cannot cast ValueContainer to {}, expected ValueContainer::Local with inner type {}", stringify!($ty), stringify!($ty))))
                }
            }
        )*
    };
}

impl_datex_direct_via_value_container!(
    Endpoint,
    Map,
    List,
    Range,
    Type,
    NominalTypeDefinition,
    Callable
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

// -------- Option<T> -------
impl<T: DatexValueProxy> DatexValueProxy for Option<T> {}

impl<T: DatexValueProxyInfallibleSerialize> DatexValueProxyInfallibleSerialize
    for Option<T>
{
    fn to_value(self) -> Value {
        match self {
            None => Value::null(),
            Some(v) => v.to_value(),
        }
    }
}

impl<T: DatexValueProxy> DatexValueProxySerialize for Option<T> {
    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
        match self {
            None => Ok(Value::null()),
            Some(v) => v.try_to_value(),
        }
    }
}

impl<T: DatexValueProxy> DatexValueProxyDeserialize for Option<T> {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        if value == Value::null() {
            Ok(None)
        } else {
            Ok(Some(T::try_from_value(value)?))
        }
    }
}

// -------- Vec<T> -------
impl<T: DatexValueContainerProxy> DatexValueProxy for Vec<T> {}

impl<T: DatexValueContainerProxy> DatexValueProxyDeserialize for Vec<T> {
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        match List::try_from(value) {
            Ok(val) => val
                .into_iter()
                .map(|v| T::try_from_value_container(v))
                .collect::<Result<Vec<T>, _>>(),
            Err(e) => Err(e),
        }
    }
}

impl<T: DatexValueContainerProxy> DatexValueProxySerialize for Vec<T> {
    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
        let list = self
            .into_iter()
            .map(|v| v.try_to_value_container())
            .collect::<Result<List, _>>()?;
        Ok(Value::from(list))
    }
}

impl<T: DatexValueContainerProxyInfallibleSerialize>
    DatexValueProxyInfallibleSerialize for Vec<T>
{
    fn to_value(self) -> Value {
        Value::from(
            self.into_iter()
                .map(|v| v.to_value_container())
                .collect::<Vec<_>>(),
        )
    }
}

// -------- HashMap<K, V> -------

impl<K: DatexValueContainerProxy + Eq + Hash, V: DatexValueContainerProxy>
    DatexValueProxy for HashMap<K, V>
{
}

impl<K: DatexValueContainerProxy + Eq + Hash, V: DatexValueContainerProxy>
    DatexValueProxyDeserialize for HashMap<K, V>
{
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        match Map::try_from(value) {
            Ok(map) => map
                .into_iter()
                .map(|(k, v)| {
                    let key = K::try_from_value_container(k.into())?;
                    let value = V::try_from_value_container(v)?;
                    Ok((key, value))
                })
                .collect::<Result<HashMap<K, V>, _>>(),
            Err(e) => Err(e),
        }
    }
}

impl<K: DatexValueContainerProxy + Eq + Hash, V: DatexValueContainerProxy>
    DatexValueProxySerialize for HashMap<K, V>
{
    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
        let map = self
            .into_iter()
            .map(|(k, v)| {
                let key = k.try_to_value_container()?;
                let value = v.try_to_value_container()?;
                Ok((key, value))
            })
            .collect::<Result<Map, _>>()?;
        Ok(Value::from(map))
    }
}

impl<
    K: DatexValueContainerProxyInfallibleSerialize + Eq + Hash,
    V: DatexValueContainerProxyInfallibleSerialize,
> DatexValueProxyInfallibleSerialize for HashMap<K, V>
{
    fn to_value(self) -> Value {
        let map = self
            .into_iter()
            .map(|(k, v)| {
                let key = k.to_value_container();
                let value = v.to_value_container();
                (key, value)
            })
            .collect::<Map>();
        Value::from(map)
    }
}
