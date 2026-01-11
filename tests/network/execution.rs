use crate::network::helpers::mock_setup::get_mock_setup_with_two_runtimes;
use core::time::Duration;
use datex_core::{
    logger::init_logger_debug,
    runtime::execution::context::{ExecutionContext, ExecutionMode},
    values::{
        core_values::{endpoint::Endpoint, integer::Integer},
        value_container::ValueContainer,
    },
};
use datex_macros::async_test;

#[async_test]
pub async fn test_basic_remote_execution() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");
    let (runtime_a, runtime_b) = get_mock_setup_with_two_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
    )
    .await;

    // sleep for a short time to ensure the connection is established
    tokio::time::sleep(Duration::from_millis(1)).await;
    runtime_a.com_hub().print_metadata();

    // create an execution context for @test_b
    let mut remote_execution_context =
        ExecutionContext::remote_unbounded(endpoint_b);

    // execute script remotely on @test_b
    let result = runtime_a
        .execute("1 + 2", &[], Some(&mut remote_execution_context))
        .await;
    assert_eq!(
        result.unwrap().unwrap(),
        ValueContainer::from(Integer::from(3i8))
    );

    let result = runtime_a
        .execute("2 + 3", &[], Some(&mut remote_execution_context))
        .await;
    assert_eq!(
        result.unwrap().unwrap(),
        ValueContainer::from(Integer::from(5i8))
    );
}

#[async_test]
pub async fn test_remote_execution_persistent_context() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");
    let (runtime_a, runtime_b) = get_mock_setup_with_two_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
    )
    .await;

    // sleep for a short time to ensure the connection is established
    tokio::time::sleep(Duration::from_millis(1)).await;
    runtime_a.com_hub().print_metadata();

    // create an execution context for @test_b
    let mut remote_execution_context =
        ExecutionContext::remote_unbounded(endpoint_b);

    // execute script remotely on @test_b
    let result = runtime_a
        .execute("const x = 10; x", &[], Some(&mut remote_execution_context))
        .await;
    assert_eq!(
        result.unwrap().unwrap(),
        ValueContainer::from(Integer::from(10i8))
    );

    // execute another script that uses the previous context
    let result = runtime_a
        .execute("x + 5", &[], Some(&mut remote_execution_context))
        .await;
    assert_eq!(
        result.unwrap().unwrap(),
        ValueContainer::from(Integer::from(15i8))
    );
}

#[async_test]
pub async fn test_remote_inline() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");
    let (runtime_a, runtime_b) = get_mock_setup_with_two_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
    )
    .await;

    // sleep for a short time to ensure the connection is established
    tokio::time::sleep(Duration::from_millis(1)).await;
    runtime_a.com_hub().print_metadata();

    // create an execution context for @test_b
    let mut execution_context = ExecutionContext::local_with_runtime_internal(
        runtime_a.internal.clone(),
        ExecutionMode::unbounded(),
    );

    // execute script remotely on @test_b
    let result = runtime_a
        .execute("@test_b :: 1 + 2", &[], Some(&mut execution_context))
        .await;
    assert_eq!(
        result.unwrap().unwrap(),
        ValueContainer::from(Integer::from(3i8))
    );
}

#[async_test]
pub async fn test_remote_inline_implicit_context() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");
    let (runtime_a, runtime_b) = get_mock_setup_with_two_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
    )
    .await;

    // sleep for a short time to ensure the connection is established
    tokio::time::sleep(Duration::from_millis(1)).await;
    runtime_a.com_hub().print_metadata();

    // execute script remotely on @test_b
    let result = runtime_a.execute("@test_b :: 1 + 2", &[], None).await;
    assert_eq!(
        result.unwrap().unwrap(),
        ValueContainer::from(Integer::from(3i8))
    );
}
