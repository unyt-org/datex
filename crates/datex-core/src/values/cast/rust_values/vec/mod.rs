//! Implements [DatexValueProxy] for [Vec<T>] where T: [DatexValueProxy].

mod as_borrowed;
#[cfg(feature = "decompiler")]
mod to_datex_expression_data;
mod value_access;
pub mod get_datex_type;
mod get_core_lib_type_id;
mod datex_native_only_structural;
mod convert_parts;

use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
    prelude::*,
    types::{
        r#type::Type,
        type_definition::collection::{
            CollectionTypeDefinition,
            type_definition::list::ListCollectionTypeDefinition,
        },
    },
    values::{core_values::list::List, value::Value},
};
use core::any::Any;

use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    types::type_definition::TypeDefinition,
    values::core_values::native::DatexNative,
};


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        libs::core::type_id::CoreLibBaseTypeId,
        values::{
            core_value::CoreValue, core_values::integer::Integer, value::Value,
            value_container::ValueContainer,
        },
    };
    use crate::preludes::derive::SharedReferencesCache;
    use crate::traits::get_datex_type::GetDatexType;

    #[test]
    fn to_value() {
        let vec = vec![Integer::new(1), Integer::new(2), Integer::new(3)];
        let value: Value = Value::native_only_structural(vec);
        assert!(matches!(
            value.inner,
            CoreValue::List(ref l) if l == &List::from(vec![ValueContainer::from(Integer::new(1)), ValueContainer::from(Integer::new(2)), ValueContainer::from(Integer::new(3))])
        ));
    }
    #[test]
    fn try_from_value() {
        let value: Value = List::from(vec![
            ValueContainer::from(Integer::new(1)),
            ValueContainer::from(Integer::new(2)),
            ValueContainer::from(Integer::new(3)),
        ])
        .into();
        let vec: Vec<Integer> = value.try_into_value().unwrap();
        assert_eq!(
            vec,
            vec![Integer::new(1), Integer::new(2), Integer::new(3)]
        );
    }

    #[test]
    fn datex_type() {
        let vec_type = Vec::<Integer>::datex_type(&mut SharedReferencesCache::default());
        vec_type.with_collapsed_type_definition(|td| {
            assert!(matches!(
                td,
                TypeDefinition::Collection(CollectionTypeDefinition::List(
                    ListCollectionTypeDefinition(inner_type)
                )) if **inner_type == Type::Definition(TypeDefinition::CoreType(CoreLibBaseTypeId::Integer.into()).into())
            ));
        });
    }
}
