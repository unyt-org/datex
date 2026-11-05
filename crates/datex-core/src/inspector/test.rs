use crate::{
    inspector::datex_inspector::Inspector,
    runtime::{
        execution::context::ExecutionContext,
        test_utils::use_mock_setup_with_two_connected_runtimes,
    },
    shared_values::traits::SharedContainerCommon,
    traits::{
        classification::Classification,
        convert_value_container::ConvertValueContainer,
        get_datex_type::GetDatexType, value_access::ValueAccess,
    },
    types::r#type::Type,
    values::{
        core_values::{endpoint::Endpoint, integer::Integer, map::Map},
        value_container::ValueContainer,
    },
};
use core::ops::{Deref, DerefMut};

#[tokio::test]
#[cfg(all(
    feature = "compiler",
    any(feature = "crypto_enabled", feature = "allow_unsigned_blocks")
))]
pub async fn inspector_instantiation() {
    flexi_logger::init();
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, runtime_b| {
            // execute @test_b.inspector.create
            let result = runtime_a
                .execute(
                    "@test_b.inspector.create(\"Test Inspector\")",
                    &[],
                    None,
                )
                .await;

            match result.unwrap().unwrap() {
                ValueContainer::Shared(value) => {
                    // should be a shared value owned by @test_b
                    assert_eq!(value.pointer_address().endpoint(), endpoint_b);

                    // inner value should be an "Inspector" with correct entity type
                    let inner_value_container = value.value_container();
                    let mut cache = runtime_a.shared_references_cache_mut();
                    let inspector_entity_type =
                        match Inspector::datex_type(cache.deref_mut()) {
                            Type::Entity(entity_type) => entity_type,
                            _ => panic!("Expected an entity type"),
                        };
                    assert_eq!(
                        inner_value_container
                            .entity_type(cache.deref_mut())
                            .unwrap(),
                        inspector_entity_type
                    );

                    // property name should be accessible and correct
                    let inner_map =
                        inner_value_container.try_as::<Map>().unwrap();
                    let property_name = inner_map
                        .try_get_property(&ValueContainer::from("name"))
                        .unwrap();
                    assert_eq!(
                        property_name.try_as::<str>().unwrap().deref(),
                        "Test Inspector"
                    );
                }
                _ => panic!("Expected a shared value"),
            };
        },
    )
    .await;
}
