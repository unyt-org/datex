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
use serde::{Serialize, Serializer, ser::SerializeStruct};

/// Serialization for [ValueContainer].
impl Serialize for ValueContainer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ValueContainer::Shared(shared) => shared.serialize(serializer),
            ValueContainer::Local(value) => value.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use log::info;

    use super::*;
    use crate::{
        libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
        runtime::{
            memory::Memory,
            pointer_address_provider::SelfOwnedPointerAddressProvider,
        },
        shared_values::SharedContainerMutability,
        values::{core_value::CoreValue, core_values::integer::Integer},
    };

    #[test]
    fn value_container() {
        let memory = Memory::new();
        let integer_container =
            ValueContainer::Shared(SharedContainer::Referenced(
                memory
                    .get_core_reference(CoreLibTypeId::Base(
                        CoreLibBaseTypeId::Integer,
                    ))
                    .clone(),
            ));
        let serialized = serde_json::to_string(&integer_container).unwrap();
        assert_eq!(serialized, r#""'$030000""#);
    }
}
