use crate::{
    dif::{
        pointer_address::PointerAddressWithOwnership,
        serde_context::SerdeContext,
    },
    prelude::*,
    shared_values::{
        ReferenceMutability, SharedContainer, SharedContainerOwnership,
    },
    utils::serde_serialize_seed::SerializeSeed,
};
use alloc::format;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer, de::DeserializeSeed,
};

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, SharedContainer> {
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
impl<'ctx> SerdeContext<'ctx, SharedContainer> {
    pub fn pointer_string(&mut self, value: &SharedContainer) -> String {
        unsafe {
            self.shared_container_cache
                .store_shared_container(value.clone_unsafe());
        }

        let ownership = match value.ownership() {
            SharedContainerOwnership::Referenced(
                ReferenceMutability::Immutable,
            ) => "'",

            SharedContainerOwnership::Referenced(
                ReferenceMutability::Mutable,
            ) => "'mut ",

            SharedContainerOwnership::Owned => "",
        };

        format!("{}{}", ownership, value.pointer_address())
    }
}
impl<'ctx> SerializeSeed for SerdeContext<'ctx, SharedContainer> {
    type Value = SharedContainer;

    /// SAFETY:
    /// The caller of the `serialize` method must either
    /// * guarantee that no direct value (accessible without borrow) is an owned shared value
    ///   (this can be guaranteed by calling clone on the top level value before passing it to [SerializeSeed])
    /// * or guarantee that the value is dropped after calling `serialize`, so that the owned shared value
    ///   is not leaked after serialization.
    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.pointer_string(value).serialize(serializer)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        prelude::*,
        runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
        shared_values::{
            OwnedSharedContainer, PointerAddress, ReferenceMutability,
            SharedContainer, SharedContainerMutability,
            SharedContainerOwnership,
            errors::UnexpectedSharedContainerOwnershipError,
        },
        values::value_container::ValueContainer,
    };

    #[test]
    fn serialize_shared_container_reference() {
        let owned_shared_container =
            OwnedSharedContainer::new_with_inferred_allowed_type(
                ValueContainer::from(42),
                SharedContainerMutability::Immutable,
                &mut SelfOwnedPointerAddressProvider::default(),
            )
            .derive_immutable_reference();
        let address = owned_shared_container.pointer_address();

        let serialized = SerdeContext::<SharedContainer>::new(
            &mut SharedValuesCache::default(),
        )
        .serialize_to_json(&SharedContainer::Referenced(
            owned_shared_container,
        ));
        assert_eq!(serialized, format!(r#""'{}""#, address.to_string()));
    }

    #[test]
    fn serialize_shared_owned_container() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        let owned_container =
            SharedContainer::new_owned_with_inferred_allowed_type(
                ValueContainer::from(42),
                SharedContainerMutability::Mutable,
                address_provider,
            );

        let serialized = SerdeContext::<SharedContainer>::new(
            &mut SharedValuesCache::default(),
        )
        .serialize_to_json(&owned_container);
        assert_eq!(
            serialized,
            format!(r#""{}""#, owned_container.pointer_address().to_string())
        );
    }

    use crate::dif::{
        serde_context::SerdeContext,
    };
    use core::assert_matches;
    use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
    use crate::runtime::cache::shared_values_cache::SharedValuesCache;

    #[test]
    fn deserialize_owned_pointer_address_to_shared_container() {
        let cache = &mut SharedValuesCache::default();

        let owned_shared_container =
            OwnedSharedContainer::new_with_inferred_allowed_type(
                ValueContainer::from(42),
                SharedContainerMutability::Immutable,
                &mut SelfOwnedPointerAddressProvider::default(),
            );
        let pointer_address = owned_shared_container.pointer_address().clone();
        cache.store_shared_container(SharedContainer::Owned(
            owned_shared_container,
        ));

        let outer = SerdeContext::<SharedContainer>::new(cache)
            .try_deserialize_from_json(
                format!(r#""{}""#, pointer_address).as_str(),
            )
            .unwrap();
        if let SharedContainer::Owned(retrieved) = &outer {
            assert_eq!(*retrieved.pointer_address(), pointer_address);
        } else {
            panic!("Expected owned shared container");
        }
    }

    #[test]
    fn deserialize_pointer_address_to_shared_container() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();
        let cache = &mut SharedValuesCache::default();

        let owned_container =
            SharedContainer::new_owned_with_inferred_allowed_type(
                ValueContainer::from(42),
                SharedContainerMutability::Mutable,
                address_provider,
            );
        let ptr_address = owned_container.pointer_address();
        let ptr_address_hex = ptr_address.to_string();

        cache.store_shared_container(owned_container);

        let outer_ref = SerdeContext::<SharedContainer>::new(cache)
            .try_deserialize_from_json(&format!(r#""'{}""#, ptr_address_hex))
            .unwrap();

        assert_matches!(
            outer_ref,
            SharedContainer::Referenced(reference)
            if reference.reference_mutability() == ReferenceMutability::Immutable &&
                reference.pointer_address() == ptr_address
        );

        let outer_ref_mut = SerdeContext::<SharedContainer>::new(cache)
            .try_deserialize_from_json(&format!(r#""'mut{}""#, ptr_address_hex))
            .unwrap();

        assert_matches!(
            outer_ref_mut,
            SharedContainer::Referenced(reference)
            if reference.reference_mutability() == ReferenceMutability::Mutable &&
                reference.pointer_address() == ptr_address
        );

        let outer_owned = SerdeContext::<SharedContainer>::new(cache)
            .try_deserialize_from_json(&format!(r#""{}""#, ptr_address_hex))
            .unwrap();

        assert_matches!(
            outer_owned,
            SharedContainer::Owned(owned)
            if PointerAddress::SelfOwned(owned.pointer_address().clone()) == ptr_address
        );

        // should no longer exist in memory as owned container should have been taken from cache
        assert_matches!(
            cache.try_take_owned_shared_container(&ptr_address),
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
        cache.remove_shared_container(&ptr_address);

        assert_matches!(
            cache.try_take_owned_shared_container(&ptr_address),
            Err(CacheValueRetrievalError::ValueNotFoundInCache(
                ValueNotFoundInCacheError
            ))
        );
    }
}
