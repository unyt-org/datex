use datex::{
    runtime::{RuntimeConfig, RuntimeRunner},
    values::core_values::endpoint::Endpoint,
};

/**
 * test if the DATEX Runtime is initialized correctly
 */
#[tokio::test]
pub async fn init_runtime() {
    RuntimeRunner::new(RuntimeConfig::new_with_endpoint(Endpoint::new(
        "@test",
    )))
    .run(async |runtime| {
        assert_eq!(runtime.version, env!("CARGO_PKG_VERSION"));
    })
    .await;
}
