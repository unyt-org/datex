use crate::prelude::*;
use serde::{Deserialize, Serialize};

pub mod interface;
pub mod reference;
pub mod representation;
pub mod r#type;
pub mod update;
pub mod value;

pub trait DIFConvertible: Serialize + for<'de> Deserialize<'de> {
    fn to_json(self) -> String {
        self.as_json()
    }
    fn to_json_pretty(self) -> String {
        self.as_json_pretty()
    }
    fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap()
    }
    fn as_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    fn as_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dif::{
            r#type::{DIFType, DIFTypeDefinition, DIFTypeMetadata},
            representation::DIFValueRepresentation,
            update::DIFUpdateData,
            value::{DIFValue, DIFValueContainer},
            DIFConvertible,
        },
        libs::core::CoreLibTypeId,
        prelude::*,
        shared_values::pointer_address::PointerAddress,
        types::structural_type_definition::StructuralTypeDefinition,
        values::{
            core_value::CoreValue,
            core_values::integer::typed_integer::IntegerTypeVariant,
            value::Value,
            value_container::ValueContainer,
        },
    };
    use alloc::string::ToString;
    use crate::types::r#type::Type;

    fn dif_value_circle(value_container: ValueContainer) -> DIFValueContainer {
        let dif_value_container: DIFValueContainer =
            DIFValueContainer::from_value_container(&value_container);
        let serialized = dif_value_container.as_json();
        let deserialized: DIFValueContainer =
            DIFValueContainer::from_json(&serialized);
        assert_eq!(dif_value_container, deserialized);
        dif_value_container
    }

    #[test]
    fn serde() {
        // replace
        let dif_update =
            DIFUpdateData::replace(DIFValueContainer::Value(DIFValue {
                value: DIFValueRepresentation::String("Hello".to_string()),
                ty: None,
            }));
        let serialized = dif_update.as_json();
        let deserialized: DIFUpdateData = DIFUpdateData::from_json(&serialized);
        assert_eq!(dif_update, deserialized);

        // update property
        let dif_update = DIFUpdateData::set(
            "name",
            DIFValueContainer::Value(DIFValue {
                value: DIFValueRepresentation::Number(42.0),
                ty: None,
            }),
        );
        let serialized = dif_update.as_json();
        let deserialized: DIFUpdateData = DIFUpdateData::from_json(&serialized);
        assert_eq!(dif_update, deserialized);
    }

    #[test]
    fn dif_value_serialization() {
        let value = DIFValue {
            value: DIFValueRepresentation::Null,
            ty: Some(DIFTypeDefinition::Unit),
        };
        let serialized = value.as_json();
        let deserialized = DIFValue::from_json(&serialized);
        assert_eq!(value, deserialized);
    }

    #[test]
    fn from_value_container_i32() {
        let dif_value_container = dif_value_circle(ValueContainer::from(42i32));
        if let DIFValueContainer::Value(dif_value) = &dif_value_container {
            assert_eq!(dif_value.value, DIFValueRepresentation::Number(42f64));
            assert_eq!(
                dif_value.ty,
                Some(DIFTypeDefinition::Reference(
                    CoreLibTypeId::Integer(Some(IntegerTypeVariant::I32))
                        .into()
                ))
            );
        } else {
            core::panic!("Expected DIFValueContainer::Value variant");
        }
    }

    #[test]
    fn from_value_container_text() {
        let dif_value_container =
            dif_value_circle(ValueContainer::from("Hello, World!"));
        if let DIFValueContainer::Value(dif_value) = &dif_value_container {
            assert_eq!(
                dif_value.value,
                DIFValueRepresentation::String("Hello, World!".to_string())
            );
            assert_eq!(dif_value.ty, None);
        } else {
            core::panic!("Expected DIFValueContainer::Value variant");
        }
    }

    #[test]
    fn dif_value_no_type() {
        let val = ValueContainer::Local(Value::null());
        let dif_val = DIFValueContainer::from_value_container(&val);
        assert_eq!(
            dif_val,
            DIFValueContainer::Value(DIFValue::new(
                DIFValueRepresentation::Null,
                Option::<DIFTypeDefinition>::None,
            ),)
        );
    }

    #[test]
    fn dif_value_with_type() {
        let val = ValueContainer::Local(Value {
            inner: CoreValue::Null,
            actual_type: Box::new(StructuralTypeDefinition::ImplType(
                Box::new(Type::integer()),
                vec![PointerAddress::owned([0, 0, 0, 0, 0])],
            )),
        });

        let dif_val = DIFValueContainer::from_value_container(&val);
        assert_eq!(
            dif_val,
            DIFValueContainer::Value(DIFValue {
                value: DIFValueRepresentation::Null,
                ty: Some(DIFTypeDefinition::ImplType(
                    Box::new(DIFType {
                        name: None,
                        metadata: DIFTypeMetadata::default(),
                        type_definition: DIFTypeDefinition::Reference(
                            PointerAddress::from(CoreLibTypeId::Integer(
                                None
                            ))
                        )
                    }),
                    vec![PointerAddress::owned([0, 0, 0, 0, 0])]
                ))
            })
        );
    }
}
