//! This module defines the [CoreValue] enum, which represents the fundamental value types in DATEX.
//! Each variant of CoreValue holds the underlying value of a specific type, such as [Integer], [Text], or [List].
//! CoreValues can be converted to and from native Rust types.

use crate::{
    libs::core::type_id::{
        CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId,
    },
    prelude::*,
    types::nominal_type_definition::NominalTypeDefinition,
};
use datex_macros_internal::FromCoreValue;
pub mod serde_dif;
use crate::{
    types::r#type::Type,
    values::{
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
        value_container::ValueContainer,
    },
};
use core::fmt::{Display, Formatter};
pub mod ops;

#[derive(Clone, Debug, PartialEq, Eq, Hash, FromCoreValue)]
pub enum CoreValue {
    Null,
    Boolean(Boolean),
    Integer(Integer),
    TypedInteger(TypedInteger),
    Decimal(Decimal),
    TypedDecimal(TypedDecimal),
    Text(Text),
    Endpoint(Endpoint),
    List(List),
    Map(Map),
    Type(Type),
    NominalTypeDefinition(NominalTypeDefinition),
    Callable(Callable),
    Range(Range),
}
mod child_iterator;
pub mod datex_proxy;
pub mod equality;

impl From<&str> for CoreValue {
    fn from(value: &str) -> Self {
        CoreValue::Text(value.into())
    }
}
impl From<String> for CoreValue {
    fn from(value: String) -> Self {
        CoreValue::Text(Text(value))
    }
}

impl<T> From<Vec<T>> for CoreValue
where
    T: Into<ValueContainer>,
{
    fn from(vec: Vec<T>) -> Self {
        CoreValue::List(vec.into())
    }
}

impl<T> FromIterator<T> for CoreValue
where
    T: Into<ValueContainer>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        CoreValue::List(List::new(iter.into_iter().map(Into::into).collect()))
    }
}

impl From<bool> for CoreValue {
    fn from(value: bool) -> Self {
        CoreValue::Boolean(value.into())
    }
}

impl From<i8> for CoreValue {
    fn from(value: i8) -> Self {
        CoreValue::TypedInteger(value.into())
    }
}
impl From<i16> for CoreValue {
    fn from(value: i16) -> Self {
        CoreValue::TypedInteger(value.into())
    }
}
impl From<i32> for CoreValue {
    fn from(value: i32) -> Self {
        CoreValue::TypedInteger(value.into())
    }
}
impl From<i64> for CoreValue {
    fn from(value: i64) -> Self {
        CoreValue::TypedInteger(value.into())
    }
}
impl From<i128> for CoreValue {
    fn from(value: i128) -> Self {
        CoreValue::TypedInteger(value.into())
    }
}

impl From<u8> for CoreValue {
    fn from(value: u8) -> Self {
        CoreValue::TypedInteger(value.into())
    }
}
impl From<u16> for CoreValue {
    fn from(value: u16) -> Self {
        CoreValue::TypedInteger(value.into())
    }
}
impl From<u32> for CoreValue {
    fn from(value: u32) -> Self {
        CoreValue::TypedInteger(value.into())
    }
}
impl From<u64> for CoreValue {
    fn from(value: u64) -> Self {
        CoreValue::TypedInteger(value.into())
    }
}
impl From<u128> for CoreValue {
    fn from(value: u128) -> Self {
        CoreValue::TypedInteger(value.into())
    }
}

impl From<f32> for CoreValue {
    fn from(value: f32) -> Self {
        CoreValue::TypedDecimal(value.into())
    }
}
impl From<f64> for CoreValue {
    fn from(value: f64) -> Self {
        CoreValue::TypedDecimal(value.into())
    }
}

impl From<usize> for CoreValue {
    fn from(value: usize) -> Self {
        #[cfg(target_pointer_width = "64")]
        {
            CoreValue::TypedInteger((value as u64).into())
        }
        #[cfg(target_pointer_width = "32")]
        {
            CoreValue::TypedInteger((value as u32).into())
        }
    }
}

impl From<isize> for CoreValue {
    fn from(value: isize) -> Self {
        #[cfg(target_pointer_width = "64")]
        {
            CoreValue::TypedInteger((value as i64).into())
        }
        #[cfg(target_pointer_width = "32")]
        {
            CoreValue::TypedInteger((value as i32).into())
        }
    }
}

impl From<&CoreValue> for CoreLibTypeId {
    fn from(value: &CoreValue) -> Self {
        match value {
            CoreValue::Map(_) => CoreLibTypeId::Base(CoreLibBaseTypeId::Map),
            CoreValue::List(_) => CoreLibTypeId::Base(CoreLibBaseTypeId::List),
            CoreValue::Text(_) => CoreLibTypeId::Base(CoreLibBaseTypeId::Text),
            CoreValue::Boolean(_) => {
                CoreLibTypeId::Base(CoreLibBaseTypeId::Boolean)
            }
            CoreValue::TypedInteger(i) => CoreLibTypeId::Variant(
                CoreLibVariantTypeId::Integer(i.variant()),
            ),
            CoreValue::TypedDecimal(d) => CoreLibTypeId::Variant(
                CoreLibVariantTypeId::Decimal(d.variant()),
            ),
            CoreValue::Integer(_) => {
                CoreLibTypeId::Base(CoreLibBaseTypeId::Integer)
            }
            CoreValue::Decimal(_) => {
                CoreLibTypeId::Base(CoreLibBaseTypeId::Decimal)
            }
            CoreValue::Endpoint(_) => {
                CoreLibTypeId::Base(CoreLibBaseTypeId::Endpoint)
            }
            CoreValue::Null => CoreLibTypeId::Base(CoreLibBaseTypeId::Null),
            CoreValue::Type(_) => CoreLibTypeId::Base(CoreLibBaseTypeId::Type),
            CoreValue::Callable(_) => {
                CoreLibTypeId::Base(CoreLibBaseTypeId::Callable)
            }
            CoreValue::Range(_) => {
                CoreLibTypeId::Base(CoreLibBaseTypeId::Range)
            }
            CoreValue::NominalTypeDefinition(_nominal_type) => {
                CoreLibTypeId::Base(CoreLibBaseTypeId::Never) // TODO: what is the type of nominal type? do we even need to handle this?
            }
        }
    }
}

impl CoreValue {
    pub fn new<T>(value: T) -> CoreValue
    where
        CoreValue: From<T>,
    {
        value.into()
    }

    /// Check if the CoreValue is a combined value type (List, Map)
    /// that contains inner ValueContainers.
    pub fn is_collection_value(&self) -> bool {
        core::matches!(self, CoreValue::List(_) | CoreValue::Map(_))
    }

    /// Get the default type of the CoreValue type definition.
    /// This method uses the CoreLibPointerId to retrieve the corresponding
    /// type reference from the core library.
    /// For example, a CoreValue::TypedInteger(i32) will return the type ref integer/i32
    pub fn default_core_type(&self) -> CoreLibTypeId {
        CoreLibTypeId::from(self)
    }

    /// Tries to get a borrow of the current value as the specified type.
    /// Does not perform any type conversion.
    pub fn try_as<'a, T: 'a>(&'a self) -> Option<&'a T>
    where
        &'a T: TryFrom<&'a CoreValue>,
    {
        <&T>::try_from(self).ok()
    }

    pub fn try_as_mut<'a, T: 'a>(&'a mut self) -> Option<&'a mut T>
    where
        &'a mut T: TryFrom<&'a mut CoreValue>,
    {
        <&mut T>::try_from(self).ok()
    }

    /// Tries to convert the current value into the specific specified type.
    /// Does not perform any type conversion.
    pub fn try_into_value<T>(self) -> Option<T>
    where
        T: TryFrom<CoreValue>,
    {
        T::try_from(self).ok()
    }

    /// Casts the value to a [Text] value
    /// Note: in contrast to [try_cast_to], [Text] values are not wrapped in quotation marks.
    pub fn cast_to_text(&self) -> Text {
        match self {
            CoreValue::Text(text) => text.clone(),
            _ => Text(self.to_string()),
        }
    }

    pub fn cast_to_bool(&self) -> Option<Boolean> {
        match self {
            CoreValue::Text(text) => Some(Boolean(!text.0.is_empty())),
            CoreValue::Boolean(bool) => Some(bool.clone()),
            CoreValue::TypedInteger(int) => Some(Boolean(int.as_i128()? != 0)),
            CoreValue::Null => Some(Boolean(false)),
            _ => None,
        }
    }

    pub fn cast_to_decimal(&self) -> Option<Decimal> {
        match self {
            CoreValue::Text(text) => {
                text.to_string().parse::<f64>().ok().map(Decimal::from)
            }
            CoreValue::TypedInteger(int) => {
                Some(Decimal::from(int.as_i128()? as f64))
            }
            CoreValue::TypedDecimal(decimal) => {
                Some(Decimal::from(decimal.clone()))
            }
            CoreValue::Integer(int) => {
                Some(Decimal::from(int.as_i128()? as f64))
            }
            CoreValue::Decimal(decimal) => Some(decimal.clone()),
            _ => None,
        }
    }

    pub fn cast_to_typed_decimal(
        &self,
        variant: DecimalTypeVariant,
    ) -> Option<TypedDecimal> {
        match self {
            CoreValue::Text(text) => TypedDecimal::try_from_string_and_variant(
                text.as_str(),
                variant,
            )
            .ok(),
            CoreValue::TypedInteger(int) => Some(
                TypedDecimal::try_from_string_and_variant(
                    &int.to_string(),
                    variant,
                )
                .ok()?,
            ),
            CoreValue::TypedDecimal(decimal) => Some(
                TypedDecimal::try_from_string_and_variant(
                    &decimal.to_string(),
                    variant,
                )
                .ok()?,
            ),
            CoreValue::Integer(int) => Some(
                TypedDecimal::try_from_string_and_variant(
                    &int.to_string(),
                    variant,
                )
                .ok()?,
            ),
            CoreValue::Decimal(decimal) => Some(
                TypedDecimal::try_from_string_and_variant(
                    &decimal.to_string(),
                    variant,
                )
                .ok()?,
            ),
            _ => None,
        }
    }

    // FIXME #314 discuss here - shall we fit the integer in the minimum viable type?
    pub fn _cast_to_integer_internal(&self) -> Option<TypedInteger> {
        match self {
            CoreValue::Text(text) => {
                Integer::try_from_string(&text.to_string())
                    .map(|x| Some(x.to_smallest_fitting()))
                    .unwrap_or(None)
            }
            CoreValue::TypedInteger(int) => {
                Some(int.to_smallest_fitting().clone())
            }
            CoreValue::Integer(int) => {
                Some(TypedInteger::IBig(int.clone()).to_smallest_fitting())
            }
            CoreValue::Decimal(decimal) => Some(
                TypedInteger::from(decimal.into_f64() as i128)
                    .to_smallest_fitting(),
            ),
            CoreValue::TypedDecimal(decimal) => Some(
                TypedInteger::from(decimal.as_f64() as i64)
                    .to_smallest_fitting(),
            ),
            _ => None,
        }
    }

    // TODO #315 improve conversion logic
    pub fn cast_to_integer(&self) -> Option<Integer> {
        match self {
            CoreValue::Text(text) => {
                Integer::try_from_string(&text.to_string()).ok()
            }
            CoreValue::TypedInteger(int) => Some(int.as_integer()),
            CoreValue::Integer(int) => Some(int.clone()),
            CoreValue::Decimal(decimal) => {
                // FIXME #316 currently bad as f64 can be infinity or nan
                // convert decimal directly to integer into_f64 is wrong here
                Some(Integer::from(decimal.into_f64() as i128))
            }
            CoreValue::TypedDecimal(decimal) => {
                decimal.as_integer().map(Integer::from)
            }
            _ => None,
        }
    }

    pub fn cast_to_typed_integer(
        &self,
        variant: IntegerTypeVariant,
    ) -> Option<TypedInteger> {
        match self {
            CoreValue::Text(text) => TypedInteger::try_from_string_and_variant(
                text.as_str(),
                variant,
            )
            .ok(),
            CoreValue::TypedInteger(int) => {
                TypedInteger::try_from_string_and_variant(
                    &int.to_string(),
                    variant,
                )
                .ok()
            }
            CoreValue::Integer(int) => {
                TypedInteger::try_from_string_and_variant(
                    int.to_string().as_str(),
                    variant,
                )
                .ok()
            }
            CoreValue::Decimal(decimal) => {
                Some(TypedInteger::from(decimal.into_f64() as i128))
            }
            CoreValue::TypedDecimal(decimal) => {
                decimal.as_integer().map(TypedInteger::from)
            }
            _ => None,
        }
    }

    pub fn cast_to_endpoint(&self) -> Option<Endpoint> {
        match self {
            CoreValue::Text(text) => Endpoint::try_from(text.as_str()).ok(),
            CoreValue::Endpoint(endpoint) => Some(endpoint.clone()),
            _ => None,
        }
    }
}

impl Display for CoreValue {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        match self {
            CoreValue::Type(ty) => write!(f, "{ty}"),
            CoreValue::Boolean(bool) => write!(f, "{bool}"),
            CoreValue::TypedInteger(int) => write!(f, "{int}"),
            CoreValue::TypedDecimal(decimal) => write!(f, "{decimal}"),
            CoreValue::Text(text) => write!(f, "{text}"),
            CoreValue::Null => write!(f, "null"),
            CoreValue::Endpoint(endpoint) => write!(f, "{endpoint}"),
            CoreValue::Map(map) => write!(f, "{map}"),
            CoreValue::Range(range) => {
                write!(f, "{}..{}", range.start, range.end)
            }
            CoreValue::Integer(integer) => write!(f, "{integer}"),
            CoreValue::Decimal(decimal) => write!(f, "{decimal}"),
            CoreValue::List(list) => write!(f, "{list}"),
            CoreValue::Callable(_callable) => write!(f, "[[ callable ]]"),
            CoreValue::NominalTypeDefinition(container) => {
                write!(f, "{container}")
            }
        }
    }
}

#[cfg(test)]
/// This module contains tests for the CoreValue struct.
/// Each CoreValue is a representation of an underlying native value.
/// The tests cover addition, casting, and type conversions.
mod tests {
    use log::{debug, info};

    use super::*;

    #[test]
    fn type_construct() {
        let a = CoreValue::from(42i32);
        assert_eq!(a.default_core_type().to_string(), "integer/i32");
    }

    #[test]
    fn addition() {
        let a = CoreValue::from(42i32);
        let b = CoreValue::from(11i32);

        let a_plus_b = (a.clone() + b.clone()).unwrap();
        assert_eq!(a_plus_b.clone(), CoreValue::from(53));
        info!("{} + {} = {}", a.clone(), b.clone(), a_plus_b.clone());
    }

    #[test]
    fn endpoint() {
        let endpoint: Endpoint =
            CoreValue::from("@test").cast_to_endpoint().unwrap();
        debug!("Endpoint: {endpoint}");
        assert_eq!(endpoint.to_string(), "@test");
    }

    #[test]
    pub fn range_from_core() {
        assert_eq!(
            CoreValue::from(Range {
                start: Box::new(Integer::from(11).into()),
                end: Box::new(Integer::from(13).into())
            })
            .to_string(),
            "11..13"
        );
    }
}
