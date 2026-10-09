pub mod classification;
mod convert_parts;
mod convert_value;
mod datex_hash;
pub mod datex_native;
mod datex_native_structural;
mod get_core_lib_type_id;
pub mod get_datex_type;
pub mod serde_dif;
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
        types::{
            r#type::Type,
            type_definition::collection::{
                CollectionTypeDefinition,
                type_definition::list::ListCollectionTypeDefinition,
            },
        },
        values::{core_values::list::List, value::Value},
    };

    use crate::{
        libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
        traits::get_datex_type::GetDatexType,
        types::type_definition::TypeDefinition,
        values::{
            core_values::integer::Integer,
            value::value_classification::ValueClassification,
            value_container::ValueContainer,
        },
    };

    #[test]
    fn to_value() {
        let vec = vec![Integer::new(1), Integer::new(2), Integer::new(3)];
        let vec_clone = vec.clone();
        let value: Value = Value::new(vec);
        assert_eq!(value.try_into_value::<Vec<Integer>>(), Ok(vec_clone));
    }

    #[test]
    fn datex_type() {
        let vec_type =
            Vec::<Integer>::datex_type(&mut SharedReferencesCache::default());
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
