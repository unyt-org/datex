//! Implements [DatexValueProxy] for [Box<T>] where T: [DatexValueProxy].
#[cfg(feature = "decompiler")]
mod to_datex_expression_data;
mod get_datex_type;
mod datex_native_structural;
mod datex_native;
mod convert_parts;
mod get_core_lib_type_id;
mod value_access;
pub mod classification;
mod try_from_core_value;

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
        let value: Value = Value::native_structural(boxed_endpoint);
        assert!(matches!(
            value.inner,
            CoreValue::Endpoint(ref e) if e == &endpoint
        ));
    }
    
    #[test]
    fn datex_type() {
        let boxed_type = Box::<Endpoint>::datex_type(&mut SharedReferencesCache::default());
        let endpoint_type = Endpoint::datex_type(&mut SharedReferencesCache::default());
        assert_eq!(boxed_type, endpoint_type);
    }
}
