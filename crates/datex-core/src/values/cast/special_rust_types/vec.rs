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

impl<T: DatexValueContainerProxy> DatexValueProxy for Vec<T> {}

impl<T: DatexValueContainerProxy> DatexValueProxyDeserialize for Vec<T> {
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

impl<T: DatexValueContainerProxy> DatexValueProxySerialize for Vec<T> {
    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
        let list = self
            .into_iter()
            .map(|v| v.try_to_value_container())
            .collect::<Result<List, _>>()?;
        Ok(Value::from(list))
    }
}

impl<T: DatexValueContainerProxyInfallibleSerialize>
    DatexValueProxyInfallibleSerialize for Vec<T>
{
    fn to_value(self) -> Value {
        Value::from(
            self.into_iter()
                .map(|v| v.to_value_container())
                .collect::<Vec<_>>(),
        )
    }
}

impl<T> DatexProxyTypes for Vec<T>
where
    T: DatexProxyTypes,
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
