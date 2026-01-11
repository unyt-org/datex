use crate::network::helpers::{
    mock_setup::{
        TEST_ENDPOINT_A, TEST_ENDPOINT_B,
    },
    mockup_interface::MockupInterface,
};
use datex_core::{
    network::{
        com_hub::InterfacePriority,
    },
    run_async_thread,
    utils::context::init_global_context,
};
use datex_macros::async_test;
use ntest_timeout::timeout;
use std::{sync::mpsc, thread};
use tokio::task::yield_now;
use datex_core::task::create_unbounded_channel;
use crate::network::helpers::mock_setup::{get_default_mock_setup_with_two_connected_com_hubs, get_mock_setup_with_com_hub, MockupSetupData};
use crate::network::helpers::mockup_interface::MockupInterfaceSetupData;

#[async_test]
#[timeout(1000)]
async fn create_network_trace() {
    let (
        (com_hub_mut_a, ..),
        ..
    ) = get_default_mock_setup_with_two_connected_com_hubs().await;
    
    yield_now().await;
    yield_now().await;
    
    log::info!("Sending trace from A to B");

    // send trace from A to B
    let network_trace =
        com_hub_mut_a.record_trace(TEST_ENDPOINT_B.clone()).await;

    assert!(network_trace.is_some());
    log::info!("Network trace:\n{}", network_trace.as_ref().unwrap());

    assert!(network_trace.unwrap().matches_hops(&[
        (TEST_ENDPOINT_A.clone(), "mockup"),
        (TEST_ENDPOINT_B.clone(), "mockup"),
        (TEST_ENDPOINT_B.clone(), "mockup"),
        (TEST_ENDPOINT_A.clone(), "mockup")
    ]));
}

// same as create_network_trace, but both com hubs in separate threads
#[tokio::test]
#[timeout(3000)]
async fn create_network_trace_separate_threads() {
    // create a new thread for each com hub
    let (sender_a, receiver_a) = create_unbounded_channel::<Vec<u8>>();
    let (sender_b, receiver_b) = create_unbounded_channel::<Vec<u8>>();


    // Endpoint A
    let thread_a = run_async_thread! {
        init_global_context();

         let (com_hub_mut_a, ..) = get_mock_setup_with_com_hub(MockupSetupData {
                interface_setup_data: MockupInterfaceSetupData {
                    endpoint: Some(TEST_ENDPOINT_A.clone()),
                    receiver_in: Some(receiver_b),
                    sender_out: Some(sender_a),
                    ..Default::default()
                },
                ..Default::default()
            }).await;
        
        log::info!("Sending trace from A to B");
        // sleep required to handle message transfer
        tokio::time::sleep(tokio::time::Duration::from_millis(100))
            .await;

        // send trace from A to B
        let network_trace = com_hub_mut_a
            .record_trace(TEST_ENDPOINT_B.clone())
            .await;

        assert!(network_trace.is_some());
        log::info!(
            "Network trace:\n{}",
            network_trace.as_ref().unwrap()
        );

        assert!(network_trace.unwrap().matches_hops(&[
            (TEST_ENDPOINT_A.clone(), "mockup"),
            (TEST_ENDPOINT_B.clone(), "mockup"),
            (TEST_ENDPOINT_B.clone(), "mockup"),
            (TEST_ENDPOINT_A.clone(), "mockup")
        ]));
    };

    // Endpoint B
    let thread_b = run_async_thread! {
        init_global_context();

        let _ = get_mock_setup_with_com_hub(MockupSetupData {
            interface_setup_data: MockupInterfaceSetupData {
                endpoint: Some(TEST_ENDPOINT_B.clone()),
                receiver_in: Some(receiver_a),
                sender_out: Some(sender_b),
                ..Default::default()
            },
            ..Default::default()
        }).await;

        // sleep 2s to ensure that the other thread has finished
        tokio::time::sleep(tokio::time::Duration::from_millis(200))
            .await;
    };

    // Wait for both threads to finish
    thread_a.join().expect("Thread A panicked");
    thread_b.join().expect("Thread B panicked");
}
