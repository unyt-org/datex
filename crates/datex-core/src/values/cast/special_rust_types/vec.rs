//! Implements [DatexValueProxy] for [Vec<T>] where T: [DatexValueProxy].
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

use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::TypeDefinition,
};

impl<T, C> DatexValueProxy<C> for Vec<T> where T: DatexValueContainerProxy<C> {}

impl<T> DatexValueProxyDeserialize for Vec<T>
where
    T: DatexValueContainerProxyDeserialize,
{
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        match List::try_from(value) {
            Ok(val) => val
                .into_iter()
                .map(|v| T::try_from_value_container(v))
                .collect::<Result<Vec<T>, _>>(),
            Err(e) => Err(e),
        }
    }
}

impl<T, C> DatexValueProxySerialize<C> for Vec<T>
where
    T: DatexValueContainerProxySerialize<C>,
{
    fn try_to_value(
        self,
        context: &mut C,
    ) -> Result<Value, TryToDatexValueError> {
        let list = self
            .into_iter()
            .map(|v| v.try_to_value_container(context))
            .collect::<Result<List, _>>()?;
        Ok(Value::from(list))
    }
}

impl<T: DatexValueContainerProxyInfallibleSerialize<C>, C>
    DatexValueProxyInfallibleSerialize<C> for Vec<T>
{
    fn to_value(self, context: &mut C) -> Value {
        Value::from(
            self.into_iter()
                .map(|v| v.to_value_container(context))
                .collect::<Vec<_>>(),
        )
    }
}

impl<T, C> DatexProxyTypes<C> for Vec<T>
where
    T: DatexProxyTypes<C>,
{
    fn datex_type(memory: &mut C) -> Type {
        Type::Definition(
            TypeDefinition::Collection(CollectionTypeDefinition::List(
                ListCollectionTypeDefinition(Box::new(T::datex_type(memory))),
            ))
            .into(),
        )
    }
}

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

    #[test]
    fn to_value() {
        let vec = vec![Integer::new(1), Integer::new(2), Integer::new(3)];
        let value: Value = vec.to_value_without_context();
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
        let vec: Vec<Integer> = Vec::try_from_value(value).unwrap();
        assert_eq!(
            vec,
            vec![Integer::new(1), Integer::new(2), Integer::new(3)]
        );
    }

    #[test]
    fn datex_type() {
        let vec_type = Vec::<Integer>::datex_type_without_context();
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
