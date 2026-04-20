use serde::de::DeserializeSeed;
use serde::{Deserializer, Serialize, Serializer};
use crate::serde::deserialization_context::DeserializationContext;
use crate::serde::Deserialize;
use crate::shared_values::pointer_address::PointerAddress;
use crate::shared_values::shared_containers::SharedContainer;
use crate::values::value_container::ValueContainer;

/// Serialization for [ValueContainer].
impl Serialize for ValueContainer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        match self {
            ValueContainer::Shared(shared) => shared.pointer_address().serialize(serializer),
            _ => todo!()
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId};
    use crate::runtime::memory::Memory;
    use super::*;

    #[test]
    fn serialize_shared_container() {
        let memory = Memory::new();
        let integer_container = SharedContainer::Referenced(memory.get_core_reference(CoreLibTypeId::Base(CoreLibBaseTypeId::Integer)).clone());
        let serialized = serde_json::to_string(&integer_container).unwrap();
        println!("{}", serialized);
        assert_eq!(serialized, r#""030000""#);
    }
}