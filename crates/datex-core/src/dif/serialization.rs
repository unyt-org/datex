use crate::{
    prelude::*,
    shared_values::shared_containers::{
        ReferenceMutability, SharedContainer, SharedContainerOwnership,
    },
    values::{
        core_value::CoreValue, core_values::integer::Integer, value::Value,
        value_container::ValueContainer,
    },
};
use alloc::format;
use serde::{Serialize, Serializer, ser::SerializeStruct};

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
        shared_values::shared_containers::SharedContainerMutability,
        values::{core_value::CoreValue, core_values::integer::Integer},
    };

    #[test]
    fn serialize_simple_local_value() {
        let value_container = ValueContainer::Local(Value::from(
            CoreValue::Integer(Integer::new(42)),
        ));
        let serialized = serde_json::to_string(&value_container).unwrap();
        assert_eq!(serialized, r#"{"value":"42"}"#);
    }
}
