//! Implements [DatexValueProxy] for [Option<T>] where T: [DatexValueProxy].
//! As `Option<T>` is a special Rust type that has no direct equivalent in DATEX, it is represented as a union of `null` and `T` in DATEX.
//! As `Some(None)` would be indistinguishable from `None` when serialized (both would be represented as `null`), we use a tagged type representation for
//! `Option<T>` in DATEX, where `None` is represented as a tagged type with the tag "None(null)", and `Some(T)` is represented as a tagged type with the tag "Some" and
//! an inner type of `T`.

mod as_borrowed;
#[cfg(feature = "decompiler")]
mod to_datex_expression_data;
mod value_access;
pub mod get_datex_type;
pub mod convert_parts;
pub mod get_core_lib_type_id;
pub mod datex_native_only_structural;

use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::{
        r#type::Type,
        type_definition::{TypeDefinition, union::UnionTypeDefinition},
    },
    values::{core_values::native::DatexNative, value::Value},
};
use core::any::Any;
use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::get_datex_type::GetDatexType;


// TODO: clean up traits
impl<T: DatexNative> DatexNative
    for Option<T>
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn value_datex_type(&self, cache: &mut SharedReferencesCache) -> Type {
        <Self as GetDatexType>::datex_type(cache)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::{core_values::integer::Integer, value::Value};

    #[test]
    fn to_value() {
        let some_option: Option<Integer> = Some(Integer::new(1));
        let none_option: Option<Integer> = None;

        let some_value: Value = Value::native_only_structural(some_option);
        let none_value: Value = Value::native_only_structural(none_option);

        assert_eq!(
            some_value,
            Value::boxed(Value::native_only_structural(Integer::new(1)))
        );
        assert_eq!(none_value, Value::boxed(Value::null()));
    }

    #[test]
    fn from_value() {
        let some_value = Value::boxed(Value::native_only_structural(Integer::new(1)));
        let some_option: Option<Integer> = some_value.try_into().unwrap();
        assert_eq!(some_option, Some(Integer::new(1)));

        let none_value: Value = Value::boxed(Value::null());
        let none_option: Option<Integer> = none_value.try_into().unwrap();
        assert_eq!(none_option, None);
    }
    #[test]
    fn datex_type() {
        let option_type = Option::<Integer>::datex_type_without_cache();
        option_type.with_collapsed_type_definition(|td| {
            assert!(matches!(td, TypeDefinition::Box(_)));
        });
    }
}
