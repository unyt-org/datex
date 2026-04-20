use serde::de::DeserializeSeed;
use serde::Deserializer;
use crate::serde::deserialization_context::DeserializationContext;
use crate::serde::Deserialize;
use crate::shared_values::pointer_address::PointerAddress;
use crate::shared_values::shared_containers::SharedContainer;
use crate::values::value_container::ValueContainer;

/// Deserialization for [ValueContainer] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for DeserializationContext<'ctx, ValueContainer> {
    type Value = ValueContainer;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<ValueContainer, D::Error> {
        // try to deserialize string to pointer address first
        if let Ok(address) = PointerAddress::deserialize(d) {
            let reference = self.memory.get_reference(&address)
                .ok_or_else(|| serde::de::Error::custom(format!("Pointer address {} not found in memory", address)))?;
            Ok(ValueContainer::Shared(SharedContainer::Referenced(reference.clone())))
        }
        else {
            todo!()
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId};
    use crate::runtime::memory::Memory;
    use super::*;

    #[test]
    fn deserialize_pointer_address_to_shared_container() {
        let json = r#""030000""#; // #core.integer

        let memory = Memory::new();

        let outer = DeserializationContext::<ValueContainer>::new(&memory)
            .deserialize(&mut serde_json::Deserializer::from_str(json))
            .unwrap();

        println!("{:#?}", outer);

        assert_eq!(
            outer,
            ValueContainer::Shared(SharedContainer::Referenced(memory.get_core_reference(CoreLibTypeId::Base(CoreLibBaseTypeId::Integer)).clone()))
        );

    }
}