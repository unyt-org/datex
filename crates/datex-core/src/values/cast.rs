use num::ToPrimitive;

use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibVariantTypeId},
    prelude::*,
    shared_values::errors::KeyNotFoundError,
    types::{
        entities::entity_type_definition::EntityTypeDefinition,
        r#type::Type,
        type_definition::collection::{
            CollectionTypeDefinition,
            type_definition::{
                list::ListCollectionTypeDefinition,
                map::MapCollectionTypeDefinition,
            },
        },
    },
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

use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::{TypeDefinition, union::UnionTypeDefinition},
    values::core_values::{
        decimal::typed_decimal::DecimalTypeVariant,
        integer::typed_integer::IntegerTypeVariant,
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

            impl<'a> TryFrom<&'a mut CoreValue> for &'a mut $type {
                type Error = TryFromDatexValueError;
                fn try_from(value: &'a mut CoreValue) -> Result<Self, Self::Error> {
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
    EntityTypeDefinition => EntityTypeDefinition,
    Range               => Range,
    Callable            => Callable,
}

macro_rules! derive_try_from_chain {
    ($type:ty, $dx_type:expr, {$($core_match:tt)*}) => {
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

        impl DatexProxyTypes for $type {
            fn datex_type(_memory: &mut SharedReferencesCache) -> Type {
                Type::Definition(TypeDefinition::CoreType($dx_type.into()).into())
            }
        }
    };
}

macro_rules! impl_datex_direct_via_value_container {
    ($type:ty, $dx_type:expr) => {
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
               value.try_into().map_err(|_| TryFromDatexValueError(format!("Cannot cast ValueContainer to {}, expected ValueContainer::Local with inner type {}", stringify!($type), stringify!($type))))
            }
        }

        impl DatexProxyTypes for $type {
            fn datex_type(_memory: &mut SharedReferencesCache) -> Type {
                Type::Definition(TypeDefinition::CoreType($dx_type.into()).into())
            }
        }
    };
}

impl_datex_direct_via_value_container!(Endpoint, CoreLibBaseTypeId::Endpoint);
impl_datex_direct_via_value_container!(Map, CoreLibBaseTypeId::Map);
impl_datex_direct_via_value_container!(List, CoreLibBaseTypeId::List);
impl_datex_direct_via_value_container!(Range, CoreLibBaseTypeId::Range);
impl_datex_direct_via_value_container!(Type, CoreLibBaseTypeId::Type);
impl_datex_direct_via_value_container!(
    EntityTypeDefinition,
    CoreLibBaseTypeId::Any
);
impl_datex_direct_via_value_container!(Callable, CoreLibBaseTypeId::Callable);

derive_try_from_chain!(
    bool,
    CoreLibBaseTypeId::Boolean,
    {
        CoreValue::Boolean(Boolean(value)) => Ok(value),
    }
);

derive_try_from_chain!(
    u8,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U8),
    {
        CoreValue::TypedInteger(TypedInteger::U8(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_u8().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u8, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_u8().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u8, value is not an integer", value))),
    }
);
derive_try_from_chain!(
    u16,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U16),
    {
        CoreValue::TypedInteger(TypedInteger::U16(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_u16().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u16, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_u16().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u16, value is not an integer", value))),
    }
);
derive_try_from_chain!(
    u32,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U32),
    {
        CoreValue::TypedInteger(TypedInteger::U32(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_u32().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u32, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_u32().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u32, value is not an integer", value))),
    }
);
derive_try_from_chain!(
    u64,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U64),
    {
        CoreValue::TypedInteger(TypedInteger::U64(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_u64().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u64, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_u64().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u64, value is not an integer", value))),
    }
);

// usize depending on platform
#[cfg(target_pointer_width = "32")]
derive_try_from_chain!(
    usize,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U32),
    {
        CoreValue::TypedInteger(TypedInteger::U32(value)) => Ok(value as usize),
        CoreValue::TypedInteger(value) => value.to_u32().map(|v| v as usize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to usize, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_u32().map(|v| v as usize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to usize, value is not an integer", value))),
    }
);
#[cfg(target_pointer_width = "64")]
derive_try_from_chain!(
    usize,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U64),
    {
        CoreValue::TypedInteger(TypedInteger::U64(value)) => Ok(value as usize),
        CoreValue::TypedInteger(value) => value.to_u64().map(|v| v as usize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to usize, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_u64().map(|v| v as usize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to usize, value is not an integer", value))),
    }
);

derive_try_from_chain!(
    i8,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::I8),
    {
        CoreValue::TypedInteger(TypedInteger::I8(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_i8().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i8, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_i8().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i8, value is not an integer", value))),
    }
);
derive_try_from_chain!(
    i16,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::I16),
    {
        CoreValue::TypedInteger(TypedInteger::I16(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_i16().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i16, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_i16().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i16, value is not an integer", value))),
    }
);
derive_try_from_chain!(
    i32,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::I32),
    {
        CoreValue::TypedInteger(TypedInteger::I32(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_i32().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i32, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_i32().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i32, value is not an integer", value))),
    }
);
derive_try_from_chain!(
    i64,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::I64),
    {
        CoreValue::TypedInteger(TypedInteger::I64(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_i64().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i64, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_i64().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i64, value is not an integer", value))),
    }
);

// isize depending on platform
#[cfg(target_pointer_width = "32")]
derive_try_from_chain!(
    isize,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::I32),
    {
        CoreValue::TypedInteger(TypedInteger::I32(value)) => Ok(value as isize),
        CoreValue::TypedInteger(value) => value.to_i32().map(|v| v as isize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to isize, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_i32().map(|v| v as isize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to isize, value is not an integer", value))),
    }
);
#[cfg(target_pointer_width = "64")]
derive_try_from_chain!(
    isize,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::I64),
    {
        CoreValue::TypedInteger(TypedInteger::I64(value)) => Ok(value as isize),
        CoreValue::TypedInteger(value) => value.to_i64().map(|v| v as isize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to isize, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_i64().map(|v| v as isize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to isize, value is not an integer", value))),
    }
);

derive_try_from_chain!(
    f32,
    CoreLibVariantTypeId::Decimal(DecimalTypeVariant::F32),
    {
        CoreValue::TypedDecimal(TypedDecimal::F32(value)) => Ok(value.into()),
        CoreValue::TypedInteger(value) => value.to_f32().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to f32, value is not a decimal", value))),
        CoreValue::TypedDecimal(value) => value.to_f32().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to f32, value is not a decimal", value))),
    }
);
derive_try_from_chain!(
    f64,
    CoreLibVariantTypeId::Decimal(DecimalTypeVariant::F64),
    {
        CoreValue::TypedDecimal(TypedDecimal::F64(value)) => Ok(value.into()),
        CoreValue::TypedInteger(value) => value.to_f64().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to f64, value is not a decimal", value))),
        CoreValue::TypedDecimal(value) => value.to_f64().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to f64, value is not a decimal", value))),
    }
);
derive_try_from_chain!(
    String,
    CoreLibBaseTypeId::Text,
    {
        CoreValue::Text(Text(value)) => Ok(value),
    }
);
// &str
impl<'a> TryFrom<&'a CoreValue> for &'a str {
    type Error = TryFromDatexValueError;

    fn try_from(value: &'a CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Text(Text(value)) => Ok(value.as_str()),
            _ => Err(TryFromDatexValueError(
                "Cannot cast CoreValue to &str".into(),
            )),
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a str {
    type Error = TryFromDatexValueError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        (&value.inner).try_into()
    }
}
impl<'a> TryFrom<&'a ValueContainer> for &'a str {
    type Error = TryFromDatexValueError;

    fn try_from(value: &'a ValueContainer) -> Result<Self, Self::Error> {
        match value {
            ValueContainer::Local(value) => value.try_into(),
            _ => Err(TryFromDatexValueError(
                "Cannot cast shared ValueContainer to &str".into(),
            )),
        }
    }
}
impl DatexProxyTypes for &str {
    fn datex_type(_memory: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Text.into()).into(),
        )
    }
}

impl DatexProxyTypes for str {
    fn datex_type(_memory: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Text.into()).into(),
        )
    }
}

// -------- Box<T> -------
impl<T> DatexValueProxy for Box<T> where T: DatexValueProxy {}

impl<T> DatexValueProxySerialize for Box<T>
where
    T: DatexValueProxySerialize,
{
    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
        (*self).try_to_value()
    }
}

impl<T> DatexValueProxyInfallibleSerialize for Box<T>
where
    T: DatexValueProxyInfallibleSerialize,
{
    fn to_value(self) -> Value {
        (*self).to_value()
    }
}

impl<T> DatexValueProxyDeserialize for Box<T>
where
    T: DatexValueProxyDeserialize,
{
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        Ok(Box::new(T::try_from_value(value)?))
    }
}

impl<T> DatexProxyTypes for Box<T>
where
    T: DatexProxyTypes,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        T::datex_type(memory)
    }
}

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
    fn try_from_map_property(
        value: Result<Value, KeyNotFoundError>,
    ) -> Result<Self, TryFromDatexValueError> {
        match value {
            Ok(value) => Self::try_from_value(value),
            Err(_) => Ok(None),
        }
    }
}

impl<T> DatexProxyTypes for Option<T>
where
    T: DatexProxyTypes,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        // null | T
        Type::Definition(
            TypeDefinition::Union(UnionTypeDefinition(vec![
                Type::Definition(
                    TypeDefinition::CoreType(CoreLibBaseTypeId::Null.into())
                        .into(),
                ),
                T::datex_type(memory),
            ]))
            .into(),
        )
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

impl<T> DatexProxyTypes for Vec<T>
where
    T: DatexProxyTypes,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::Collection(CollectionTypeDefinition::List(
                ListCollectionTypeDefinition(Box::new(T::datex_type(memory))),
            ))
            .into(),
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

impl<K, V> DatexProxyTypes for HashMap<K, V>
where
    K: DatexProxyTypes + Eq + Hash,
    V: DatexProxyTypes,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::Collection(CollectionTypeDefinition::Map(
                MapCollectionTypeDefinition::new(
                    K::datex_type(memory),
                    V::datex_type(memory),
                ),
            ))
            .into(),
        )
    }
}
