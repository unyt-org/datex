use crate::{
    runtime::{
        execution::{
            context::{ExecutionContext, ExecutionMode},
            execution_input::ExecutionCallerMetadata,
        },
        test_utils::use_mock_setup_with_two_connected_runtimes,
    },
    shared_values::{
        PointerAddress, SharedContainer, SharedContainerMutability,
        base_shared_value_container::observers::TransceiverId,
        traits::SharedContainerCommon,
    },
    task::sleep,
    value_updates::{
        update_data::{ReplaceUpdateData, Update, UpdateData},
        update_handler::UpdateHandler,
    },
    values::{
        core_values::{
            endpoint::Endpoint, integer::Integer, list::List, time::Instant,
        },
        value_container::ValueContainer,
    },
};
use core::{assert_matches, ops::DerefMut, time::Duration};
use log::info;

#[tokio::test]
#[cfg(all(
    feature = "compiler",
    any(feature = "crypto_enabled", feature = "allow_unsigned_blocks")
))]
pub async fn basic_remote_execution() {
    flexi_logger::init();
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
                ExecutionContext::remote_unbounded(vec![endpoint_b], runtime_b);

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
#[cfg(all(
    feature = "compiler",
    any(feature = "crypto_enabled", feature = "allow_unsigned_blocks")
))]
pub async fn remote_execution_persistent_context() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, runtime_b| {
            // create an execution context for @test_b
            let mut remote_execution_context =
                ExecutionContext::remote_unbounded(vec![endpoint_b], runtime_b);

            // execute script remotely on @test_b
            let result = runtime_a
                .execute(
                    "const x = 10; clone x", // FIXME: auto copy for integer?
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
#[cfg(all(
    feature = "compiler",
    any(feature = "crypto_enabled", feature = "allow_unsigned_blocks")
))]
pub async fn remote_inline() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, _runtime_b| {
            // create an execution context for @test_b
            let mut execution_context = ExecutionContext::local(
                ExecutionMode::unbounded(),
                runtime_a.clone(),
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
#[cfg(all(
    feature = "compiler",
    any(feature = "crypto_enabled", feature = "allow_unsigned_blocks")
))]
pub async fn remote_inline_implicit_context() {
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
#[cfg(all(
    feature = "compiler",
    any(feature = "crypto_enabled", feature = "allow_unsigned_blocks")
))]
pub async fn remote_shared_value_inject_move() {
    flexi_logger::init();
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, _runtime_b| {
            // execute script remotely on @test_b
            let result = runtime_a
                .execute("var x = shared 42; @test_b :: x + 1", &[], None)
                .await;
            assert_eq!(
                result.unwrap().unwrap(),
                ValueContainer::from(Integer::from(43))
            );
        },
    )
    .await;
}

#[tokio::test]
#[cfg(all(
    feature = "compiler",
    any(feature = "crypto_enabled", feature = "allow_unsigned_blocks")
))]
pub async fn remote_shared_value_inject_ref() {
    flexi_logger::init();
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, _runtime_b| {
            // execute script remotely on @test_b
            let result = runtime_a
                .execute(
                    "var x = shared 42; @test_b :: ['x + 1, 'x]",
                    &[],
                    None,
                )
                .await
                .unwrap()
                .unwrap();

            let result_list = result.try_into_value::<List>().unwrap();
            let result_vec = result_list.as_vec();

            // 'x + 1
            assert_eq!(result_vec[0], ValueContainer::from(Integer::from(43)));

            // 'x
            if let ValueContainer::Shared(shared_container) = &result_vec[1] {
                assert_matches!(
                    shared_container,
                    SharedContainer::Referenced(..)
                );
                assert_matches!(
                    shared_container.pointer_address(),
                    PointerAddress::SelfOwned(..)
                );
                assert_eq!(
                    *shared_container
                        .inner()
                        .base_shared_container()
                        .mutability(),
                    SharedContainerMutability::Immutable
                );
                assert_eq!(
                    *shared_container.value_container(),
                    ValueContainer::from(Integer::from(42))
                )
            } else {
                panic!("Expected SharedContainer");
            }
        },
    )
    .await;
}

#[cfg(all(
    feature = "compiler",
    any(feature = "crypto_enabled", feature = "allow_unsigned_blocks")
))]
#[test_case::test_case("shared", SharedContainerMutability::Immutable ; "immutable")]
#[test_case::test_case("shared mut", SharedContainerMutability::Mutable ; "mutable")]
#[tokio::test]
pub async fn remote_shared_value_return(
    shared_string: &'static str,
    mutable_value: SharedContainerMutability,
) {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, _runtime_b| {
            // execute script remotely on @test_b
            let result = runtime_a
                .execute(&format!("@test_b :: ({shared_string} 42)"), &[], None)
                .await
                .unwrap()
                .unwrap();

            if let ValueContainer::Shared(shared_container) = result {
                shared_container
                    .try_get_owned()
                    .expect("shared container should be owned");
                assert_matches!(
                    shared_container.pointer_address(),
                    PointerAddress::SelfOwned(..)
                );
                assert_eq!(
                    *shared_container
                        .inner()
                        .base_shared_container()
                        .mutability(),
                    mutable_value
                );
                assert_eq!(
                    *shared_container.value_container(),
                    ValueContainer::from(Integer::from(42))
                )
            } else {
                panic!("Expected SharedContainer");
            }
        },
    )
    .await;
}

#[cfg(all(
    feature = "compiler",
    any(feature = "crypto_enabled", feature = "allow_unsigned_blocks")
))]
#[test_case::test_case("shared", SharedContainerMutability::Immutable ; "immutable")]
#[test_case::test_case("shared mut", SharedContainerMutability::Mutable; "mutable")]
#[tokio::test]
pub async fn remote_shared_roundtrip_move(
    shared_string: &'static str,
    mutable_value: SharedContainerMutability,
) {
    flexi_logger::init();
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, _runtime_b| {
            // execute script remotely on @test_b
            let result = runtime_a
                .execute(
                    &format!(
                        "const x = {shared_string} 42; @test_b :: (print 'x; x)"
                    ),
                    &[],
                    None,
                )
                .await
                .unwrap()
                .unwrap();
            if let ValueContainer::Shared(shared_container) = result {
                shared_container
                    .try_get_owned()
                    .expect("shared container should be owned");
                assert_matches!(
                    shared_container.pointer_address(),
                    PointerAddress::SelfOwned(..)
                );
                assert_eq!(
                    *shared_container
                        .inner()
                        .base_shared_container()
                        .mutability(),
                    mutable_value
                );
                assert_eq!(
                    *shared_container.value_container(),
                    ValueContainer::from(Integer::from(42))
                )
            } else {
                panic!("Expected SharedContainer");
            }
        },
    )
    .await;
}

#[tokio::test]
#[cfg(all(
    feature = "compiler",
    any(feature = "crypto_enabled", feature = "allow_unsigned_blocks")
))]
pub async fn test_remote_datetime_literal() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, _runtime_b| {
            let mut execution_context = ExecutionContext::local(
                ExecutionMode::unbounded(),
                runtime_a.clone(),
                ExecutionCallerMetadata::local_default(),
            );

            let result = runtime_a
                .execute(
                    "@test_b :: 2026-04-13T18:28:09.415Z",
                    &[],
                    Some(&mut execution_context),
                )
                .await
                .unwrap()
                .unwrap();

            let expected =
                Instant::instant_from_iso("2026-04-13T18:28:09.415Z");
            assert_eq!(result, ValueContainer::from(Integer::new(expected.0)));
        },
    )
    .await;
}

#[tokio::test]
#[cfg(all(
    feature = "compiler",
    any(feature = "crypto_enabled", feature = "allow_unsigned_blocks")
))]
pub async fn test_remote_datetime_arithmetic() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, _runtime_b| {
            let mut execution_context = ExecutionContext::local(
                ExecutionMode::unbounded(),
                runtime_a.clone(),
                ExecutionCallerMetadata::local_default(),
            );

            let result = runtime_a
                .execute(
                    "@test_b :: 2026-04-13T18:28:09.415Z + 1000",
                    &[],
                    Some(&mut execution_context),
                )
                .await
                .unwrap()
                .unwrap();

            let expected =
                Instant::instant_from_iso("2026-04-13T18:28:10.415Z");
            assert_eq!(result, ValueContainer::from(Integer::new(expected.0)));
        },
    )
    .await;
}

#[tokio::test]
#[cfg(all(
    feature = "compiler",
    any(feature = "crypto_enabled", feature = "allow_unsigned_blocks")
))]
pub async fn test_remote_sync() {
    let endpoint_a = Endpoint::new("@test_a");
    let endpoint_b = Endpoint::new("@test_b");

    flexi_logger::init();
    use_mock_setup_with_two_connected_runtimes(
        endpoint_a.clone(),
        endpoint_b.clone(),
        async |runtime_a, runtime_b| {
            let mut execution_context = ExecutionContext::local(
                ExecutionMode::unbounded(),
                runtime_a.clone(),
                ExecutionCallerMetadata::local_default(),
            );

            let shared_value =
                SharedContainer::new_owned_with_inferred_allowed_type(
                    42,
                    SharedContainerMutability::Mutable,
                    runtime_a.pointer_address_provider_mut().deref_mut(),
                );

            let reference =
                ValueContainer::Shared(SharedContainer::Referenced(
                    shared_value.derive_immutable_reference(),
                ));

            let result = runtime_a
                .execute(
                    "@test_b :: (@@local.a = '?; @@local.a)",
                    core::slice::from_ref(&reference),
                    Some(&mut execution_context),
                )
                .await
                .unwrap()
                .unwrap();

            assert_eq!(
                match result {
                    ValueContainer::Shared(shared) => shared,
                    _ => unreachable!(),
                },
                shared_value
            );

            shared_value
                .base_shared_container_mut()
                .update(Update::new(
                    TransceiverId(0),
                    UpdateData::Replace(ReplaceUpdateData {
                        value: ValueContainer::from(100),
                    }),
                ))
                .unwrap();

            sleep(Duration::from_millis(100)).await;

            let shared_value_on_b =
                runtime_b.get_endpoint_property_by_name("a").unwrap();
            let shared_value_on_b = shared_value_on_b.shared_unchecked();
            assert_eq!(
                shared_value_on_b.pointer_address().normalize(&endpoint_a),
                shared_value.pointer_address()
            );
            println!("val: {:?}", shared_value_on_b);
        },
    )
    .await;
}
