use datex_core::stdlib::env;

use datex_core::{
    runtime::{RuntimeConfig},
    values::core_values::endpoint::Endpoint,
};
use datex_core::native_global_context::get_global_context_native;
use datex_core::runtime::RuntimeRunner;

/**
 * test if the DATEX Runtime is initialized correctly
 */
#[tokio::test]
pub async fn init_runtime() {
    RuntimeRunner::new(RuntimeConfig::new_with_endpoint(Endpoint::new("@test")), get_global_context_native())
        .run(async |runtime| {
            assert_eq!(runtime.version, env!("CARGO_PKG_VERSION"));
        }).await;
}
