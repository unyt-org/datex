use datex_core::run_async;
use datex_core::stdlib::env;

use datex_core::runtime::{Runtime, RuntimeConfig};
use datex_core::values::core_values::endpoint::Endpoint;

/**
 * test if the DATEX Runtime is initialized correctly
 */
#[tokio::test]
pub async fn init_runtime() {
    run_async! {
        let runtime = Runtime::init_native(RuntimeConfig::new_with_endpoint(
            Endpoint::new("@test"),
        ));
        assert_eq!(runtime.version, env!("CARGO_PKG_VERSION"));
    }
}