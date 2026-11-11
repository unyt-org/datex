use datex_core::{
    runtime::{RuntimeConfig, RuntimeRunner},
    values::core_values::endpoint::Endpoint,
};
use datex_core::runtime::CustomRuntimeInfo;

/**
 * test if the DATEX Runtime is initialized correctly
 */
#[tokio::test]
pub async fn init_runtime() {
    RuntimeRunner::new(RuntimeConfig::new_with_endpoint(Endpoint::new(
        "@test",
    )), CustomRuntimeInfo::default())
    .run(async |runtime| {
        assert_eq!(runtime.version(), env!("CARGO_PKG_VERSION"));
    })
    .await;
}
