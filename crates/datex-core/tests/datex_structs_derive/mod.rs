use core::assert_matches;
use datex_core::{
    assert_structural_eq,
    datex_proxy::{DatexProxyTypes, DatexValueContainerProxy},
    prelude::*,
    values::{
        core_values::{endpoint::Endpoint, map::Map},
        value_container::ValueContainer,
    },
};
use datex_macros_internal::Datex;
use serde::{Deserialize, Serialize};

#[derive(Datex, Debug)]
enum ExampleEnum {
    VariantA,
    VariantB(u8, u8),
    VariantC { x: u8, y: String },
    VariantD(u8),
}

#[derive(Datex, Debug, Clone, PartialEq)]
struct Example {
    a: u8,
    b: String,
    c: Endpoint,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SerdeExample {
    inner_a: u8,
    inner_b: String,
}

#[derive(Datex, Debug, Clone, PartialEq)]
struct SerdeDatexExample {
    a: u8,
    #[datex(serde)]
    serde: SerdeExample,
}

#[derive(Datex, Debug, PartialEq)]
struct ExampleNewType(Example);

fn assert_round_trip<T>(value: T)
where
    T: DatexValueContainerProxy + PartialEq + std::fmt::Debug + Clone,
{
    let value_container = value.clone().try_to_value_container().unwrap();
    let deserialized_value =
        T::try_from_value_container(value_container).unwrap();
    assert_eq!(value, deserialized_value);
}

use datex_core::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibVariantTypeId},
    runtime::{
        cache::shared_references_cache::SharedReferencesCache,
        pointer_address_provider::SelfOwnedPointerAddressProvider,
    },
    shared_values::{
        OwnedSharedContainer, PointerAddress, SharedContainer,
        SharedContainerMutability,
    },
    types::{
        literal_type_definition::LiteralTypeDefinition,
        r#type::Type,
        type_definition::{
            TypeDefinition, list::ListTypeDefinition, map::MapTypeDefinition,
            tagged_type::TaggedTypeDefinition, union::UnionTypeDefinition,
        },
    },
    values::{
        core_value::CoreValue,
        core_values::integer::typed_integer::IntegerTypeVariant, value::Value,
    },
};
use test_case::test_case;

#[test_case(
    Example {
        a: 42u8,
        b: "Test".to_string(),
        c: Endpoint::default(),
    } ; "example struct")]
#[test_case(
    SerdeDatexExample {
        a: 42u8,
        serde: SerdeExample {
            inner_a: 1,
            inner_b: "Inner".to_string(),
        },
    } ; "struct with serde field")]
#[test_case(vec![1u8, 2, 3] ; "vector of primitives")]
#[test_case(vec![Endpoint::try_from("@ben").unwrap(), Endpoint::try_from("@jonas").unwrap()] ; "vector of datex direct types")]
#[test_case(Map::from(vec![
    ("key1".to_string(), ValueContainer::from(42u8)),
    ("key2".to_string(), ValueContainer::from("Value".to_string())),
]) ; "map of primitives")]
fn round_trip_struct<T>(structure: T)
where
    T: DatexValueContainerProxy + PartialEq + std::fmt::Debug + Clone,
{
    assert_round_trip(structure);
}

#[test]
fn struct_to_value_container() {
    let value_container: ValueContainer = Example {
        a: 42u8,
        b: "Test".to_string(),
        c: Endpoint::default(),
    }
    .into();

    let map: Map = value_container.try_into_value().unwrap();
    assert_eq!(map.get("a").unwrap(), &ValueContainer::from(42u8));
    assert_eq!(
        map.get("b").unwrap(),
        &ValueContainer::from("Test".to_string())
    );
    assert_eq!(
        map.get("c").unwrap(),
        &ValueContainer::from(Endpoint::default())
    );
}

#[test]
fn skip() {
    #[derive(Datex, Debug, PartialEq)]
    struct SerdeDatexWithSkip {
        a: u8,

        #[datex(skip)]
        b: String,
    }
    let value_container: ValueContainer = SerdeDatexWithSkip {
        a: 42,
        b: "Hello".to_string(),
    }
    .into();

    let map: Map = value_container.try_into_value().unwrap();
    assert!(map.has("a"));
    assert!(!map.has("b"));

    let value_container = ValueContainer::from(map);
    let deserialized: SerdeDatexWithSkip = value_container.try_into().unwrap();

    assert_eq!(deserialized.a, 42);
    assert_eq!(deserialized.b, "".to_string());
}

#[test]
fn skip2() {
    #[derive(Default, Debug, PartialEq)]
    struct NoDerive {
        a: u8,
        b: String,
    }
    #[derive(Datex, Debug, PartialEq)]
    struct SerdeDatexWithSkip2 {
        a: u8,
        #[datex(skip)]
        b: NoDerive,
    }
    let value_container: ValueContainer = SerdeDatexWithSkip2 {
        a: 42,
        b: NoDerive {
            a: 1,
            b: "Hello".to_string(),
        },
    }
    .into();
    let deserialized: SerdeDatexWithSkip2 = value_container.try_into().unwrap();
    assert_eq!(deserialized.a, 42);
    assert_eq!(deserialized.b, NoDerive::default());
}

#[test]
fn default() {
    #[derive(Datex, Debug, PartialEq)]
    struct SerdeDatexWithDefault {
        a: u8,
        #[datex(default)]
        b: String,
    }

    let map: Map =
        Map::from(vec![("a".to_string(), ValueContainer::from(42u8))]);
    let value_container = ValueContainer::from(map);
    let deserialized: SerdeDatexWithDefault =
        value_container.try_into().unwrap();
    assert_eq!(deserialized.a, 42);
    assert_eq!(deserialized.b, "".to_string());
}

#[test]
fn enum_to_value() {
    let variant_a: Value = ExampleEnum::VariantA.into();

    assert_structural_eq!(variant_a, Value::null());
    assert_eq!(
        variant_a.custom_type,
        Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantA".to_string(),
            ty: None
        }))
    );

    let variant_b: Value = ExampleEnum::VariantB(1, 2).into();
    assert_structural_eq!(
        variant_b,
        Value::from(vec![Value::from(1u8), Value::from(2u8)])
    );
    assert_eq!(
        variant_b.custom_type,
        Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantB".to_string(),
            ty: None,
        }))
    );

    let variant_c: Value = ExampleEnum::VariantC {
        x: 3,
        y: "Hello".to_string(),
    }
    .into();
    assert_structural_eq!(
        variant_c,
        Value::from(Map::from(vec![
            ("x".to_string(), Value::from(3).into()),
            ("y".to_string(), Value::from("Hello".to_string()).into()),
        ]))
    );
    assert_eq!(
        variant_c.custom_type,
        Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantC".to_string(),
            ty: None,
        }))
    );

    let variant_d: Value = ExampleEnum::VariantD(1).into();
    assert_structural_eq!(variant_d, Value::from(1u8));
    assert_eq!(
        variant_d.custom_type,
        Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantD".to_string(),
            ty: None,
        }))
    );
}

#[test]
fn struct_to_value() {
    let value: Value = Example {
        a: 42u8,
        b: "Test".to_string(),
        c: Endpoint::default(),
    }
    .into();

    let map: Map = value.try_into().unwrap();
    assert_eq!(map.get("a").unwrap(), &ValueContainer::from(42u8));
    assert_eq!(
        map.get("b").unwrap(),
        &ValueContainer::from("Test".to_string())
    );
    assert_eq!(
        map.get("c").unwrap(),
        &ValueContainer::from(Endpoint::default())
    );
}

#[test]
fn new_type_struct_to_value() {
    let value: Value = ExampleNewType(Example {
        a: 42u8,
        b: "Test".to_string(),
        c: Endpoint::default(),
    })
    .into();

    let map: Map = value.try_into().unwrap();
    assert_eq!(map.get("a").unwrap(), &ValueContainer::from(42u8));
    assert_eq!(
        map.get("b").unwrap(),
        &ValueContainer::from("Test".to_string())
    );
    assert_eq!(
        map.get("c").unwrap(),
        &ValueContainer::from(Endpoint::default())
    );
}

#[test]
fn value_container_to_struct() {
    let value_container: ValueContainer =
        ValueContainer::from(Map::from(vec![
            ("a".to_string(), ValueContainer::from(42u8)),
            ("b".to_string(), ValueContainer::from("Test".to_string())),
            ("c".to_string(), ValueContainer::from(Endpoint::default())),
        ]));

    let example: Example = value_container.try_into().unwrap();

    assert_eq!(example.a, 42u8);
    assert_eq!(example.b, "Test".to_string());
    assert_eq!(example.c, Endpoint::default());
}

#[test]
fn value_to_struct() {
    let value: Value = Value::from(Map::from(vec![
        ("a".to_string(), ValueContainer::from(42u8)),
        ("b".to_string(), ValueContainer::from("Test".to_string())),
        ("c".to_string(), ValueContainer::from(Endpoint::default())),
    ]));

    let example: Example = value.try_into().unwrap();

    assert_eq!(example.a, 42u8);
    assert_eq!(example.b, "Test".to_string());
    assert_eq!(example.c, Endpoint::default());
}

#[test]
fn value_to_new_typestruct() {
    let value: Value = Value::from(Map::from(vec![
        ("a".to_string(), ValueContainer::from(42u8)),
        ("b".to_string(), ValueContainer::from("Test".to_string())),
        ("c".to_string(), ValueContainer::from(Endpoint::default())),
    ]));

    let example: ExampleNewType = value.try_into().unwrap();

    assert_eq!(example.0.a, 42u8);
    assert_eq!(example.0.b, "Test".to_string());
    assert_eq!(example.0.c, Endpoint::default());
}

#[test]
fn value_to_enum() {
    let variant_a = Value {
        inner: CoreValue::Null,
        custom_type: Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantA".to_string(),
            ty: None,
        })),
    };

    let example: ExampleEnum = variant_a.try_into().unwrap();
    assert_matches!(example, ExampleEnum::VariantA);

    let variant_b = Value {
        inner: CoreValue::from(vec![
            ValueContainer::from(1u8),
            ValueContainer::from(2u8),
        ]),
        custom_type: Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantB".to_string(),
            ty: None,
        })),
    };
    let example: ExampleEnum = variant_b.try_into().unwrap();
    assert_matches!(example, ExampleEnum::VariantB(1, 2));

    let variant_c = Value {
        inner: CoreValue::from(Map::from(vec![
            ("x".to_string(), ValueContainer::from(3u8)),
            ("y".to_string(), ValueContainer::from("Hello".to_string())),
        ])),
        custom_type: Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantC".to_string(),
            ty: None,
        })),
    };
    let example: ExampleEnum = variant_c.try_into().unwrap();
    assert_matches!(example, ExampleEnum::VariantC { x: 3, y } if &y == "Hello" );

    let variant_d = Value {
        inner: CoreValue::from(42u8),
        custom_type: Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantD".to_string(),
            ty: None,
        })),
    };
    let example: ExampleEnum = variant_d.try_into().unwrap();
    assert_matches!(example, ExampleEnum::VariantD(42));
}

#[test]
fn value_to_enum_failure() {
    let invalid_variant = Value {
        inner: CoreValue::from(vec![
            ValueContainer::from(1u8),
            ValueContainer::from(2u8),
        ]),
        custom_type: Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantX".to_string(),
            ty: None,
        })),
    };
    assert!(ExampleEnum::try_from(invalid_variant).is_err());

    let invalid_variant = Value {
        inner: CoreValue::from(42),
        custom_type: Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantA".to_string(),
            ty: None,
        })),
    };
    assert!(ExampleEnum::try_from(invalid_variant).is_err());
}

#[test]
fn struct_with_serde_to_value_container() {
    let serde_example = SerdeDatexExample {
        a: 42u8,
        serde: SerdeExample {
            inner_a: 1,
            inner_b: "Inner".to_string(),
        },
    };

    // Note: uses try_into because of datex(serde)
    let value_container: ValueContainer = serde_example.try_into().unwrap();

    let map: Map = value_container.try_into_value().unwrap();
    assert_eq!(map.get("a").unwrap(), &ValueContainer::from(42u8));
    let serde_map: Map = map.get("serde").unwrap().clone().try_into_value().unwrap();
    assert_eq!(
        serde_map.get("inner_a").unwrap(),
        &ValueContainer::from(1u8)
    );
    assert_eq!(
        serde_map.get("inner_b").unwrap(),
        &ValueContainer::from("Inner".to_string())
    );
}

#[test]
fn struct_with_serde_infallible_to_value_container() {
    #[derive(Datex, Debug, Clone, PartialEq)]
    struct SerdeDatexExampleInfallible {
        a: u8,
        #[datex(serde_infallible)]
        serde: SerdeExample,
    }

    let serde_example = SerdeDatexExampleInfallible {
        a: 42u8,
        serde: SerdeExample {
            inner_a: 1,
            inner_b: "Inner".to_string(),
        },
    };

    // Note: uses into instead of try_into because of datex(serde_infallible)
    let value_container: ValueContainer = serde_example.into();

    let map: Map = value_container.try_into_value().unwrap();
    assert_eq!(map.get("a").unwrap(), &ValueContainer::from(42u8));
    let serde_map: Map = map.get("serde").unwrap().clone().try_into_value().unwrap();
    assert_eq!(
        serde_map.get("inner_a").unwrap(),
        &ValueContainer::from(1u8)
    );
    assert_eq!(
        serde_map.get("inner_b").unwrap(),
        &ValueContainer::from("Inner".to_string())
    );
}

#[test]
fn struct_with_value_container() {
    let address_provider = &mut SelfOwnedPointerAddressProvider::default();

    #[derive(Datex, Debug, PartialEq)]
    struct ExampleWithValueContainer {
        a: u8,
        val: ValueContainer,
    }

    // local inner value container
    let example_local = ExampleWithValueContainer {
        a: 42u8,
        val: ValueContainer::from("Test".to_string()),
    };

    let value: Value = example_local.into();
    let map: Map = value.try_into().unwrap();
    assert_eq!(map.get("a").unwrap(), &ValueContainer::from(42u8));
    assert_eq!(
        map.get("val").unwrap(),
        &ValueContainer::from("Test".to_string())
    );

    // shared inner value container
    let shared_container = ValueContainer::Shared(
        SharedContainer::new_owned_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Mutable,
            address_provider,
        ),
    );
    let example_shared = ExampleWithValueContainer {
        a: 42u8,
        val: shared_container.clone(),
    };

    let value_container: ValueContainer = example_shared.into();
    let map: &Map = value_container.try_as().unwrap();
    assert_eq!(map.get("a").unwrap(), &ValueContainer::from(42u8));
    assert_eq!(map.get("val").unwrap(), &shared_container);

    let deserialized_example_shared: ExampleWithValueContainer =
        value_container.try_into().unwrap();
    assert_eq!(deserialized_example_shared.a, 42u8);
    assert_eq!(deserialized_example_shared.val, shared_container);
}

#[test]
fn struct_with_owned_shared_value_container() {
    let address_provider = &mut SelfOwnedPointerAddressProvider::default();

    #[derive(Datex, Debug, PartialEq)]
    struct ExampleWithOwnedContainer {
        owned: OwnedSharedContainer,
    }

    let owned_container = OwnedSharedContainer::new_with_inferred_allowed_type(
        42,
        SharedContainerMutability::Mutable,
        address_provider,
    );
    let address = owned_container.pointer_address().clone();
    let example = ExampleWithOwnedContainer {
        owned: owned_container,
    };

    let value_container: ValueContainer = example.into();

    value_container.try_with(|map: &Map| {
        if let ValueContainer::Shared(SharedContainer::Owned(
            shared_container,
        )) = map.get("owned").unwrap()
        {
            assert_eq!(*shared_container.pointer_address(), address);
        } else {
            panic!("Expected a Shared Owned variant in the ValueContainer");
        }
    });

    // TODO: function mapping, SharedRef<x>, Shared<x>
}

#[test]
fn get_datex_type_from_struct() {
    let dx_type = Example::datex_type(&mut SharedReferencesCache::default());
    println!("{}", dx_type);

    assert_eq!(
        dx_type,
        Type::Alias(
            TypeDefinition::Map(MapTypeDefinition(vec![
                (
                    Type::Alias(
                        TypeDefinition::Literal(LiteralTypeDefinition::Text(
                            "a".into()
                        ))
                        .into()
                    ),
                    Type::Alias(
                        TypeDefinition::CoreType(
                            CoreLibVariantTypeId::Integer(
                                IntegerTypeVariant::U8
                            )
                            .into()
                        )
                        .into()
                    )
                ),
                (
                    Type::Alias(
                        TypeDefinition::Literal(LiteralTypeDefinition::Text(
                            "b".into()
                        ))
                        .into()
                    ),
                    Type::Alias(
                        TypeDefinition::CoreType(
                            CoreLibBaseTypeId::Text.into()
                        )
                        .into()
                    )
                ),
                (
                    Type::Alias(
                        TypeDefinition::Literal(LiteralTypeDefinition::Text(
                            "c".into()
                        ))
                        .into()
                    ),
                    Type::Alias(
                        TypeDefinition::CoreType(
                            CoreLibBaseTypeId::Endpoint.into()
                        )
                        .into()
                    )
                )
            ]))
            .into()
        )
    )
}

#[test]
fn get_datex_type_from_enum() {
    let dx_type = ExampleEnum::datex_type(&mut SharedReferencesCache::default());
    println!("{}", dx_type);

    assert_eq!(
        dx_type,
        Type::Alias(
            TypeDefinition::Union(UnionTypeDefinition(vec![
                TypeDefinition::TaggedType(TaggedTypeDefinition {
                    tag: "VariantA".to_string(),
                    ty: None,
                })
                .into(),
                TypeDefinition::TaggedType(TaggedTypeDefinition {
                    tag: "VariantB".to_string(),
                    ty: Some(Box::new(
                        TypeDefinition::List(ListTypeDefinition(vec![
                            Type::Alias(
                                TypeDefinition::CoreType(
                                    CoreLibVariantTypeId::Integer(
                                        IntegerTypeVariant::U8
                                    )
                                    .into()
                                )
                                .into()
                            ),
                            Type::Alias(
                                TypeDefinition::CoreType(
                                    CoreLibVariantTypeId::Integer(
                                        IntegerTypeVariant::U8
                                    )
                                    .into()
                                )
                                .into()
                            ),
                        ]))
                        .into()
                    ))
                })
                .into(),
                TypeDefinition::TaggedType(TaggedTypeDefinition {
                    tag: "VariantC".to_string(),
                    ty: Some(Box::new(
                        TypeDefinition::Map(MapTypeDefinition(vec![
                            (
                                Type::Alias(
                                    TypeDefinition::Literal(
                                        LiteralTypeDefinition::Text("x".into())
                                    )
                                    .into()
                                ),
                                Type::Alias(
                                    TypeDefinition::CoreType(
                                        CoreLibVariantTypeId::Integer(
                                            IntegerTypeVariant::U8
                                        )
                                        .into()
                                    )
                                    .into()
                                )
                            ),
                            (
                                Type::Alias(
                                    TypeDefinition::Literal(
                                        LiteralTypeDefinition::Text("y".into())
                                    )
                                    .into()
                                ),
                                Type::Alias(
                                    TypeDefinition::CoreType(
                                        CoreLibBaseTypeId::Text.into()
                                    )
                                    .into()
                                )
                            )
                        ]))
                        .into()
                    ))
                })
                .into(),
                TypeDefinition::TaggedType(TaggedTypeDefinition {
                    tag: "VariantD".to_string(),
                    ty: Some(Box::new(
                        TypeDefinition::CoreType(
                            CoreLibVariantTypeId::Integer(
                                IntegerTypeVariant::U8
                            )
                            .into()
                        )
                        .into()
                    ))
                })
                .into(),
            ]))
            .into()
        )
    );
}
