use crate::datex_structs_derive::dif::{
    deserialize_from_dif_default_ctx, serialize_to_json_value_default_ctx,
};
use datex_core::{
    libs::core::{
        core_lib_id::CoreLibIdIndex,
        type_id::{CoreLibBaseTypeId, CoreLibVariantTypeId},
    },
    prelude::*,
    values::core_values::{
        endpoint::Endpoint, integer::typed_integer::IntegerTypeVariant,
    },
};
use datex_macros_internal::Datex;

#[derive(Datex, Debug, Clone, PartialEq)]
#[datex(structural)]
enum ExampleEnum {
    Unit,
    TupleSingle(Endpoint),
    TupleMultiple(Endpoint, u8),
    Struct { x: String, y: u16 },
}

#[test]
fn enum_struct_to_dif() {
    let variant = ExampleEnum::Struct {
        x: "test".to_string(),
        y: 24,
    };
    let serialized = serialize_to_json_value_default_ctx(&variant);
    let json_value = serde_json::json!({
        "_": "t",
        "t": ["Struct"],
        "v": [
            CoreLibIdIndex::from(CoreLibBaseTypeId::Map),
            {
                "x": "test",
                "y": [CoreLibIdIndex::from(CoreLibVariantTypeId::Integer(IntegerTypeVariant::U16)), 24]
            }
        ]
    });
    println!("json_value: {json_value:#}");
    assert_eq!(serialized, json_value);
}

#[test]
fn enum_struct_from_dif() {
    let json_value = serde_json::json!({
        "_": "t",
        "t": ["Struct"],
        "v": [
            CoreLibIdIndex::from(CoreLibBaseTypeId::Map),
            {
                "x": "test",
                "y": [CoreLibIdIndex::from(CoreLibVariantTypeId::Integer(IntegerTypeVariant::U16)), 24]
            }
        ]
    });
    let deserialized: ExampleEnum =
        deserialize_from_dif_default_ctx(&json_value);

    assert_eq!(
        deserialized,
        ExampleEnum::Struct {
            x: "test".to_string(),
            y: 24
        }
    );
}

#[test]
fn enum_struct_roundtrip() {
    let variant = ExampleEnum::Struct {
        x: "test".to_string(),
        y: 24,
    };
    let serialized = serialize_to_json_value_default_ctx(&variant);
    let json_value = serde_json::json!({
        "_": "t",
        "t": ["Struct"],
        "v": [
            CoreLibIdIndex::from(CoreLibBaseTypeId::Map),
            {
                "x": "test",
                "y": [CoreLibIdIndex::from(CoreLibVariantTypeId::Integer(IntegerTypeVariant::U16)), 24]
            }
        ]
    });
    println!("json_value: {json_value:#}");
    assert_eq!(serialized, json_value);

    let deserialized: ExampleEnum =
        deserialize_from_dif_default_ctx(&json_value);

    assert_eq!(deserialized, variant);
}

#[test]
fn enum_tuple_single_roundtrip() {
    let variant = ExampleEnum::TupleSingle(Endpoint::new("@jonas"));
    let serialized = serialize_to_json_value_default_ctx(&variant);
    let json_value = serde_json::json!({
        "_": "t",
        "t": ["TupleSingle"],
        "v": [CoreLibIdIndex::from(CoreLibBaseTypeId::Endpoint), "@jonas"]
    });
    println!("json_value: {json_value:#}");
    assert_eq!(serialized, json_value);

    let deserialized: ExampleEnum =
        deserialize_from_dif_default_ctx(&json_value);

    assert_eq!(deserialized, variant);
}

#[test]
fn enum_tuple_multiple_roundtrip() {
    let variant = ExampleEnum::TupleMultiple(Endpoint::new("@jonas"), 42);
    let serialized = serialize_to_json_value_default_ctx(&variant);
    let json_value = serde_json::json!({
        "_": "t",
        "t": ["TupleMultiple"],
        "v": [
            CoreLibIdIndex::from(CoreLibBaseTypeId::List),
            [
                [CoreLibIdIndex::from(CoreLibBaseTypeId::Endpoint), "@jonas"],
                [CoreLibIdIndex::from(CoreLibVariantTypeId::Integer(IntegerTypeVariant::U8)), 42]
            ]
        ]
    });
    println!("json_value: {json_value:#}");
    assert_eq!(serialized, json_value);

    let deserialized: ExampleEnum =
        deserialize_from_dif_default_ctx(&json_value);

    assert_eq!(deserialized, variant);
}

#[test]
fn enum_unit_roundtrip() {
    let variant = ExampleEnum::Unit;
    let serialized = serialize_to_json_value_default_ctx(&variant);
    let json_value = serde_json::json!({
        "_": "t",
        "t": ["Unit", true],
        "v": null
    });
    println!("json_value: {json_value:#}");
    assert_eq!(serialized, json_value);

    let deserialized: ExampleEnum =
        deserialize_from_dif_default_ctx(&json_value);

    assert_eq!(deserialized, variant);
}
