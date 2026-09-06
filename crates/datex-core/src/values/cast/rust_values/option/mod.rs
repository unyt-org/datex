pub mod classification;
pub mod convert_parts;
mod datex_hash;
pub mod datex_native;
pub mod datex_native_structural;
pub mod get_core_lib_type_id;
pub mod get_datex_type;
#[cfg(feature = "ast")]
mod to_datex_expression_data;
mod to_instructions;
mod try_from_core_value;
mod value_access;

#[cfg(test)]
mod tests {
    use crate::{
        preludes::derive::SharedReferencesCache,
        traits::get_datex_type::GetDatexType,
        types::type_definition::{TypeDefinition, union::UnionTypeDefinition},
        values::{core_values::integer::Integer, value::Value},
    };

    #[test]
    fn to_value() {
        let some_option: Option<Integer> = Some(Integer::new(1));
        let none_option: Option<Integer> = None;

        let some_value: Value = Value::native_structural(some_option);
        let none_value: Value = Value::native_structural(none_option);

        assert_eq!(
            some_value,
            Value::boxed(Value::native_structural(Integer::new(1)))
        );
        assert_eq!(none_value, Value::boxed(Value::null()));
    }

    #[test]
    fn from_value() {
        let some_value =
            Value::boxed(Value::native_structural(Integer::new(1)));
        let some_option =
            some_value.try_into_value::<Option<Integer>>().unwrap();
        assert_eq!(some_option, Some(Integer::new(1)));

        let none_value: Value = Value::boxed(Value::null());
        let none_option =
            none_value.try_into_value::<Option<Integer>>().unwrap();
        assert_eq!(none_option, None);
    }
    #[test]
    fn datex_type() {
        let option_type = Option::<Integer>::datex_type(
            &mut SharedReferencesCache::default(),
        );
        option_type.with_collapsed_type_definition(|td| {
            assert!(matches!(td, TypeDefinition::Box(_)));
        });
    }
}
