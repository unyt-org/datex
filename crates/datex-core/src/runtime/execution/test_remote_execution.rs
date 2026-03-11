use crate::{
    runtime::{
        execution::context::{ExecutionContext, ExecutionMode},
        test_utils::use_mock_setup_with_two_connected_runtimes,
    },
    values::{
        core_values::{endpoint::Endpoint, integer::Integer},
        value_container::ValueContainer,
    },
};
use crate::runtime::execution::execution_input::ExecutionCallerMetadata;

#[tokio::test]
#[cfg(feature = "compiler")]
pub async fn test_basic_remote_execution() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, runtime_b| {
            runtime_a.com_hub().print_metadata();
            runtime_b.com_hub().print_metadata();

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
        },
    )
    .await;
}

#[tokio::test]
#[cfg(feature = "compiler")]
pub async fn test_remote_execution_persistent_context() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, _runtime_b| {
            // create an execution context for @test_b
            let mut remote_execution_context =
                ExecutionContext::remote_unbounded(endpoint_b);

            // execute script remotely on @test_b
            let result = runtime_a
                .execute(
                    "const x = 10; x",
                    &[],
                    Some(&mut remote_execution_context),
                )
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
        },
    )
    .await;
}

#[tokio::test]
#[cfg(feature = "compiler")]
pub async fn test_remote_inline() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, _runtime_b| {
            // create an execution context for @test_b
            let mut execution_context = ExecutionContext::local(
                ExecutionMode::unbounded(),
                runtime_a.internal.clone(),
                ExecutionCallerMetadata::local_default(),
            );

            // execute script remotely on @test_b
            let result = runtime_a
                .execute("@test_b :: 1 + 2", &[], Some(&mut execution_context))
                .await;
            assert_eq!(
                result.unwrap().unwrap(),
                ValueContainer::from(Integer::from(3i8))
            );
        },
    )
    .await;
}

#[tokio::test]
#[cfg(feature = "compiler")]
pub async fn test_remote_inline_implicit_context() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, _runtime_b| {
            // execute script remotely on @test_b
            let result = runtime_a.execute("@test_b :: 1 + 2", &[], None).await;
            assert_eq!(
                result.unwrap().unwrap(),
                ValueContainer::from(Integer::from(3i8))
            );
        },
    )
    .await;
}

#[tokio::test]
#[cfg(feature = "compiler")]
pub async fn test_remote_shared_value() {
    flexi_logger::init();
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, _runtime_b| {
            // execute script remotely on @test_b
            let result = runtime_a
                .execute("var x = shared 42u8; @test_b :: x", &[], None)
                .await;
            assert_eq!(
                result.unwrap().unwrap(),
                ValueContainer::from(Integer::from(42u8))
            );
        },
    )
    .await;
}
