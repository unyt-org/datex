use datex_core::stdlib::env;

use datex_core::{
    runtime::{Runtime, RuntimeConfig},
    values::core_values::endpoint::Endpoint,
};
use datex_macros::async_test;

/**
 * test if the DATEX Runtime is initialized correctly
 */
#[async_test]
pub async fn init_runtime() {
    let runtime = Runtime::init_native(RuntimeConfig::new_with_endpoint(
        Endpoint::new("@test"),
    ));
    assert_eq!(runtime.version, env!("CARGO_PKG_VERSION"));
}
