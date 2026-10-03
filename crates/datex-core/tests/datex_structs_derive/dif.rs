use core::cell::RefCell;
use datex_core::dif::serde_context::SerdeContext;
use datex_core::runtime::cache::shared_values_cache::SharedValuesCache;
use datex_core::values::core_values::endpoint::Endpoint;
use datex_macros_internal::Datex;
use datex_core::{prelude::*};
use datex_core::utils::serde_with_context::SerializeWithSerdeContext;

#[derive(Datex, Debug, Clone, PartialEq)]
#[datex(structural)]
struct Example {
    a: u8,
    b: String,
    c: Endpoint,
}


/// TODO: general helper function in datex_core
fn serialize_to_json(
    context: &SerdeContext,
    value: &impl SerializeWithSerdeContext,
) -> String {
    let mut serializer = serde_json::Serializer::new(Vec::new());
    value.serialize_with_ctx(context, &mut serializer).unwrap();
    let bytes = serializer.into_inner();
    String::from_utf8(bytes).unwrap()
}

/// Generates a serialized DIF JSON string from a value using a default SerdeContext and SharedValuesCache.
fn serialize_to_json_default_ctx(value: &impl SerializeWithSerdeContext) -> String {
    let cache = RefCell::new(SharedValuesCache::default());
    let context = SerdeContext::new(&cache);
    serialize_to_json(&context, value)
}

#[test]
fn struct_to_dif() {
    let example = Example {
        a: 42,
        b: "Hello, Datex!".to_string(),
        c: Endpoint::new("@jonas"),
    };
    let serialized =
        serialize_to_json_default_ctx(&example);

    panic!("{:?}", serialized);
}