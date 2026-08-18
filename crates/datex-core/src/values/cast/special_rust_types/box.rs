//! Implements [DatexValueProxy] for [Box<T>] where T: [DatexValueProxy].

use core::any::Any;
use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
    prelude::*,
    types::r#type::Type,
    values::{value::Value, value_container::ValueContainer},
};

use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl<T, C> DatexValueProxy<C> for Box<T> where T: DatexValueProxy<C> {}

impl<T, C> DatexValueProxySerialize<C> for Box<T>
where
    T: DatexValueProxySerialize<C>,
{
    fn try_to_value(
        self,
        context: &mut C,
    ) -> Result<Value, TryToDatexValueError> {
        (*self).try_to_value(context)
    }
}

impl<T, C> DatexValueProxyInfallibleSerialize<C> for Box<T>
where
    T: DatexValueProxyInfallibleSerialize<C>,
{
    fn to_value(self, context: &mut C) -> Value {
        (*self).to_value(context)
    }
}
// FIXME do we want to allow ValueContainer directly to be boxed, or should DatexValueProxyDeserialize be enought?
// We at least would avoid the Umweg over ValueContainer, but we would be able to fullfill `let boxed_value_container: Result<Box<ValueContainer>, _> = Box::try_from_value_container(value_container)`
// impl<T> DatexValueProxyDeserialize for Box<T>
// where
//     T: DatexValueProxyDeserialize,
// {
//     fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
//         Ok(Box::new(T::try_from_value(value)?))
//     }
// }
impl<T> DatexValueProxyDeserialize for Box<T>
where
    T: DatexValueContainerProxyDeserialize + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        T::try_from_value_container(value.into()).map(Box::new)
    }
}

impl<T, C> DatexProxyTypes<C> for Box<T>
where
    T: DatexProxyTypes<C>,
{
    fn datex_type(memory: &mut C) -> Type {
        T::datex_type(memory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::{
        core_value::CoreValue,
        core_values::{endpoint::Endpoint, integer::Integer},
        value::Value,
        value_container::ValueContainer,
    };
    #[test]
    fn boxed_integer() {
        // if impl_datex_direct_via_value_container would be not implemented, for Value it defintely is (user defined types)
        let value: Value = Integer::from(42).into();
        let boxed_integer = Box::new(value);
        let value: Value = boxed_integer.to_value_without_context();
        assert!(matches!(
            value.inner,
            CoreValue::Integer(ref i) if i == &Integer::from(42)
        ));
    }

    #[test]
    fn endpoint_boxed() {
        let endpoint = Endpoint::new("@jonas");
        let boxed_endpoint = Box::new(endpoint.clone());
        let value: Value = boxed_endpoint.to_value_without_context();
        assert!(matches!(
            value.inner,
            CoreValue::Endpoint(ref e) if e == &endpoint
        ));
    }

    #[test]
    fn try_from_value_boxed() {
        // endpoint boxed
        let endpoint = Endpoint::new("@jonas");
        let value: Value = endpoint.clone().to_value_without_context();
        let boxed_endpoint: Result<Box<Endpoint>, _> =
            Box::try_from_value(value);
        assert!(boxed_endpoint.is_ok());
        assert_eq!(*boxed_endpoint.unwrap(), endpoint);

        // value boxed
        let value: Value = Integer::from(42).into();
        let boxed_integer: Result<Box<Value>, _> = Box::try_from_value(value);
        assert!(boxed_integer.is_ok());
        assert!(matches!(
            *boxed_integer.unwrap(),
            Value { inner: CoreValue::Integer(ref i), .. } if i == &Integer::from(42)
        ));

        // value container boxed
        let value_container: ValueContainer = Integer::from(42).into();
        let boxed_value_container: Result<Box<ValueContainer>, _> =
            Box::try_from_value_container(value_container.clone());
        assert!(boxed_value_container.is_ok());
        assert_eq!(*boxed_value_container.unwrap(), value_container);
    }

    #[test]
    fn datex_type() {
        let boxed_type = Box::<Endpoint>::datex_type_without_context();
        let endpoint_type = Endpoint::datex_type_without_context();
        assert_eq!(boxed_type, endpoint_type);
    }
}
