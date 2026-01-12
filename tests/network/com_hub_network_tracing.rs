use crate::network::helpers::{
    mock_setup::{
        MockupSetupData, TEST_ENDPOINT_A, TEST_ENDPOINT_B,
        get_default_mock_setup_with_two_connected_com_hubs,
        get_mock_setup_with_com_hub,
    },
    mockup_interface::MockupInterfaceSetupData,
};
use datex_core::{
    global::dxb_block::IncomingSection,
    network::com_interfaces::com_interface::ComInterfaceProxy,
    run_async_thread, task::create_unbounded_channel,
    utils::context::init_global_context,
};
use datex_macros::async_test;
use ntest_timeout::timeout;
use std::thread;
use log::info;
use tokio::{sync::oneshot, task::yield_now};
use datex_core::network::com_interfaces::com_interface::properties::InterfaceProperties;

#[async_test]
#[timeout(1000)]
async fn create_network_trace() {
    let ((com_hub_a, ..), ..) =
        get_default_mock_setup_with_two_connected_com_hubs().await;
    yield_now().await;

    info!("Sending trace from A to B");

    // send trace from A to B
    let network_trace =
        com_hub_a.record_trace(TEST_ENDPOINT_B.clone()).await;
    yield_now().await;

    assert!(network_trace.is_some());
    info!("Network trace:\n{}", network_trace.as_ref().unwrap());

    network_trace.unwrap().assert_matches_hops(&[
        (TEST_ENDPOINT_A.clone(), "mockup"),
        (TEST_ENDPOINT_B.clone(), "mockup"),
        (TEST_ENDPOINT_B.clone(), "mockup"),
        (TEST_ENDPOINT_A.clone(), "mockup")
    ]);
}

// same as create_network_trace, but both com hubs in separate threads
#[async_test]
#[timeout(1000)]
async fn create_network_trace_separate_threads() {
    // is later sent from thread a
    let (interface_proxy_a_tx, interface_proxy_a_rx) =
        oneshot::channel::<ComInterfaceProxy>();
    let (interface_proxy_b_tx, interface_proxy_b_rx) =
        oneshot::channel::<ComInterfaceProxy>();

    // Endpoint A
    let thread_a = run_async_thread! {
        init_global_context();

        let (com_hub_a, interface_proxy_a) = get_mock_setup_with_com_hub(MockupSetupData {
             interface_properties: InterfaceProperties {
                name: Some("A->B".to_string()),
                interface_type: "mockup".to_string(),
                channel: "mockup".to_string(),
                ..Default::default()
            },
            local_endpoint: TEST_ENDPOINT_A.clone(),
            com_hub_sections_sender: None,
            ..Default::default()
        }).await;

        interface_proxy_a_tx.send(interface_proxy_a).unwrap();

        // sleep required to wait for interface_proxy_a_tx send
        tokio::time::sleep(tokio::time::Duration::from_millis(10))
            .await;
        info!("Sending trace from A to B");

        // send trace from A to B
        let network_trace = com_hub_a
            .record_trace(TEST_ENDPOINT_B.clone())
            .await;

        assert!(network_trace.is_some());
        info!(
            "Network trace:\n{}",
            network_trace.as_ref().unwrap()
        );

        network_trace.unwrap().assert_matches_hops(&[
            (TEST_ENDPOINT_A.clone(), "mockup"),
            (TEST_ENDPOINT_B.clone(), "mockup"),
            (TEST_ENDPOINT_B.clone(), "mockup"),
            (TEST_ENDPOINT_A.clone(), "mockup")
        ]);
    };

    // Endpoint B
    let thread_b = run_async_thread! {
        init_global_context();

        let (_, interface_proxy_b) = get_mock_setup_with_com_hub(MockupSetupData {
             interface_properties: InterfaceProperties {
                name: Some("B->A".to_string()),
                interface_type: "mockup".to_string(),
                channel: "mockup".to_string(),
                ..Default::default()
            },
            local_endpoint: TEST_ENDPOINT_B.clone(),
            com_hub_sections_sender: None,
            ..Default::default()
        }).await;

        interface_proxy_b_tx.send(interface_proxy_b).unwrap();

        // sleep to ensure that the other thread has finished
        tokio::time::sleep(tokio::time::Duration::from_millis(20))
            .await;
    };

    // wait for both interface proxies from the threads and couple them
    ComInterfaceProxy::couple_bidirectional(
        (
            interface_proxy_a_rx.await.unwrap(),
            None,
        ),
        (
            interface_proxy_b_rx.await.unwrap(),
            None,
        ),
    );

    info!("Both interface proxies coupled");

    // sleep to run coupling tasks in background before joining threads
    tokio::time::sleep(tokio::time::Duration::from_millis(30))
        .await;

    // Wait for both threads to finish
    thread_a.join().expect("Thread A panicked");
    thread_b.join().expect("Thread B panicked");
}
