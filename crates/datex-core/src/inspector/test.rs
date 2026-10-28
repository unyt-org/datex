use crate::inspector::datex_inspector::Inspector;
use crate::runtime::execution::context::ExecutionContext;
use crate::runtime::test_utils::use_mock_setup_with_two_connected_runtimes;
use crate::shared_values::traits::SharedContainerCommon;
use crate::traits::convert_value_container::ConvertValueContainer;
use crate::values::core_values::endpoint::Endpoint;
use crate::values::core_values::integer::Integer;
use crate::values::value_container::ValueContainer;

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
                .execute("@test_b.inspector.create(\"Test Inspector\")", &[], None)
                .await;

            match result.unwrap().unwrap() {
                ValueContainer::Shared(value) => {
                    // should be a shared value owned by @test_b
                    assert_eq!(value.pointer_address().endpoint(), endpoint_b);

                    // inner value should be an Inspector instance
                    let inner_value = value.value_container();
                    match inner_value.try_as::<Inspector>() {
                        Some(inspector) => {
                            assert_eq!(inspector, &Inspector {
                                name: "Test Inspector".to_string(),
                            });
                        },
                        None => panic!("Expected a shared value of type Inspector, got {:#?}", inner_value),
                    }
                },
                _ => panic!("Expected a shared value"),
            };
        }
    )
        .await;
}