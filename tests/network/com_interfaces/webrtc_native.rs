use std::{cell::RefCell, io::Bytes, rc::Rc, sync::Arc, time::Duration};

use crate::network::helpers::mock_setup::{TEST_ENDPOINT_A, TEST_ENDPOINT_B};
use datex_core::global::dxb_block::DXBBlock;
use datex_core::network::com_interfaces::com_interface::ComInterface;
use datex_core::network::com_interfaces::com_interface::socket::{
    ComInterfaceSocket, ComInterfaceSocketEvent, ComInterfaceSocketUUID,
};
use datex_core::task::UnboundedReceiver;
use datex_core::utils::context::init_global_context;
use datex_core::{
    network::com_interfaces::default_com_interfaces::webrtc::{
        webrtc_common::{
            media_tracks::{MediaKind, MediaTrack},
            webrtc_commons::WebRTCInterfaceSetupData,
            webrtc_trait::{WebRTCTrait, WebRTCTraitInternal},
        },
        webrtc_native_interface::{TrackLocal, WebRTCNativeInterface},
    },
    task::{sleep, spawn_local},
    utils::uuid::UUID,
};
use datex_macros::async_test;
use ntest_timeout::timeout;
use webrtc::{
    media::Sample,
    rtp::{header::Header, packet::Packet},
    track::track_local::{
        TrackLocalWriter, track_local_static_rtp::TrackLocalStaticRTP,
        track_local_static_sample::TrackLocalStaticSample,
    },
};

async fn create_webrtc_interfaces() -> (
    Rc<ComInterface>,
    Rc<ComInterface>,
    UnboundedReceiver<ComInterfaceSocketEvent>,
    UnboundedReceiver<ComInterfaceSocketEvent>,
) {
    // Create a WebRTCNativeInterface instance on each side (remote: @a)
    let com_interface_a = ComInterface::create_async_with_implementation::<
        WebRTCNativeInterface,
    >(WebRTCInterfaceSetupData {
        peer_endpoint: TEST_ENDPOINT_A.clone(),
        ice_servers: None,
    })
    .await
    .expect("Failed to create WebRTCNativeInterface");
    let receiver_a = com_interface_a.take_socket_event_receiver();

    // Create a WebRTCNativeInterface instance on each side (remote: @b)
    let com_interface_b = ComInterface::create_async_with_implementation::<
        WebRTCNativeInterface,
    >(WebRTCInterfaceSetupData {
        peer_endpoint: TEST_ENDPOINT_B.clone(),
        ice_servers: None,
    })
    .await
    .expect("Failed to create WebRTCNativeInterface");
    let receiver_b = com_interface_b.take_socket_event_receiver();

    let interface_a_clone = com_interface_a.clone();
    let inteface_b_clone = com_interface_b.clone();

    let webrtc_interface_a = com_interface_a.clone();
    let webrtc_interface_a =
        webrtc_interface_a.implementation_mut::<WebRTCNativeInterface>();
    let webrtc_interface_b = com_interface_b.clone();
    let webrtc_interface_b =
        webrtc_interface_b.implementation_mut::<WebRTCNativeInterface>();

    // Set up the on_ice_candidate callback for both interfaces
    // The candidate would be transmitted to the other side via some signaling server
    // In this case, we are using a mock setup and since we are in the same process,
    // we can directly call the "add_ice_candidate" callback on the other side
    webrtc_interface_a.set_on_ice_candidate(Box::new(move |candidate| {
        let interface_b = inteface_b_clone.clone();
        spawn_local(async move {
            let webrtc_interface_b =
                interface_b.implementation_mut::<WebRTCNativeInterface>();
            webrtc_interface_b
                .add_ice_candidate(candidate)
                .await
                .unwrap();
        });
    }));

    webrtc_interface_b.set_on_ice_candidate(Box::new(move |candidate| {
        let interface_a = interface_a_clone.clone();
        spawn_local(async move {
            let webrtc_interface_a =
                interface_a.implementation_mut::<WebRTCNativeInterface>();
            webrtc_interface_a
                .add_ice_candidate(candidate)
                .await
                .unwrap();
        });
    }));
    (com_interface_a, com_interface_b, receiver_a, receiver_b)
}

async fn setup_webrtc_interfaces() -> (
    Rc<ComInterface>,
    Rc<ComInterface>,
    ComInterfaceSocket,
    ComInterfaceSocket,
) {
    let (com_interface_a, com_interface_b, mut receiver_a, mut receiver_b) =
        create_webrtc_interfaces().await;

    let webrtc_interface_a = com_interface_a.clone();
    let webrtc_interface_a =
        webrtc_interface_a.implementation::<WebRTCNativeInterface>();
    let webrtc_interface_b = com_interface_b.clone();
    let webrtc_interface_b =
        webrtc_interface_b.implementation::<WebRTCNativeInterface>();

    // Create an offer on one side and an answer on the other side
    // The initator would send the offer to the other side via some other channel
    // When a connection handshake is planned on both side, the initiator should be
    // picked by the endpoint name or something deterministic that both sides
    // can agree on
    let offer = webrtc_interface_a.create_offer().await.unwrap();

    // The offer would be transmitted to the other side via some other channel
    // In this case, we are using a mock setup and since we are in the same process,
    // we can directly call the "create_answer" and "set_answer" callbacks on the other side
    let answer = webrtc_interface_b.create_answer(offer).await.unwrap();
    drop(webrtc_interface_b);

    webrtc_interface_a.set_answer(answer).await.unwrap();
    drop(webrtc_interface_a);

    // Wait for the data channel and socket to be connected

    let socket_a = match receiver_a.next().await {
        Some(ComInterfaceSocketEvent::NewSocket(socket)) => socket,
        _ => panic!("Expected NewSocket event for server"),
    };
    let socket_b = match receiver_b.next().await {
        Some(ComInterfaceSocketEvent::NewSocket(socket)) => socket,
        _ => panic!("Expected NewSocket event for server"),
    };

    (com_interface_a, com_interface_b, socket_a, socket_b)
}

#[async_test]
#[timeout(10000)]
pub async fn test_connect() {
    let block_a_to_b = DXBBlock::new_with_body(b"Hello from A to B");
    let block_b_to_a = DXBBlock::new_with_body(b"Hello from B to A");
    let (com_interface_a, com_interface_b, mut socket_a, mut socket_b) =
        setup_webrtc_interfaces().await;

    // com_interface_a.borrow().wait_for_connection().await.unwrap();
    // com_interface_b.borrow().wait_for_connection().await.unwrap();

    // Since the WebRTC connection interface is a single socket provider,
    // it currently doesn't care about the socket uuid. In the future, we could
    // have different sockets for the same endpoint but with different channel configs
    // such as reliable, unreliable, ordered, unordered, etc.
    com_interface_a
        .send_block(&block_a_to_b.to_bytes().unwrap(), socket_a.uuid.clone());
    com_interface_b
        .send_block(&block_b_to_a.to_bytes().unwrap(), socket_b.uuid.clone());

    // Wait for the messages to be received
    sleep(Duration::from_secs(1)).await;

    let mut socket_a_in = socket_a.take_block_in_receiver();
    assert_eq!(socket_a_in.next().await.unwrap(), block_b_to_a);

    let mut socket_b_in = socket_b.take_block_in_receiver();
    assert_eq!(socket_b_in.next().await.unwrap(), block_a_to_b);
}

#[async_test]
#[timeout(10000)]
pub async fn test_media_track() {
    let (com_interface_a, com_interface_b, mut receiver_a, mut receiver_b) =
        create_webrtc_interfaces().await;

    let webrtc_interface_a = com_interface_a.clone();
    let webrtc_interface_a =
        webrtc_interface_a.implementation_mut::<WebRTCNativeInterface>();
    let webrtc_interface_b = com_interface_b.clone();
    let webrtc_interface_b =
        webrtc_interface_b.implementation_mut::<WebRTCNativeInterface>();
    let tx_track: Rc<RefCell<MediaTrack<Arc<TrackLocal>>>> = webrtc_interface_a
        .create_media_track("dx".to_owned(), MediaKind::Audio)
        .await
        .unwrap();
    println!("Has local media track: {:?}", tx_track.borrow().kind);

    let offer = webrtc_interface_a.create_offer().await.unwrap();

    let answer = webrtc_interface_b.create_answer(offer).await.unwrap();
    webrtc_interface_a.set_answer(answer).await.unwrap();

    drop(webrtc_interface_a);
    drop(webrtc_interface_b);

    // interface_a.borrow().wait_for_connection().await.unwrap();
    // interface_b.borrow().wait_for_connection().await.unwrap();

    // Wait for the data channel and socket to be connected
    match receiver_a.next().await {
        Some(ComInterfaceSocketEvent::NewSocket(socket)) => socket,
        _ => panic!("Expected NewSocket event for server"),
    };
    match receiver_b.next().await {
        Some(ComInterfaceSocketEvent::NewSocket(socket)) => socket,
        _ => panic!("Expected NewSocket event for server"),
    };

    spawn_local(async move {
        let binding = tx_track.borrow();
        let track = binding
            .track
            .as_any()
            .downcast_ref::<TrackLocalStaticSample>()
            .unwrap();
        track
            .write_sample(&webrtc::media::Sample {
                data: vec![0u8; 960].into(),
                duration: Duration::from_millis(20),
                ..Default::default()
            })
            .await;

        // let track = binding.track.as_any().downcast_ref::<TrackLocalStaticRTP>().unwrap();
        // let mut sequence_number = 0u16;
        // loop {
        //     let packet = Packet {
        //         header: Header {
        //             version: 2,
        //             sequence_number,
        //             payload_type: 96,
        //             ..Default::default()
        //         },
        //         payload: vec![0u8; 2].into(),
        //     };
        //     sequence_number = sequence_number.wrapping_add(1);
        //     track
        //         .write_rtp_with_extensions(&packet, &[])
        //         .await
        //         .unwrap();
        // }
    });
    sleep(Duration::from_secs(2)).await;

    let webrtc_interface_b =
        com_interface_b.implementation_mut::<WebRTCNativeInterface>();
    let tracks = webrtc_interface_b.provide_remote_media_tracks();
    let tracks = &tracks.borrow();
    let track = tracks.tracks.values().next().unwrap();
    let track = track.borrow();
    println!("Received track id: {:?}", track.id());
    let n = track.track.read_rtp().await.unwrap().0.to_string();
    println!("Read {} bytes from track", n);
    println!("Tracks B: {:?}", track.kind());
}
