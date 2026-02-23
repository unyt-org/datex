use alloc::rc::Rc;
use core::cell::Cell;
use crate::network::com_hub::test_utils::couple_com_hubs;
use crate::runtime::{Runtime, RuntimeConfig, RuntimeRunner};
use crate::values::core_values::endpoint::Endpoint;

/// Helper function to set up two connected runtimes with the provided endpoints and run the provided application logic
pub async fn use_mock_setup_with_two_connected_runtimes<AppReturn, AppFuture>(
    endpoint_a: Endpoint,
    endpoint_b: Endpoint,
    app_logic: impl FnOnce(Runtime, Runtime) -> AppFuture,
) where
    AppFuture: Future<Output = AppReturn> {
    let runner_a = RuntimeRunner::new(RuntimeConfig::new_with_endpoint(endpoint_a.clone()));
    let runner_b = RuntimeRunner::new(RuntimeConfig::new_with_endpoint(endpoint_b.clone()));

    let end_reached = Rc::new(Cell::new(false));
    let end_reached_clone = end_reached.clone();

    runner_a.run(async move |runtime_a| {
        runner_b.run(async move |runtime_b| {
            // couple the com hubs of the two runtimes
            couple_com_hubs(runtime_a.com_hub(), runtime_b.com_hub()).await;
            // run the provided application logic
            app_logic(runtime_a, runtime_b).await;
            // mark the end of the app logic
            end_reached_clone.set(true);
        }).await;
    }).await;

    // make sure the app logic was actually executed and finished
    assert!(end_reached.get(), "The provided app logic did not finish executing");
}