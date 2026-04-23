use crate::{
    dif::{
        deserialization_context::DeserializationContext,
        pointer_address::PointerAddressWithOwnership,
    },
    shared_values::{
        ReferenceMutability, SharedContainer, SharedContainerOwnership,
    },
};
use alloc::format;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer, de::DeserializeSeed,
};

impl Serialize for SharedContainer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Only serialize the ownership and pointer address
        let ownership = match self.ownership() {
            SharedContainerOwnership::Referenced(
                ReferenceMutability::Immutable,
            ) => "'",
            SharedContainerOwnership::Referenced(
                ReferenceMutability::Mutable,
            ) => "'mut ",
            SharedContainerOwnership::Owned => "",
        };

        format!("{}{}", ownership, self.pointer_address()).serialize(serializer)
    }
}

impl<'de, 'ctx> DeserializeSeed<'de>
    for DeserializationContext<'ctx, SharedContainer>
{
    type Value = SharedContainer;
    fn deserialize<D: Deserializer<'de>>(
        self,
        d: D,
    ) -> Result<SharedContainer, D::Error> {
        let PointerAddressWithOwnership { address, ownership } =
            PointerAddressWithOwnership::deserialize(d)?;
        let reference = self
            .shared_container_cache
            .try_get_shared_container_with_ownership(&address, ownership)
            .map_err(|e| {
                serde::de::Error::custom(format!(
                    "Failed to retrieve shared container from cache: {}",
                    e
                ))
            })?;
        Ok(reference)
    }
}

#[cfg(test)]
mod tests {
    use log::info;

    use super::*;
    use crate::{
        libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
        prelude::*,
        runtime::{
            memory::Memory,
            pointer_address_provider::SelfOwnedPointerAddressProvider,
        },
        shared_values::{
            PointerAddress, SharedContainer, SharedContainerMutability,
            errors::UnexpectedSharedContainerOwnershipError,
        },
        values::{
            core_value::CoreValue, core_values::integer::Integer,
            value_container::ValueContainer,
        },
    };

    #[test]
    fn serialize_shared_container_reference() {
        let memory = Memory::new();
        let integer_container = SharedContainer::Referenced(
            memory
                .get_core_reference(CoreLibTypeId::Base(
                    CoreLibBaseTypeId::Integer,
                ))
                .clone(),
        );
        let serialized = serde_json::to_string(&integer_container).unwrap();
        assert_eq!(serialized, r#""'$030000""#);
    }

    #[test]
    fn serialize_shared_owned_container() {
        let memory = &Memory::new();
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        let owned_container =
            SharedContainer::new_owned_with_inferred_allowed_type(
                ValueContainer::from(42),
                SharedContainerMutability::Mutable,
                address_provider,
                memory,
            );

        let serialized = serde_json::to_string(&owned_container).unwrap();
        assert_eq!(
            serialized,
            format!(r#""{}""#, owned_container.pointer_address().to_string())
        );
    }

    use crate::dif::{
        cache::{
            CacheValueRetrievalError, DIFSharedContainerCache,
            ValueNotFoundInCacheError,
        },
        deserialization_context::DeserializationContext,
    };
    use core::assert_matches;

    fn deserialize_json_string(
        str: impl Into<String>,
        dif_cache: &mut DIFSharedContainerCache,
    ) -> SharedContainer {
        DeserializationContext::<SharedContainer>::new(dif_cache)
            .deserialize(&mut serde_json::Deserializer::from_str(
                str.into().as_str(),
            ))
            .unwrap()
    }

    #[test]
    fn deserialize_core_pointer_address_to_shared_container() {
        let json = r#""'$030000""#; // integer

        let memory = Memory::new();
        let dif_cache = &mut DIFSharedContainerCache::default();

        let integer_container = SharedContainer::Referenced(
            memory
                .get_core_reference(CoreLibTypeId::Base(
                    CoreLibBaseTypeId::Integer,
                ))
                .clone(),
        );
        dif_cache.store_shared_container(integer_container.clone());

        let outer = deserialize_json_string(json, dif_cache);

        assert_eq!(outer, integer_container);
    }

    #[test]
    fn deserialize_memory_pointer_address_to_shared_container() {
        let memory = &mut Memory::new();
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();
        let dif_cache = &mut DIFSharedContainerCache::default();

        let owned_container =
            SharedContainer::new_owned_with_inferred_allowed_type(
                ValueContainer::from(42),
                SharedContainerMutability::Mutable,
                address_provider,
                memory,
            );
        let ptr_address = owned_container.pointer_address();
        let ptr_address_hex = ptr_address.to_string();

        dif_cache.store_shared_container(owned_container);

        let outer_ref = deserialize_json_string(
            format!(r#""'{}""#, ptr_address_hex),
            dif_cache,
        );
        assert_matches!(
            outer_ref,
            SharedContainer::Referenced(reference)
            if reference.reference_mutability() == ReferenceMutability::Immutable &&
                reference.pointer_address() == ptr_address
        );

        let outer_ref_mut = deserialize_json_string(
            format!(r#""'mut{}""#, ptr_address_hex),
            dif_cache,
        );
        assert_matches!(
            outer_ref_mut,
            SharedContainer::Referenced(reference)
            if reference.reference_mutability() == ReferenceMutability::Mutable &&
                reference.pointer_address() == ptr_address
        );

        let outer_owned = deserialize_json_string(
            format!(r#""{}""#, ptr_address_hex),
            dif_cache,
        );
        assert_matches!(
            outer_owned,
            SharedContainer::Owned(owned)
            if PointerAddress::SelfOwned(owned.pointer_address().clone()) == ptr_address
        );

        // should no longer exist in memory as owned container should have been taken from cache
        assert_matches!(
            dif_cache.try_take_owned_shared_container(&ptr_address),
            Err(
                CacheValueRetrievalError::UnexpectedSharedContainerOwnership(
                    UnexpectedSharedContainerOwnershipError {
                        actual: SharedContainerOwnership::Referenced(
                            ReferenceMutability::Mutable
                        ),
                        expected: SharedContainerOwnership::Owned
                    }
                )
            )
        );

        // should no longer exist in memory at all after explicitly removing the shared container from cache
        dif_cache.remove_shared_container(&ptr_address);

        assert_matches!(
            dif_cache.try_take_owned_shared_container(&ptr_address),
            Err(CacheValueRetrievalError::ValueNotFoundInCache(
                ValueNotFoundInCacheError
            ))
        );
    }
}
