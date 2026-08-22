//! Implements [DatexValueProxy] for [Vec<T>] where T: [DatexValueProxy].

use std::any::Any;
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
use crate::ast::expressions::DatexExpressionData;
use crate::ast::spanned::Spanned;
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::core_values::native::DatexNative;

impl<T> DatexValueProxy for Vec<T> where T: DatexValueContainerProxy + 'static {}

impl<T> DatexValueProxyDeserialize for Vec<T>
where
    T: DatexValueContainerProxyDeserialize + 'static,
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

impl<T> DatexValueProxySerialize for Vec<T>
where
    T: DatexValueContainerProxySerialize,
{
    fn try_boxed_to_value(
        self: Box<Self>,
        context: &mut SharedReferencesCache,
    ) -> Result<Value, TryToDatexValueError> {
        let list = self
            .into_iter()
            .map(|v| Box::new(v).try_boxed_to_value_container(context))
            .collect::<Result<List, _>>()?;
        Ok(Value::from(list))
    }
}

impl<T: DatexValueContainerProxyInfallibleSerialize>
    DatexValueProxyInfallibleSerialize for Vec<T>
{
    fn boxed_to_value(
        self: Box<Self>,
        context: &mut SharedReferencesCache,
    ) -> Value {
        Value::from(
            self.into_iter()
                .map(|v| Box::new(v).boxed_to_value_container(context))
                .collect::<Vec<_>>(),
        )
    }
}

impl<T> DatexProxyType for Vec<T>
where
    T: DatexProxyType,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::Collection(CollectionTypeDefinition::List(
                ListCollectionTypeDefinition(Box::new(T::datex_type(memory))),
            ))
            .into(),
        )
    }
}

impl<T> ToDatexExpressionData for Vec<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(
        &self,
    ) -> DatexExpressionData {
        DatexExpressionData::List(self.iter().map(|v| v.to_datex_expression_data().with_default_span()).collect())
    }
}

// TODO: clean up traits
impl<T: DatexNative + DatexProxyType + DatexValueProxy> DatexNative for Vec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn boxed_to_datex_native_value(self: Box<Self>, cache: &mut SharedReferencesCache) -> Value {
        Value::native_boxed(self, cache)
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
        let value: Value = vec.to_value_without_cache();
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
        let vec_type = Vec::<Integer>::datex_type_without_cache();
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
