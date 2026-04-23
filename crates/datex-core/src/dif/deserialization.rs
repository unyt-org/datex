use crate::{
    dif::deserialization_context::DeserializationContext,
    prelude::*,
    shared_values::{
        pointer_address::PointerAddress,
        shared_containers::{ReferenceMutability, SharedContainerOwnership},
    },
    values::{
        core_value::CoreValue, core_values::list::List, value::Value,
        value_container::ValueContainer,
    },
};
use alloc::format;
use core::fmt;
use serde::{
    Deserializer,
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
};
/// Deserialization for [ValueContainer] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de>
    for DeserializationContext<'ctx, ValueContainer>
{
    type Value = ValueContainer;
    fn deserialize<D: Deserializer<'de>>(
        self,
        d: D,
    ) -> Result<ValueContainer, D::Error> {
        d.deserialize_any(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for DeserializationContext<'ctx, ValueContainer> {
    type Value = ValueContainer;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a pointer address string or a Value map")
    }

    /// Visits a pointer address with ownership information encoded as string:
    /// "'$ABCDEF" | "'mut$ABCDEF" | "$ABCDEF"
    fn visit_str<E: serde::de::Error>(
        self,
        v: &str,
    ) -> Result<ValueContainer, E> {
        // split str at "$" to check for reference prefix
        let (prefix, address_str) = match v.split_once('$') {
            Some((prefix, address_str)) => (Some(prefix), address_str),
            None => (None, v),
        };
        let address = PointerAddress::try_from(address_str).map_err(|_| {
            E::custom(format!("invalid pointer address: {}", v))
        })?;
        let ownership = match prefix {
            Some("'mut") => SharedContainerOwnership::Referenced(
                ReferenceMutability::Mutable,
            ),
            Some("'") => SharedContainerOwnership::Referenced(
                ReferenceMutability::Immutable,
            ),
            Some("") => SharedContainerOwnership::Owned,
            None => {
                return Err(E::custom(format!(
                    "invalid pointer address: {}",
                    v
                )));
            }
            Some(other) => {
                return Err(E::custom(format!(
                    "invalid pointer address prefix '{}': {}",
                    other, v
                )));
            }
        };
        let reference = self
            .shared_container_cache
            .try_get_shared_container_with_ownership(&address, ownership)
            .map_err(|e| {
                E::custom(format!("Cannot get {} from DIF cache: {}", v, e))
            })?;
        Ok(ValueContainer::Shared(reference))
    }

    // map => local Value
    fn visit_map<A: MapAccess<'de>>(
        mut self,
        map: A,
    ) -> Result<ValueContainer, A::Error> {
        // reuse the Value visitor directly
        let value = self.cast::<Value>().visit_map(map)?;
        Ok(ValueContainer::Local(value))
    }
}

/// Deserialization for [Value] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for DeserializationContext<'ctx, Value> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // deserialize "value" property as CoreValue
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for DeserializationContext<'ctx, Value> {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("struct Value with a 'value' property")
    }

    fn visit_map<A: MapAccess<'de>>(
        mut self,
        mut map: A,
    ) -> Result<Value, A::Error> {
        let mut core_value: Option<CoreValue> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "value" => {
                    core_value =
                        Some(map.next_value_seed(self.cast::<CoreValue>())?);
                }
                _ => {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
        }

        let core_value = core_value
            .ok_or_else(|| serde::de::Error::missing_field("value"))?;
        Ok(Value {
            inner: core_value,
            custom_type: None,
        })
    }
}

/// Deserialization for [CoreValue] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de>
    for DeserializationContext<'ctx, CoreValue>
{
    type Value = CoreValue;

    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<CoreValue, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for DeserializationContext<'ctx, CoreValue> {
    type Value = CoreValue;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a CoreValue")
    }

    fn visit_seq<A: SeqAccess<'de>>(
        mut self,
        mut seq: A,
    ) -> Result<CoreValue, A::Error> {
        let mut items = Vec::new();
        while let Some(item) =
            seq.next_element_seed(self.cast::<ValueContainer>())?
        {
            items.push(item);
        }
        Ok(CoreValue::List(List::from(items)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dif::cache::{
            CacheValueRetrievalError, DIFSharedContainerCache,
            ValueNotFoundInCacheError,
        },
        libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
        runtime::{
            memory::Memory,
            pointer_address_provider::SelfOwnedPointerAddressProvider,
        },
        shared_values::{
            SharedContainer, SharedContainerMutability,
            errors::UnexpectedSharedContainerOwnershipError,
        },
        values::{core_value::CoreValue, core_values::list::List},
    };
    use core::assert_matches;

    fn deserialize_json_string(
        str: impl Into<String>,
        dif_cache: &mut DIFSharedContainerCache,
    ) -> ValueContainer {
        DeserializationContext::<ValueContainer>::new(dif_cache)
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

        assert_eq!(outer, ValueContainer::Shared(integer_container));
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
            ValueContainer::Shared(SharedContainer::Referenced(reference))
            if reference.reference_mutability() == ReferenceMutability::Immutable &&
                reference.pointer_address() == ptr_address
        );

        let outer_ref_mut = deserialize_json_string(
            format!(r#""'mut{}""#, ptr_address_hex),
            dif_cache,
        );
        assert_matches!(
            outer_ref_mut,
            ValueContainer::Shared(SharedContainer::Referenced(reference))
            if reference.reference_mutability() == ReferenceMutability::Mutable &&
                reference.pointer_address() == ptr_address
        );

        let outer_owned = deserialize_json_string(
            format!(r#""{}""#, ptr_address_hex),
            dif_cache,
        );
        assert_matches!(
            outer_owned,
            ValueContainer::Shared(SharedContainer::Owned(owned))
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

    #[test]
    fn deserialize_nested_pointer_address_to_shared_container() {
        let json = r#"{"value": ["'$030000"]}"#; // [integer]

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

        assert_eq!(
            outer,
            ValueContainer::Local(Value::from(CoreValue::List(List::from(
                vec![ValueContainer::Shared(integer_container)]
            ))))
        );
    }
}
