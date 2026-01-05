use crate::network::helpers::{
    mock_setup::{
        TEST_ENDPOINT_A, TEST_ENDPOINT_B, TEST_ENDPOINT_C, TEST_ENDPOINT_D,
    },
    network::{
        InterfaceConnection, MockupInterfaceSetupData, Network, Node, Route,
        RouteAssertionError, test_routes,
    },
};
use core::{str::FromStr, time::Duration};
use datex_core::{
    native_global_context::init_global_context_native,
    network::com_hub::{
        InterfacePriority, network_response::ResponseOptions,
        network_tracing::TraceOptions,
    },
    values::core_values::endpoint::Endpoint,
};
use datex_macros_internal::async_test;
use log::info;
use ntest_timeout::timeout;
use tokio::task;

#[tokio::test]
#[timeout(2000)]
async fn create_network_with_two_nodes() {
    let local = task::LocalSet::new();
    local
        .run_until(async {
            let mut network = Network::new(vec![
                // @test-a
                Node::new(TEST_ENDPOINT_A.clone()).with_connection(
                    InterfaceConnection::new(
                        InterfacePriority::default(),
                        MockupInterfaceSetupData::new("ab"),
                    ),
                ),
                // @test-b
                Node::new(TEST_ENDPOINT_B.clone()).with_connection(
                    InterfaceConnection::new(
                        InterfacePriority::default(),
                        MockupInterfaceSetupData::new("ab"),
                    ),
                ),
            ]);

            network.start().await;

            // sleep 100ms
            tokio::time::sleep(Duration::from_millis(10)).await;

            info!("Network started");

            let runtime_a = network.get_runtime(TEST_ENDPOINT_A.clone());
            let runtime_b = network.get_runtime(TEST_ENDPOINT_B.clone());

            // send trace from A to B
            let network_trace = runtime_a
                .com_hub()
                .record_trace(TEST_ENDPOINT_B.clone())
                .await;
            assert!(network_trace.is_some());
            info!("Network trace:\n{}", network_trace.as_ref().unwrap());
            network_trace.unwrap().assert_matches_hops(&[
                (TEST_ENDPOINT_A.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_A.clone(), "mockup"),
            ]);

            // send trace from B to A
            let network_trace = runtime_b
                .com_hub()
                .record_trace(TEST_ENDPOINT_A.clone())
                .await;
            assert!(network_trace.is_some());
            info!("Network trace:\n{}", network_trace.as_ref().unwrap());
            network_trace.unwrap().assert_matches_hops(&[
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_A.clone(), "mockup"),
                (TEST_ENDPOINT_A.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
            ]);

            // send trace from A to A
            let network_trace = runtime_a
                .com_hub()
                .record_trace(TEST_ENDPOINT_A.clone())
                .await;
            assert!(network_trace.is_some());
            info!("Network trace:\n{}", network_trace.as_ref().unwrap());
            network_trace.unwrap().assert_matches_hops(&[
                (TEST_ENDPOINT_A.clone(), "local"),
                (TEST_ENDPOINT_A.clone(), "local"),
                (TEST_ENDPOINT_A.clone(), "local"),
                (TEST_ENDPOINT_A.clone(), "local"),
            ]);
        })
        .await;
}

async fn get_test_network_1() -> Network {
    let mut network = Network::new(vec![
        // @test-a
        Node::new(TEST_ENDPOINT_A.clone()).with_connection(
            InterfaceConnection::new(
                InterfacePriority::default(),
                MockupInterfaceSetupData::new("ab"),
            ),
        ),
        // @test-b
        Node::new(TEST_ENDPOINT_B.clone())
            .with_connection(InterfaceConnection::new(
                InterfacePriority::default(),
                MockupInterfaceSetupData::new("ab"),
            ))
            .with_connection(InterfaceConnection::new(
                InterfacePriority::default(),
                MockupInterfaceSetupData::new("bc"),
            )),
        // @test-c
        Node::new(TEST_ENDPOINT_C.clone())
            .with_connection(InterfaceConnection::new(
                InterfacePriority::default(),
                MockupInterfaceSetupData::new("bc"),
            ))
            .with_connection(InterfaceConnection::new(
                InterfacePriority::default(),
                MockupInterfaceSetupData::new("cd"),
            )),
        // @test-d
        Node::new(TEST_ENDPOINT_D.clone()).with_connection(
            InterfaceConnection::new(
                InterfacePriority::default(),
                MockupInterfaceSetupData::new("cd"),
            ),
        ),
    ]);

    network.start().await;
    network
}

async fn get_test_network_1_with_deterministic_priorities() -> Network {
    let mut network = Network::new(vec![
        // @test-a
        Node::new(TEST_ENDPOINT_A.clone()).with_connection(
            InterfaceConnection::new(
                InterfacePriority::Priority(0),
                MockupInterfaceSetupData::new("ab"),
            ),
        ),
        // @test-b
        Node::new(TEST_ENDPOINT_B.clone())
            .with_connection(InterfaceConnection::new(
                InterfacePriority::Priority(0),
                MockupInterfaceSetupData::new("ab"),
            ))
            .with_connection(InterfaceConnection::new(
                InterfacePriority::Priority(1),
                MockupInterfaceSetupData::new("bc"),
            )),
        // @test-c
        Node::new(TEST_ENDPOINT_C.clone())
            .with_connection(InterfaceConnection::new(
                InterfacePriority::Priority(0),
                MockupInterfaceSetupData::new("bc"),
            ))
            .with_connection(InterfaceConnection::new(
                InterfacePriority::Priority(1),
                MockupInterfaceSetupData::new("cd"),
            )),
        // @test-d
        Node::new(TEST_ENDPOINT_D.clone()).with_connection(
            InterfaceConnection::new(
                InterfacePriority::Priority(0),
                MockupInterfaceSetupData::new("cd"),
            ),
        ),
    ]);

    network.start().await;
    network
}

#[tokio::test]
#[timeout(2000)]
async fn network_routing_with_four_nodes_1() {
    let local = task::LocalSet::new();
    local
        .run_until(async {
            let network = get_test_network_1().await;

            // sleep 100ms
            tokio::time::sleep(Duration::from_millis(20)).await;

            info!("Network started");

            for endpoint in network.endpoints.iter() {
                if let Some(runtime) = &endpoint.runtime {
                    runtime.com_hub().print_metadata();
                }
            }

            let runtime_a = network.get_runtime(TEST_ENDPOINT_A.clone());
            let runtime_b = network.get_runtime(TEST_ENDPOINT_B.clone());
            let runtime_c = network.get_runtime(TEST_ENDPOINT_C.clone());

            // send trace from A to C
            let network_trace = runtime_a
                .com_hub()
                .record_trace(TEST_ENDPOINT_C.clone())
                .await;
            assert!(network_trace.is_some());
            info!("Network trace:\n{}", network_trace.as_ref().unwrap());
            network_trace.unwrap().assert_matches_hops(&[
                (TEST_ENDPOINT_A.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_A.clone(), "mockup"),
            ]);
        })
        .await;
}

#[tokio::test]
#[timeout(2000)]
async fn network_routing_with_four_nodes_2() {
    let local = task::LocalSet::new();
    local
        .run_until(async {
            let network = get_test_network_1().await;

            // sleep 100ms
            tokio::time::sleep(Duration::from_millis(20)).await;

            info!("Network started");

            for endpoint in network.endpoints.iter() {
                if let Some(runtime) = &endpoint.runtime {
                    runtime.com_hub().print_metadata();
                }
            }

            let runtime_a = network.get_runtime(TEST_ENDPOINT_A.clone());
            let runtime_b = network.get_runtime(TEST_ENDPOINT_B.clone());
            let runtime_c = network.get_runtime(TEST_ENDPOINT_C.clone());

            // send trace from C to A
            // this first trace does not route deterministically depending on the
            // order in the priority list
            // after the first trace, the routing should be deterministic
            let network_trace = runtime_c
                .com_hub()
                .record_trace(TEST_ENDPOINT_A.clone())
                .await;
            assert!(network_trace.is_some());
            info!("Network trace:\n{}", network_trace.as_ref().unwrap());

            // clear endpoint blacklist to make sure it has no influence on the following routing
            runtime_c.com_hub().clear_endpoint_blacklist();

            // send trace from C to A again
            let network_trace = runtime_c
                .com_hub()
                .record_trace(TEST_ENDPOINT_A.clone())
                .await;
            assert!(network_trace.is_some());
            info!("Network trace:\n{}", network_trace.as_ref().unwrap());
            network_trace.unwrap().assert_matches_hops(&[
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_A.clone(), "mockup"),
                (TEST_ENDPOINT_A.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
            ]);
        })
        .await;
}

#[tokio::test]
#[timeout(2000)]
async fn network_routing_with_four_nodes_3() {
    let local = task::LocalSet::new();
    local
        .run_until(async {
            let network = get_test_network_1().await;

            // sleep 100ms
            tokio::time::sleep(Duration::from_millis(20)).await;

            info!("Network started");

            for endpoint in network.endpoints.iter() {
                if let Some(runtime) = &endpoint.runtime {
                    runtime.com_hub().print_metadata();
                }
            }

            let runtime_a = network.get_runtime(TEST_ENDPOINT_A.clone());
            let runtime_b = network.get_runtime(TEST_ENDPOINT_B.clone());
            let runtime_c = network.get_runtime(TEST_ENDPOINT_C.clone());

            // send trace from A to D
            let network_trace = runtime_a
                .com_hub()
                .record_trace(TEST_ENDPOINT_D.clone())
                .await;
            assert!(network_trace.is_some());
            info!("Network trace:\n{}", network_trace.as_ref().unwrap());
            network_trace.unwrap().assert_matches_hops(&[
                (TEST_ENDPOINT_A.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_D.clone(), "mockup"),
                (TEST_ENDPOINT_D.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_A.clone(), "mockup"),
            ]);
        })
        .await;
}

#[tokio::test]
#[timeout(2000)]
async fn network_routing_with_four_nodes_4() {
    let local = task::LocalSet::new();
    local
        .run_until(async {
            let network = get_test_network_1().await;

            // sleep 100ms
            tokio::time::sleep(Duration::from_millis(20)).await;

            info!("Network started");

            for endpoint in network.endpoints.iter() {
                if let Some(runtime) = &endpoint.runtime {
                    runtime.com_hub().print_metadata();
                }
            }

            let runtime_a = network.get_runtime(TEST_ENDPOINT_A.clone());
            let runtime_b = network.get_runtime(TEST_ENDPOINT_B.clone());
            let runtime_c = network.get_runtime(TEST_ENDPOINT_C.clone());

            // send trace from B to D
            // this first trace does not route deterministically depending on the
            // order in the priority list
            // after the first trace, the routing should be deterministic
            let network_trace = runtime_b
                .com_hub()
                .record_trace(TEST_ENDPOINT_D.clone())
                .await;
            assert!(network_trace.is_some());
            info!("Network trace:\n{}", network_trace.as_ref().unwrap());

            // clear endpoint blacklist to make sure it has no influence on the following routing
            runtime_c.com_hub().clear_endpoint_blacklist();

            // send trace from B to D again
            let network_trace = runtime_b
                .com_hub()
                .record_trace(TEST_ENDPOINT_D.clone())
                .await;
            assert!(network_trace.is_some());
            info!("Network trace:\n{}", network_trace.as_ref().unwrap());
            network_trace.unwrap().assert_matches_hops(&[
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_D.clone(), "mockup"),
                (TEST_ENDPOINT_D.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
            ]);
        })
        .await;
}

#[tokio::test]
#[timeout(2000)]
async fn network_routing_with_four_nodes_5_deterministic_priorities() {
    let local = task::LocalSet::new();
    local
        .run_until(async {
            let network =
                get_test_network_1_with_deterministic_priorities().await;

            // sleep 100ms
            tokio::time::sleep(Duration::from_millis(20)).await;

            info!("Network started");

            for endpoint in network.endpoints.iter() {
                if let Some(runtime) = &endpoint.runtime {
                    runtime.com_hub().print_metadata();
                }
            }

            let runtime_a = network.get_runtime(TEST_ENDPOINT_A.clone());
            let runtime_b = network.get_runtime(TEST_ENDPOINT_B.clone());
            let runtime_c = network.get_runtime(TEST_ENDPOINT_C.clone());

            // send trace from B to D

            let network_trace = runtime_b
                .com_hub()
                .record_trace(TEST_ENDPOINT_D.clone())
                .await;
            assert!(network_trace.is_some());
            info!("Network trace:\n{}", network_trace.as_ref().unwrap());
            network_trace.unwrap().assert_matches_hops(&[
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_D.clone(), "mockup"),
                (TEST_ENDPOINT_D.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
            ]);
        })
        .await;
}

#[tokio::test]
#[timeout(2000)]
async fn network_routing_with_four_nodes_6_deterministic_priorities() {
    let local = task::LocalSet::new();
    local
        .run_until(async {
            let network =
                get_test_network_1_with_deterministic_priorities().await;

            // sleep 100ms
            tokio::time::sleep(Duration::from_millis(20)).await;

            info!("Network started");

            for endpoint in network.endpoints.iter() {
                if let Some(runtime) = &endpoint.runtime {
                    runtime.com_hub().print_metadata();
                }
            }

            let runtime_c = network.get_runtime(TEST_ENDPOINT_C.clone());

            // send trace from C A

            let network_trace = runtime_c
                .com_hub()
                .record_trace(TEST_ENDPOINT_A.clone())
                .await;
            assert!(network_trace.is_some());
            info!("Network trace:\n{}", network_trace.as_ref().unwrap());
            network_trace.unwrap().assert_matches_hops(&[
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_D.clone(), "mockup"),
                (TEST_ENDPOINT_D.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_A.clone(), "mockup"),
                (TEST_ENDPOINT_A.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_B.clone(), "mockup"),
                (TEST_ENDPOINT_C.clone(), "mockup"),
            ]);
        })
        .await;
}

#[async_test]
#[timeout(2000)]
async fn simple_network() {
    let mut network = Network::load("simple.json");
    network.start().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    Route::between("@4726", "@s5zw")
        .to_via("@yhr9", "4726_mockup_yhr9")
        .hop("@s5zw")
        .hop("@4726")
        .test(&network)
        .await
        .unwrap();
}

#[async_test]
#[timeout(7000)]
async fn complex_network_1() {
    let mut network = Network::load("complex.json");
    network.start().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    Route::between("@bk2y", "@n7oe")
        .hop("@em68")
        .hop("@msun")
        .hop("@fyig")
        .hop("@n7oe")
        .hop("@fyig")
        .hop("@msun")
        .hop("@ajil")
        .hop("@bk2y")
        .test(&network)
        .await
        .unwrap();
}

#[async_test]
#[timeout(7000)]
async fn complex_network_2() {
    let mut network = Network::load("complex.json");
    network.start().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    Route::between("@msun", "@bk2y")
        .hop("@fyig")
        .hop("@n7oe")
        .hop("@fyig")
        .hop("@msun")
        .hop("@ajil")
        .hop("@bk2y")
        .hop("@em68")
        .hop("@msun")
        .test(&network)
        .await
        .unwrap();
}

#[async_test]
#[timeout(7000)]
async fn complex_network_3() {
    let mut network = Network::load("complex.json");
    network.start().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    Route::between("@fyig", "@n7oe")
        .hop("@n7oe")
        .hop("@fyig")
        .test(&network)
        .await
        .unwrap();
}

#[async_test]
#[timeout(7000)]
async fn threesome_1() {
    let mut network = Network::load("threesome.json");
    network.start().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    Route::between("@msun", "@n7oe")
        .hop("@em68")
        .hop("@msun")
        .hop("@ajil")
        .hop("@arh0")
        .hop("@ajil")
        .hop("@msun")
        .hop("@fyig")
        .hop("@n7oe")
        .hop("@fyig")
        .hop("@msun")
        .test(&network)
        .await
        .unwrap();
}

#[async_test]
#[timeout(7000)]
async fn multi_tracing_1() {
    let mut network = Network::load("threesome.json");
    network.start().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;

    test_routes(
        &[
            Route::between("@msun", "@n7oe")
                .fork("0")
                .hop("@em68")
                .hop("@msun")
                .hop("@ajil")
                .hop("@arh0")
                .fork("00")
                .hop("@ajil")
                .hop("@msun")
                .hop("@fyig")
                .hop("@n7oe")
                .hop("@fyig")
                .hop("@msun"),
            Route::between("@msun", "@arh0")
                .fork("0")
                .hop("@em68")
                .hop("@msun")
                .hop("@ajil")
                .hop("@arh0")
                .hop("@ajil")
                .hop("@msun"),
            Route::between("@msun", "@ajil")
                .fork("1")
                .hop("@ajil")
                .hop("@msun"),
        ],
        &network,
        TraceOptions::default(),
    )
    .await
    .unwrap();
}

#[async_test]
#[timeout(7000)]
async fn ttl_reached() {
    // working network
    let mut network = Network::load("complex.json");
    network.start().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    Route::between("@msun", "@n7oe")
        .hop("@fyig")
        .hop("@n7oe")
        .hop("@fyig")
        .hop("@msun")
        .test(&network)
        .await
        .unwrap();

    // network with only 1 hop, fails
    let mut network = Network::load("complex.json");
    network.start().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let res = Route::between("@msun", "@n7oe")
        .hop("@fyig")
        .hop("@n7oe")
        .hop("@fyig")
        .hop("@msun")
        .test_with_options(
            &network,
            TraceOptions::new(
                Some(1),
                ResponseOptions::new_with_timeout(Duration::from_secs(3)),
            ),
        )
        .await;
    assert_eq!(
        res,
        Err(RouteAssertionError::MissingResponse(
            Endpoint::from_str("@n7oe").unwrap()
        ))
    )
}

#[async_test]
#[timeout(7000)]
async fn multi_tracing_2() {
    let mut network = Network::load("se_house_of_se_nikolaus.json");
    network.start().await;
    tokio::time::sleep(Duration::from_millis(1000)).await;

    test_routes(
        &[
            Route::between("@4pk8", "@xxif")
                .fork("0")
                .hop("@46l6")
                .hop("@xxif")
                .hop("@46l6")
                .hop("@4pk8"),
            Route::between("@4pk8", "@kz0l")
                .fork("0")
                .hop("@46l6")
                .hop("@xxif")
                .fork("00")
                .hop("@owyg")
                .hop("@4pk8")
                .hop("@owyg")
                .hop("@82nq")
                .hop("@7iyl")
                .hop("@kz0l")
                .hop("@7iyl")
                .hop("@4pk8"),
            Route::between("@4pk8", "@iq1a")
                .fork("0")
                .hop("@46l6")
                .hop("@xxif")
                .fork("00")
                .hop("@owyg")
                .hop("@4pk8")
                .hop("@owyg")
                .hop("@82nq")
                .hop("@7iyl")
                .hop("@kz0l")
                .fork("000")
                .hop("@iq1a")
                .hop("@kz0l")
                .hop("@7iyl")
                .hop("@4pk8"),
        ],
        &network,
        TraceOptions::default(),
    )
    .await
    .unwrap();
}
