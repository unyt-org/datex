use crate::prelude::*;
use alloc::rc::Rc;
use core::pin::Pin;
use alloc::sync::Arc;
use core::str::FromStr;
use futures_util::future::join;
use tokio::task::{spawn_local, yield_now, LocalSet};
use crate::channel::mpsc::{create_unbounded_channel, UnboundedReceiver};
use crate::global::dxb_block::{DXBBlock, IncomingSection};
use crate::network::com_hub::{ComHub, InterfacePriority};
use crate::network::com_hub::metadata::ComHubMetadata;
use crate::network::com_interfaces::com_interface::ComInterfaceUUID;
use crate::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SendCallback, SocketConfiguration, SocketProperties};
use crate::network::com_interfaces::com_interface::properties::{ComInterfaceProperties, InterfaceDirection};
use crate::values::core_values::endpoint::Endpoint;
use crate::std_sync::Mutex;

lazy_static::lazy_static! {
    pub static ref TEST_ENDPOINT_A: Endpoint = Endpoint::from_str("@test-a").unwrap();
    pub static ref TEST_ENDPOINT_B: Endpoint = Endpoint::from_str("@test-b").unwrap();
    pub static ref TEST_ENDPOINT_C: Endpoint = Endpoint::from_str("@test-c").unwrap();
    pub static ref TEST_ENDPOINT_D: Endpoint = Endpoint::from_str("@test-d").unwrap();
}

pub struct ComHubPeerWithFuture {
    pub com_hub: Rc<ComHub>,
    pub task_future: Pin<Box<dyn Future<Output = ()>>>,
    pub incoming_sections_receiver: UnboundedReceiver<IncomingSection>,
    pub com_interface_uuid: ComInterfaceUUID,
    pub com_interface_properties: Rc<ComInterfaceProperties>,
}

pub struct ComHubPeer {
    pub com_hub: Rc<ComHub>,
    pub incoming_sections_receiver: UnboundedReceiver<IncomingSection>,
    pub com_interface_uuid: ComInterfaceUUID,
    pub com_interface_properties: Rc<ComInterfaceProperties>,
}

impl ComHubPeerWithFuture {
    fn split(self) -> (ComHubPeer, Pin<Box<dyn Future<Output = ()>>>) {
        (
            ComHubPeer {
                com_hub: self.com_hub,
                incoming_sections_receiver: self.incoming_sections_receiver,
                com_interface_uuid: self.com_interface_uuid,
                com_interface_properties: self.com_interface_properties,
            },
            self.task_future
        )
    }
}

/// Creates two bidirectionally coupled ComInterfaceConfigurations for testing purposes.
pub async fn get_coupled_com_hubs() -> (ComHubPeerWithFuture, ComHubPeerWithFuture, impl Future<Output = ()>) {

    let (incoming_sections_sender_a, incoming_sections_receiver_a) =
        create_unbounded_channel::<IncomingSection>();
    let (com_hub_a, task_future_a) = ComHub::create(TEST_ENDPOINT_A.clone(), incoming_sections_sender_a);

    let (incoming_sections_sender_b, incoming_sections_receiver_b) =
        create_unbounded_channel::<IncomingSection>();
    let (com_hub_b, task_future_b) = ComHub::create(TEST_ENDPOINT_B.clone(), incoming_sections_sender_b);

    let res = couple_com_hubs_internal(com_hub_a.clone(), com_hub_b.clone());

    let ((com_interface_uuid_a, com_interface_properties_a), (com_interface_uuid_b, com_interface_properties_b), init_future) = res;

    (
        ComHubPeerWithFuture {
            com_hub: com_hub_a,
            task_future: Box::pin(task_future_a),
            incoming_sections_receiver: incoming_sections_receiver_a,
            com_interface_uuid: com_interface_uuid_a,
            com_interface_properties: com_interface_properties_a,
        },
        ComHubPeerWithFuture {
            com_hub: com_hub_b,
            task_future: Box::pin(task_future_b),
            incoming_sections_receiver: incoming_sections_receiver_b,
            com_interface_uuid: com_interface_uuid_b,
            com_interface_properties: com_interface_properties_b,
        },
        init_future
    )
}

/// Couples to existing ComHubs by adding bidirectional interfaces.
pub async fn couple_com_hubs(com_hub_a: Rc<ComHub>, com_hub_b: Rc<ComHub>) -> (
    (ComInterfaceUUID, Rc<ComInterfaceProperties>),
    (ComInterfaceUUID, Rc<ComInterfaceProperties>),
) {
    let (a, b, init_future) = couple_com_hubs_internal(com_hub_a, com_hub_b);
    init_future.await;
    (a, b)
}

/// Couples to existing ComHubs by adding bidirectional interfaces.
/// Returns the interface UUIDs and properties of the added interfaces, as well as a future that completes when the interfaces are fully connected and ready to use.
fn couple_com_hubs_internal(com_hub_a: Rc<ComHub>, com_hub_b: Rc<ComHub>) -> (
    (ComInterfaceUUID, Rc<ComInterfaceProperties>),
    (ComInterfaceUUID, Rc<ComInterfaceProperties>),
    impl Future<Output = ()>
) {
    let (send_a_to_b, mut receive_a_to_b) = create_unbounded_channel::<DXBBlock>();
    let send_a_to_b = Arc::new(Mutex::new(send_a_to_b));
    let (send_b_to_a, mut receive_b_to_a) = create_unbounded_channel::<DXBBlock>();
    let send_b_to_a = Arc::new(Mutex::new(send_b_to_a));

    let config_b = ComInterfaceConfiguration::new_single_socket(
        ComInterfaceProperties {
            interface_type: "test-interface".to_string(),
            direction: InterfaceDirection::InOut,
            ..Default::default()
        },
        SocketConfiguration::new_in_out(
            SocketProperties::new(InterfaceDirection::InOut, 1),
            async gen move {
                while let Some(block) = receive_a_to_b.next().await {
                    yield Ok(block.to_bytes())
                }
            },
            SendCallback::new_async(move |block| {
                let send_b_to_a = send_b_to_a.clone();
                async move {
                    send_b_to_a.try_lock().unwrap().start_send(block).unwrap();
                    Ok(())
                }
            })
        )
    );

    let config_a = ComInterfaceConfiguration::new_single_socket(
        ComInterfaceProperties {
            interface_type: "test-interface".to_string(),
            direction: InterfaceDirection::InOut,
            ..Default::default()
        },
        SocketConfiguration::new_in_out(
            SocketProperties::new(InterfaceDirection::InOut, 1),
            async gen move {
                while let Some(block) = receive_b_to_a.next().await {
                    yield Ok(block.to_bytes())
                }
            },
            SendCallback::new_async(move |block| {
                let send_a_to_b = send_a_to_b.clone();
                async move {
                    send_a_to_b.try_lock().unwrap().start_send(block).unwrap();
                    Ok(())
                }
            })
        )
    );

    let com_interface_uuid_a = config_a.uuid();
    let com_interface_uuid_b = config_b.uuid();
    let com_interface_properties_a = config_a.properties.clone();
    let com_interface_properties_b = config_b.properties.clone();

    // add and connect interfaces
    let res_a = com_hub_a.clone().add_interface_from_configuration(config_a, InterfacePriority::default()).unwrap();
    let res_b = com_hub_b.clone().add_interface_from_configuration(config_b, InterfacePriority::default()).unwrap();


    (
        (com_interface_uuid_a, com_interface_properties_a),
        (com_interface_uuid_b, com_interface_properties_b),
        async {
            join(
                async {
                    if let Some(fut) = res_a {
                        fut.await.unwrap()
                    }
                },
                async {
                    if let Some(fut) = res_b {
                        fut.await.unwrap()
                    }
                }
            ).await;
        }
    )
}

pub async fn run_with_coupled_com_hubs<F, Fut>(test: F) -> Fut::Output
where
    F: FnOnce(ComHubPeer, ComHubPeer) -> Fut,
    Fut: Future,
{
    let local = LocalSet::new();
    local
        .run_until(async {
            let (peer_a_with_future, peer_b_with_future, init_future) = get_coupled_com_hubs().await;

            let (peer_a, peer_a_future) = peer_a_with_future.split();
            let (peer_b, peer_b_future) = peer_b_with_future.split();

            // run task futures in background
            spawn_local(peer_a_future);
            spawn_local(peer_b_future);
            spawn_local(init_future);

            // allow sockets to connect
            yield_now().await;

            test(peer_a, peer_b).await
        }
        ).await
}


pub fn get_endpoints_from_com_hub_metadata(
    com_hub_metadata: ComHubMetadata,
) -> Vec<(Option<Endpoint>, Option<i8>)> {
    com_hub_metadata
        .interfaces
        .into_iter()
        .flat_map(|e| {
            e.sockets
                .into_iter()
                .map(|s| (s.endpoint, s.properties.map(|p| p.distance)))
        })
        .collect::<Vec<_>>()
}
