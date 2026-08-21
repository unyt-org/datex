mod impls;

use core::assert_matches;
use datex_core::{
    datex_proxy::{DatexProxyType, DatexValueContainerProxy},
    prelude::*,
    values::{
        core_values::{endpoint::Endpoint, map::Map},
        value_container::ValueContainer,
    },
};
use datex_macros_internal::Datex;
use serde::{Deserialize, Serialize};

#[derive(Datex, Debug)]
#[datex(structural_recursive)]
enum ExampleEnum {
    VariantA,
    VariantB(u8, u8),
    VariantC { x: u8, y: String },
    VariantD(u8),
}

#[derive(Datex, Debug, Clone, PartialEq)]
#[datex(structural_recursive)]
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
#[datex(structural_recursive)]
struct SerdeDatexExample {
    a: u8,
    #[datex(serde)]
    serde: SerdeExample,
}

#[derive(Datex, Debug, PartialEq)]
#[datex(structural_recursive)]
struct ExampleNewType(Example);

fn assert_round_trip<T>(value: T)
where
    T: DatexValueContainerProxy + PartialEq + std::fmt::Debug + Clone,
{
    let value_container = value
        .clone()
        .try_to_value_container_without_cache()
        .unwrap();
    let deserialized_value =
        T::try_from_value_container(value_container).unwrap();
    assert_eq!(value, deserialized_value);
}

use datex_core::{
    self,
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibVariantTypeId},
    runtime::{
        cache::shared_references_cache::SharedReferencesCache,
        pointer_address_provider::SelfOwnedPointerAddressProvider,
    },
    shared_values::{
        OwnedSharedContainer, SharedContainer, SharedContainerMutability,
    },
    traits::structural_eq::assert_structural_eq,
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
    assert_eq!(map.try_get("a").unwrap(), &ValueContainer::from(42u8));
    assert_eq!(
        map.try_get("b").unwrap(),
        &ValueContainer::from("Test".to_string())
    );
    assert_eq!(
        map.try_get("c").unwrap(),
        &ValueContainer::from(Endpoint::default())
    );
}

#[test]
fn skip() {
    #[derive(Datex, Debug, PartialEq)]
    #[datex(structural_recursive)]
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
    #[datex(structural_recursive)]
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
    #[datex(structural_recursive)]
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
    assert_eq!(map.try_get("a").unwrap(), &ValueContainer::from(42u8));
    assert_eq!(
        map.try_get("b").unwrap(),
        &ValueContainer::from("Test".to_string())
    );
    assert_eq!(
        map.try_get("c").unwrap(),
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
    assert_eq!(map.try_get("a").unwrap(), &ValueContainer::from(42u8));
    assert_eq!(
        map.try_get("b").unwrap(),
        &ValueContainer::from("Test".to_string())
    );
    assert_eq!(
        map.try_get("c").unwrap(),
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
    let variant_a = Value::new(
        CoreValue::Null,
        Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantA".to_string(),
            ty: None,
        })),
    );

    let example: ExampleEnum = variant_a.try_into().unwrap();
    assert_matches!(example, ExampleEnum::VariantA);

    let variant_b = Value::new(
        CoreValue::from(vec![
            ValueContainer::from(1u8),
            ValueContainer::from(2u8),
        ]),
        Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantB".to_string(),
            ty: None,
        })),
    );
    let example: ExampleEnum = variant_b.try_into().unwrap();
    assert_matches!(example, ExampleEnum::VariantB(1, 2));

    let variant_c = Value::new(
        CoreValue::from(Map::from(vec![
            ("x".to_string(), ValueContainer::from(3u8)),
            ("y".to_string(), ValueContainer::from("Hello".to_string())),
        ])),
        Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantC".to_string(),
            ty: None,
        })),
    );
    let example: ExampleEnum = variant_c.try_into().unwrap();
    assert_matches!(example, ExampleEnum::VariantC { x: 3, y } if &y == "Hello" );

    let variant_d = Value::new(
        CoreValue::from(42u8),
        Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantD".to_string(),
            ty: None,
        })),
    );
    let example: ExampleEnum = variant_d.try_into().unwrap();
    assert_matches!(example, ExampleEnum::VariantD(42));
}

#[test]
fn value_to_enum_failure() {
    let invalid_variant = Value::new(
        CoreValue::from(vec![
            ValueContainer::from(1u8),
            ValueContainer::from(2u8),
        ]),
        Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantX".to_string(),
            ty: None,
        })),
    );
    assert!(ExampleEnum::try_from(invalid_variant).is_err());

    let invalid_variant = Value::new(
        CoreValue::from(42u8),
        Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "VariantA".to_string(),
            ty: None,
        })),
    );
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
    assert_eq!(map.try_get("a").unwrap(), &ValueContainer::from(42u8));
    let serde_map: Map = map
        .try_get("serde")
        .unwrap()
        .clone()
        .try_into_value()
        .unwrap();
    assert_eq!(
        serde_map.try_get("inner_a").unwrap(),
        &ValueContainer::from(1u8)
    );
    assert_eq!(
        serde_map.try_get("inner_b").unwrap(),
        &ValueContainer::from("Inner".to_string())
    );
}

#[test]
fn struct_with_serde_infallible_to_value_container() {
    #[derive(Datex, Debug, Clone, PartialEq)]
    #[datex(structural_recursive)]
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
    assert_eq!(map.try_get("a").unwrap(), &ValueContainer::from(42u8));
    let serde_map: Map = map
        .try_get("serde")
        .unwrap()
        .clone()
        .try_into_value()
        .unwrap();
    assert_eq!(
        serde_map.try_get("inner_a").unwrap(),
        &ValueContainer::from(1u8)
    );
    assert_eq!(
        serde_map.try_get("inner_b").unwrap(),
        &ValueContainer::from("Inner".to_string())
    );
}

#[test]
fn struct_with_value_container() {
    let address_provider = &mut SelfOwnedPointerAddressProvider::default();

    #[derive(Datex, Debug, PartialEq)]
    #[datex(structural_recursive)]
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
    assert_eq!(map.try_get("a").unwrap(), &ValueContainer::from(42u8));
    assert_eq!(
        map.try_get("val").unwrap(),
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
    assert_eq!(map.try_get("a").unwrap(), &ValueContainer::from(42u8));
    assert_eq!(map.try_get("val").unwrap(), &shared_container);

    let deserialized_example_shared: ExampleWithValueContainer =
        value_container.try_into().unwrap();
    assert_eq!(deserialized_example_shared.a, 42u8);
    assert_eq!(deserialized_example_shared.val, shared_container);
}

#[test]
fn struct_with_owned_shared_value_container() {
    let address_provider = &mut SelfOwnedPointerAddressProvider::default();

    #[derive(Datex, Debug, PartialEq)]
    #[datex(structural_recursive)]
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

    let map: &Map = value_container.try_as().unwrap();

    if let ValueContainer::Shared(SharedContainer::Owned(shared_container)) =
        map.try_get("owned").unwrap()
    {
        assert_eq!(*shared_container.pointer_address(), address);
    } else {
        panic!("Expected a Shared Owned variant in the ValueContainer");
    }

    // TODO: function mapping, SharedRef<x>, Shared<x>
}

#[test]
fn get_datex_type_from_struct() {
    let dx_type = Example::datex_type_without_cache();
    println!("{}", dx_type);

    assert_eq!(
        dx_type,
        Type::Definition(
            TypeDefinition::Map(MapTypeDefinition(vec![
                (
                    Type::Definition(
                        TypeDefinition::Literal(LiteralTypeDefinition::Text(
                            "a".into()
                        ))
                        .into()
                    ),
                    Type::Definition(
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
                    Type::Definition(
                        TypeDefinition::Literal(LiteralTypeDefinition::Text(
                            "b".into()
                        ))
                        .into()
                    ),
                    Type::Definition(
                        TypeDefinition::CoreType(
                            CoreLibBaseTypeId::Text.into()
                        )
                        .into()
                    )
                ),
                (
                    Type::Definition(
                        TypeDefinition::Literal(LiteralTypeDefinition::Text(
                            "c".into()
                        ))
                        .into()
                    ),
                    Type::Definition(
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
    let dx_type = ExampleEnum::datex_type_without_cache();
    println!("{}", dx_type);

    assert_eq!(
        dx_type,
        Type::Definition(
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
                            Type::Definition(
                                TypeDefinition::CoreType(
                                    CoreLibVariantTypeId::Integer(
                                        IntegerTypeVariant::U8
                                    )
                                    .into()
                                )
                                .into()
                            ),
                            Type::Definition(
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
                                Type::Definition(
                                    TypeDefinition::Literal(
                                        LiteralTypeDefinition::Text("x".into())
                                    )
                                    .into()
                                ),
                                Type::Definition(
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
                                Type::Definition(
                                    TypeDefinition::Literal(
                                        LiteralTypeDefinition::Text("y".into())
                                    )
                                    .into()
                                ),
                                Type::Definition(
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

#[test]
fn recursive_struct() {
    #[derive(Datex)]
    struct Node {
        next: Option<Box<Node>>,
    }
    let cache = &mut SharedReferencesCache::default();
    let ty = Node::datex_type(cache);
    ty.with_collapsed_type_definition(|ty_def| match ty_def {
        TypeDefinition::Map(map) => {
            let next_type = map
                .first()
                .expect("Expected 'next' field in map type definition")
                .1
                .clone();
            next_type.with_collapsed_type_definition(|next_ty_def| {
                match next_ty_def {
                    TypeDefinition::Union(union) => {
                        let option_type = union
                            .iter()
                            .find(|ty| **ty != Type::NULL)
                            .expect("Expected Option type in union");
                        assert_eq!(option_type, &ty);
                    }
                    _ => panic!(
                        "Expected Union type for Option, got {:?}",
                        next_ty_def
                    ),
                }
            })
        }
        _ => panic!("Expected map type definition"),
    });
}

#[test]
fn mutual_recursion() {
    #[derive(Datex)]
    #[datex(structural_recursive)]
    struct A {
        b: Box<B>,
    }

    #[derive(Datex)]
    #[datex(structural_recursive)]
    struct B {
        a: Box<A>,
    }
    let cache = &mut SharedReferencesCache::default();

    let ty_a = A::datex_type_without_cache();
    let ty_b = B::datex_type_without_cache();

    ty_a.with_collapsed_type_definition(|ty_def| match ty_def {
        TypeDefinition::Map(map) => {
            assert_eq!(map.first().expect("wtf").1, ty_b);
        }
        _ => panic!("Expected map type definition for A"),
    });
    ty_b.with_collapsed_type_definition(|ty_def| match ty_def {
        TypeDefinition::Map(map) => {
            assert_eq!(map.first().expect("wtf").1, ty_a);
        }
        _ => panic!("Expected map type definition for B"),
    });
}

#[test_case(None ; "none")]
#[test_case(Some(0u8) ; "some zero")]
#[test_case(Some(u8::MAX) ; "some max")]
fn round_trip_option(value: Option<u8>) {
    assert_round_trip(value);
}

#[test_case(None ; "outer none")]
#[test_case(Some(None) ; "inner none")]
#[test_case(Some(Some(42u8)) ; "some value")]
fn round_trip_nested_option(value: Option<Option<u8>>) {
    assert_round_trip(value);
}

#[test_case(Box::new(0u8) ; "boxed zero")]
#[test_case(Box::new(42u8) ; "boxed primitive")]
#[test_case(Box::new(u8::MAX) ; "boxed max")]
fn round_trip_box(value: Box<u8>) {
    assert_round_trip(value);
}

#[test_case(None ; "none")]
#[test_case(Some(Box::new(0u8)) ; "some boxed zero")]
#[test_case(Some(Box::new(42u8)) ; "some boxed value")]
fn round_trip_option_box(value: Option<Box<u8>>) {
    assert_round_trip(value);
}

#[test_case(Box::new(None) ; "boxed none")]
#[test_case(Box::new(Some(0u8)) ; "boxed some zero")]
#[test_case(Box::new(Some(42u8)) ; "boxed some value")]
fn round_trip_box_option(value: Box<Option<u8>>) {
    assert_round_trip(value);
}

#[test_case(None ; "outer none")]
#[test_case(Some(Box::new(None)) ; "boxed none")]
#[test_case(Some(Box::new(Some(42u8))) ; "boxed some value")]
fn round_trip_option_box_option(value: Option<Box<Option<u8>>>) {
    assert_round_trip(value);
}

#[test]
fn round_trip_boxed_struct() {
    let example = Box::new(Example {
        a: 42u8,
        b: "Test".to_string(),
        c: Endpoint::default(),
    });
    assert_round_trip(example);
}

#[test]
fn struct_with_option() {
    #[derive(Datex, Debug, Clone, PartialEq)]
    #[datex(structural_recursive)]
    struct ExampleWithOption {
        value: Option<u8>,
    }
    assert_round_trip(ExampleWithOption { value: None });
    assert_round_trip(ExampleWithOption { value: Some(42) });
}

#[test]
fn struct_with_box() {
    #[derive(Datex, Debug, Clone, PartialEq)]
    #[datex(structural_recursive)]
    struct ExampleWithBox {
        value: Box<u8>,
    }
    assert_round_trip(ExampleWithBox {
        value: Box::new(42),
    });
}

#[test]
fn struct_with_option_box() {
    #[derive(Datex, Debug, Clone, PartialEq)]
    #[datex(structural_recursive)]
    struct ExampleWithOptionBox {
        value: Option<Box<u8>>,
    }
    assert_round_trip(ExampleWithOptionBox { value: None });
    assert_round_trip(ExampleWithOptionBox {
        value: Some(Box::new(42)),
    });
}

#[test]
fn struct_with_box_option() {
    #[derive(Datex, Debug, Clone, PartialEq)]
    #[datex(structural_recursive)]
    struct ExampleWithBoxOption {
        value: Box<Option<u8>>,
    }
    assert_round_trip(ExampleWithBoxOption {
        value: Box::new(None),
    });
    assert_round_trip(ExampleWithBoxOption {
        value: Box::new(Some(42)),
    });
}

#[test]
fn struct_with_nested_option() {
    #[derive(Datex, Debug, Clone, PartialEq)]
    #[datex(structural_recursive)]
    struct ExampleWithNestedOption {
        value: Option<Option<u8>>,
    }
    assert_round_trip(ExampleWithNestedOption { value: None });
    assert_round_trip(ExampleWithNestedOption { value: Some(None) });
    assert_round_trip(ExampleWithNestedOption {
        value: Some(Some(42)),
    });
}

#[test_case(
    ExampleEnumWithOptionAndBox::Optional(None)
    ; "option none"
)]
#[test_case(
    ExampleEnumWithOptionAndBox::Optional(Some(42))
    ; "option some"
)]
#[test_case(
    ExampleEnumWithOptionAndBox::Boxed(Box::new(42))
    ; "boxed"
)]
#[test_case(
    ExampleEnumWithOptionAndBox::OptionalBoxed(None)
    ; "option boxed none"
)]
#[test_case(
    ExampleEnumWithOptionAndBox::OptionalBoxed(Some(Box::new(42)))
    ; "option boxed some"
)]
fn round_trip_enum_with_option_and_box(value: ExampleEnumWithOptionAndBox) {
    assert_round_trip(value);
}

#[derive(Datex, Debug, Clone, PartialEq)]
#[datex(structural_recursive)]
enum ExampleEnumWithOptionAndBox {
    Optional(Option<u8>),
    Boxed(Box<u8>),
    OptionalBoxed(Option<Box<u8>>),
}

#[test]
fn recursive_struct_round_trip() {
    #[derive(Datex, Debug, Clone, PartialEq)]
    #[datex(structural_recursive)]
    struct Node {
        value: u8,
        next: Option<Box<Node>>,
    }

    let node = Node {
        value: 42,
        next: Some(Box::new(Node {
            value: 69,
            next: Some(Box::new(Node {
                value: 10,
                next: None,
            })),
        })),
    };
    assert_round_trip(node);
}

#[test]
fn vec_with_option_box() {
    let value = vec![None, Some(Box::new(0u8)), Some(Box::new(42u8)), None];
    assert_round_trip(value);
}
