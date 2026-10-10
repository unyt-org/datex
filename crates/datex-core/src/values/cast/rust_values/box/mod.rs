//! Implements [DatexValueProxy] for [Box<T>] where T: [DatexValueProxy].
mod child_iterator;
pub mod classification;
mod convert_parts;
mod convert_value;
pub mod datex_hash;
mod datex_native;
mod datex_native_structural;
mod get_core_lib_type_id;
mod get_datex_type;
mod serde_dif;
#[cfg(feature = "ast")]
mod to_datex_expression_data;
mod to_instructions;
mod update_handler;
mod value_access;

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        runtime::cache::shared_references_cache::SharedReferencesCache,
        traits::get_datex_type::GetDatexType,
        values::{
            core_value::CoreValue,
            core_values::{
                endpoint::Endpoint, integer::Integer, native::NativeCoreValue,
            },
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
        let value: Value = Value::native(boxed_endpoint);
        assert_eq!(
            value,
            Value::Native(NativeCoreValue::new(Box::new(endpoint.clone())))
        );
        assert_eq!(
            value.try_as::<Endpoint>().expect("Expected Endpoint"),
            &endpoint
        );
        assert_eq!(
            *value.try_as::<Box<Endpoint>>().expect("Expected Endpoint"),
            Box::new(endpoint.clone())
        );
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
