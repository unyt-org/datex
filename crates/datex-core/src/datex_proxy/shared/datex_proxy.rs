use crate::{
    datex_proxy::{
        DatexProxyTypes, DatexValueContainerProxy,
        DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize,
        DatexValueProxyInfallibleSerialize,
        DatexValueProxySerialize, TryFromDatexValueError, TryToDatexValueError,
        shared::Shared,
    },
    prelude::*,
    shared_values::{SharedContainerMutability, SharedContainerOwnership},
    types::{
        r#type::Type,
        type_definition::TypeDefinition,
        type_definition_with_metadata::{
            TypeDefinitionWithMetadata, TypeMetadata,
        },
    },
    values::value_container::ValueContainer,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl<T, C> DatexValueContainerProxyInfallibleSerialize<C> for Shared<T, C>
where
    Shared<T, C>: DatexValueProxyInfallibleSerialize<C>,
    T: DatexValueContainerProxySerialize<C> + DatexValueContainerProxyDeserialize,
{
    fn to_value_container(self, _context: &mut C) -> ValueContainer {
        // FIXME
        ValueContainer::Shared(self.container)
    }
}

// Note: concrete impls for SharedReferencesCache and () are required, generic context does not work

impl<T> DatexValueContainerProxySerialize<SharedReferencesCache>
for Shared<T, SharedReferencesCache>
where
    T: DatexValueContainerProxySerialize<SharedReferencesCache>
    + DatexValueContainerProxyDeserialize,
{
    fn try_to_value_container(
        self,
        _context: &mut SharedReferencesCache,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(ValueContainer::Shared(self.container))
    }
}
impl<T> DatexValueContainerProxySerialize<()>
for Shared<T, ()>
where
    T: DatexValueContainerProxySerialize<()>
    + DatexValueContainerProxyDeserialize,
{
    fn try_to_value_container(
        self,
        _context: &mut (),
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(ValueContainer::Shared(self.container))
    }
}

impl<T, C> DatexValueContainerProxyDeserialize for Shared<T, C>
where
    T: DatexValueContainerProxySerialize<C> + DatexValueContainerProxyDeserialize,
{
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError> {
        match value {
            ValueContainer::Shared(container) => Shared::try_from(container),
            _ => Err(TryFromDatexValueError(
                "Expected ValueContainer::Shared, ValueContainer::Local"
                    .to_string(),
            )),
        }
    }
}

impl<T, C> DatexProxyTypes<C> for Shared<T, C>
where
    T: DatexValueContainerProxySerialize<C> + DatexValueContainerProxyDeserialize + DatexProxyTypes<C>,
{
    fn datex_type(context: &mut C) -> Type {
        Type::Definition(TypeDefinitionWithMetadata::new(
            TypeDefinition::Box(Box::new(T::datex_type(context))),
            TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Owned,
            },
        ))
    }
}

impl<T, C> DatexValueContainerProxy<C> for Shared<T, C>
where
    Shared<T, C>: DatexValueContainerProxy<C>,
    T: DatexValueContainerProxySerialize<C> + DatexValueContainerProxyDeserialize,
{
}
