use serde::{Deserialize, Serialize};
use datex_core::values::core_values::endpoint::Endpoint;
use datex_core::values::value_container::ValueContainer;
use datex_core::prelude::*;
use datex_core::values::core_values::map::Map;
use datex_macros_internal::Datex;

#[derive(Datex, Debug)]
struct Example {
    a: u8,
    b: String,
    c: Endpoint,
}

#[derive(Serialize, Deserialize)]
struct SerdeExample {
    a: u8,
    b: String,
}

#[derive(Datex)]
struct SerdeDatexExample {
    a: u8,
    serde: SerdeExample,
}

#[test]
fn struct_to_value_container() {
    let value_container: ValueContainer = Example {
        a: 42u8,
        b: "Test".to_string(),
        c: Endpoint::default(),
    }.try_into().unwrap();

    println!("{}", value_container);

    let map: Map = value_container.try_as().unwrap();
    assert_eq!(map.get("a").unwrap(), &ValueContainer::from(42u8));
    assert_eq!(map.get("b").unwrap(), &ValueContainer::from("Test".to_string()));
    assert_eq!(map.get("c").unwrap(), &ValueContainer::from(Endpoint::default()));
}

#[test]
fn value_container_to_struct() {
    let value_container: ValueContainer = ValueContainer::from(Map::from(vec![
        ("a".to_string(), ValueContainer::from(42u8)),
        ("b".to_string(), ValueContainer::from("Test".to_string())),
        ("c".to_string(), ValueContainer::from(Endpoint::default())),
    ]));

    let example: Example = value_container.try_into().unwrap();
    
    assert_eq!(example.a, 42u8);
    assert_eq!(example.b, "Test".to_string());
    assert_eq!(example.c, Endpoint::default());

    println!("{:#?}", example);
}

#[test]
fn struct_with_serde_to_value_container() {
    let serde_example = SerdeDatexExample {
        a: 42u8,
        serde: SerdeExample {
            a: 1,
            b: "Inner".to_string(),
        },
    };
    
    let value_container: ValueContainer = serde_example.try_into().unwrap();
    println!("{}", value_container);
}