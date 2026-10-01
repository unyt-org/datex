use datex_core::dif::serde_context::SerdeContext;
use datex_core::runtime::cache::shared_values_cache::SharedValuesCache;
use datex_core::values::core_values::endpoint::Endpoint;
use datex_macros_internal::Datex;

#[derive(Datex, Debug, Clone, PartialEq)]
#[datex(structural)]
struct Example {
    a: u8,
    b: String,
    c: Endpoint,
}

#[test]
fn struct_to_dif() {
    let example = Example {
        a: 42,
        b: "Hello, Datex!".to_string(),
        c: Endpoint::new("@jonas"),
    };
    let serialized =
        SerdeContext::<Example>::new(&mut SharedValuesCache::default())
            .serialize_to_json(&example);
    
    panic!("{:?}", serialized);
}