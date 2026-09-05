//! Implements [DatexValueProxy] for [Box<T>] where T: [DatexValueProxy].
pub mod classification;
mod convert_parts;
pub mod datex_hash;
mod datex_native;
mod datex_native_structural;
mod get_core_lib_type_id;
mod get_datex_type;
#[cfg(feature = "ast")]
mod to_datex_expression_data;
mod to_instructions;
mod try_from_core_value;
mod value_access;

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        preludes::derive::SharedReferencesCache,
        traits::get_datex_type::GetDatexType,
        values::{
            core_value::CoreValue,
            core_values::{endpoint::Endpoint, integer::Integer},
            value::Value,
            value_container::ValueContainer,
        },
    };
    // FIXME: how to handle Box<Value>
    // #[test]
    // fn boxed_integer() {
    //     // if impl_datex_direct_via_value_container would be not implemented, for Value it definitely is (user defined types)
    //     let value: Value = Integer::from(42).into();
    //     let boxed_integer = Box::new(value);
    //     let value: Value = Value::native(boxed_integer, &mut SharedReferencesCache::default());
    //     assert!(matches!(
    //         value.inner,
    //         CoreValue::Integer(ref i) if i == &Integer::from(42)
    //     ));
    // }

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
        let boxed_type =
            Box::<Endpoint>::datex_type(&mut SharedReferencesCache::default());
        let endpoint_type =
            Endpoint::datex_type(&mut SharedReferencesCache::default());
        assert_eq!(boxed_type, endpoint_type);
    }
}
