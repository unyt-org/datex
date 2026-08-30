//! Implements [DatexValueProxy] for [Box<T>] where T: [DatexValueProxy].
#[cfg(feature = "decompiler")]
mod to_datex_expression_data;
mod get_datex_type;
mod datex_native_structural;
mod datex_native;
mod convert_parts;
mod get_core_lib_type_id;
mod value_access;

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        values::value::Value,
    };
    use crate::preludes::derive::SharedReferencesCache;
    use crate::traits::get_datex_type::GetDatexType;
    use crate::values::{
        core_value::CoreValue,
        core_values::{endpoint::Endpoint, integer::Integer},
        value_container::ValueContainer,
    };
    #[test]
    fn boxed_integer() {
        // if impl_datex_direct_via_value_container would be not implemented, for Value it definitely is (user defined types)
        let value: Value = Integer::from(42).into();
        let boxed_integer = Box::new(value);
        let value: Value = Value::native(boxed_integer, &mut SharedReferencesCache::default());
        assert!(matches!(
            value.inner,
            CoreValue::Integer(ref i) if i == &Integer::from(42)
        ));
    }

    #[test]
    fn endpoint_boxed() {
        let endpoint = Endpoint::new("@jonas");
        let boxed_endpoint = Box::new(endpoint.clone());
        let value: Value = Value::native_only_structural(boxed_endpoint);
        assert!(matches!(
            value.inner,
            CoreValue::Endpoint(ref e) if e == &endpoint
        ));
    }

    #[test]
    fn try_from_value_boxed() {
        // endpoint boxed
        let endpoint = Endpoint::new("@jonas");
        let value: Value = Value::native_only_structural(Box::new(endpoint.clone()));
        let boxed_endpoint: Box<Endpoint> = value.try_into_value().unwrap();
        assert_eq!(*boxed_endpoint, endpoint);

        // value boxed
        let value: Value = Integer::from(42).into();
        let boxed_integer: Box<Value> = value.try_into_value().unwrap();
        assert!(matches!(
            *boxed_integer,
            Value { inner: CoreValue::Integer(ref i), .. } if i == &Integer::from(42)
        ));

        // value container boxed
        let value_container: ValueContainer = Integer::from(42).into();
        let boxed_value_container: Box<ValueContainer> = value_container.try_into_value().unwrap();
        assert_eq!(*boxed_value_container, value_container);
    }

    #[test]
    fn datex_type() {
        let boxed_type = Box::<Endpoint>::datex_type(&mut SharedReferencesCache::default());
        let endpoint_type = Endpoint::datex_type(&mut SharedReferencesCache::default());
        assert_eq!(boxed_type, endpoint_type);
    }
}
