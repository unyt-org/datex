use crate::{
    prelude::*,
    shared_values::{
        ReferenceMutability, SharedContainer, SharedContainerOwnership,
    },
    values::{
        core_value::CoreValue, core_values::integer::Integer, value::Value,
        value_container::ValueContainer,
    },
};
use alloc::format;
use serde::{Serialize, Serializer, ser::SerializeStruct};

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

#[cfg(test)]
mod tests {
    use crate::{
        libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
        runtime::{
            memory::Memory,
            pointer_address_provider::SelfOwnedPointerAddressProvider,
        },
        shared_values::{SharedContainer, SharedContainerMutability},
        values::value_container::ValueContainer,
    };
    use alloc::format;

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
}
