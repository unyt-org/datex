use crate::{
    datex_proxy::{
        DatexProxyType, DatexValueContainerProxy,
        DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, DatexValueProxyInfallibleSerialize,
        ToDatexNativeValueContainer, TryFromDatexValueError,
        TryToDatexValueError, shared::Shared,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::{SharedContainerMutability, SharedContainerOwnership},
    types::{
        r#type::Type,
        type_definition::TypeDefinition,
        type_definition_with_metadata::{
            TypeDefinitionWithMetadata, TypeMetadata,
        },
    },
    values::{
        core_values::native::DatexNative, value_container::ValueContainer,
    },
};

impl<T> DatexValueContainerProxyInfallibleSerialize for Shared<T>
where
    Shared<T>: DatexValueProxyInfallibleSerialize,
    T: DatexNative,
{
    fn boxed_to_value_container(
        self: Box<Self>,
        _context: &mut SharedReferencesCache,
    ) -> ValueContainer {
        // FIXME
        ValueContainer::Shared(self.container)
    }
}

impl<T> DatexValueContainerProxySerialize for Shared<T>
where
    T: DatexNative,
{
    fn try_boxed_to_value_container(
        self: Box<Self>,
        _context: &mut SharedReferencesCache,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(ValueContainer::Shared(self.container))
    }
}

impl<T> DatexValueContainerProxyDeserialize for Shared<T>
where
    T: DatexNative,
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

impl<T> DatexProxyType for Shared<T>
where
    T: DatexNative + DatexProxyType,
{
    fn datex_type(context: &mut SharedReferencesCache) -> Type {
        Type::Definition(TypeDefinitionWithMetadata::new(
            TypeDefinition::Box(Box::new(T::datex_type(context))),
            TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Owned,
            },
        ))
    }
}

impl<T> DatexValueContainerProxy for Shared<T>
where
    Shared<T>: DatexValueContainerProxy,
    T: DatexNative,
{
}

impl<T: DatexNative> ToDatexNativeValueContainer for Shared<T> {
    fn boxed_to_datex_native_value_container(
        self,
        _cache: &mut SharedReferencesCache,
    ) -> ValueContainer {
        ValueContainer::Shared(self.container)
    }
}
