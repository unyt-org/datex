use datex_core::{
    macro_utils::datex_proxy::DatexProxy,
    prelude::*,
    values::{
        core_values::{endpoint::Endpoint, map::Map},
        value_container::ValueContainer,
    },
};
use datex_macros_internal::Datex;
use serde::{Deserialize, Serialize};

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
    serde: SerdeExample,
}

fn assert_round_trip<T>(value: T)
where
    T: DatexProxy + PartialEq + std::fmt::Debug + Clone,
{
    let value_container = value.clone().datex_to_value_container().unwrap();
    let deserialized_value =
        T::datex_from_value_container(value_container).unwrap();
    assert_eq!(value, deserialized_value);
}

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
// #[case(Map::from(vec![
//     ("key1".to_string(), ValueContainer::from(42u8)),
//     ("key2".to_string(), ValueContainer::from("Value".to_string())),
// ]) ; "map of primitives")]
fn round_trip_struct<T>(structure: T)
where
    T: DatexProxy + PartialEq + std::fmt::Debug + Clone,
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
    .try_into()
    .unwrap();

    let map: Map = value_container.try_as().unwrap();
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
fn struct_with_serde_to_value_container() {
    let serde_example = SerdeDatexExample {
        a: 42u8,
        serde: SerdeExample {
            inner_a: 1,
            inner_b: "Inner".to_string(),
        },
    };

    let value_container: ValueContainer = serde_example.try_into().unwrap();

    let map: Map = value_container.try_as().unwrap();
    assert_eq!(map.get("a").unwrap(), &ValueContainer::from(42u8));
    let serde_map: Map = map.get("serde").unwrap().try_as().unwrap();
    assert_eq!(serde_map.get("inner_a").unwrap(), &ValueContainer::from(1u8));
    assert_eq!(
        serde_map.get("inner_b").unwrap(),
        &ValueContainer::from("Inner".to_string())
    );
}
