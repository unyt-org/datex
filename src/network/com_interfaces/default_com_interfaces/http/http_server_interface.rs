use axum::{extract::Request, routing::post};
use bytes::Bytes;
use core::cell::RefCell;

use crate::{
    collections::HashMap,
    stdlib::{net::SocketAddr, rc::Rc, sync::Arc},
    task::{UnboundedReceiver, spawn, spawn_with_panic_notify_default},
};
use axum::response::Response;
use core::time::Duration;
use futures::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use super::http_common::{HTTPError, HTTPServerInterfaceSetupData};
use crate::network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::com_interface::{
            ComInterfaceEvent,
            implementation::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
                ComInterfaceSyncFactory,
            },
            properties::InterfaceProperties,
            socket::ComInterfaceSocketUUID,
        },
    };
use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};
use log::{debug, error, info};
use tokio::sync::{RwLock, broadcast, mpsc};
use url::Url;
use uuid::serde::compact;
use datex_core::network::com_interfaces::com_interface::ComInterfaceProxy;
use crate::task::spawn_with_panic_notify;

async fn server_to_client_handler(
    Path(route): Path<String>,
    State(state): State<HTTPServerState>,
) -> Response {
    let map = state.channels.read().await;
    if let Some((sender, _)) = map.get(&route) {
        let receiver = sender.subscribe();
        let stream = BroadcastStream::new(receiver);
        Response::builder()
            .header("Content-Type", "application/octet-stream")
            .header("Cache-Control", "no-cache")
            .body(axum::body::Body::from_stream(stream))
            .unwrap()
    } else {
        Response::builder()
            .status(404)
            .body("Channel not found".into())
            .unwrap()
    }
}
async fn client_to_server_handler(
    Path(route): Path<String>,
    State(state): State<HTTPServerState>,
    req: Request,
) -> Response {
    let map = state.channels.read().await;
    if let Some((_, sender)) = map.get(&route) {
        let mut stream = req.into_body().into_data_stream();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    debug!("Received junk {}", chunk.len());
                    sender
                        .send(chunk)
                        .await
                        .map_err(|_| HTTPError::SendError)
                        .unwrap();
                }
                Err(e) => {
                    error!("Error reading body {e}");
                    return Response::builder()
                        .status(400)
                        .body("Bad Request".into())
                        .unwrap();
                }
            }
        }
        Response::builder().status(200).body("OK".into()).unwrap()
    } else {
        Response::builder()
            .status(404)
            .body("Channel not found".into())
            .unwrap()
    }
}

type HTTPChannelMap =
    HashMap<String, (broadcast::Sender<Bytes>, mpsc::Sender<Bytes>)>;

#[derive(Clone)]
struct HTTPServerState {
    channels: Arc<RwLock<HTTPChannelMap>>,
}

impl HTTPServerInterfaceSetupData {
    // FIXME: implement channel if needed, but not via methods on the HTTPServerInterfaceSetupData struct
    // pub async fn add_channel(&mut self, route: &str, endpoint: Endpoint) {
    //     let mut map = self.channels.write().await;
    //     if !map.contains_key(route) {
    //         let (server_tx, _) = broadcast::channel::<Bytes>(100);
    //         let (client_tx, mut rx) = mpsc::channel::<Bytes>(100); // FIXME #198 not braodcast needed
    //         map.insert(route.to_string(), (server_tx, client_tx));
    //         let (socket_uuid, mut sender) = self
    //             .com_interface
    //             .socket_manager()
    //             .lock()
    //             .unwrap()
    //             .create_and_init_socket(InterfaceDirection::InOut, 1);
    //         self.com_interface
    //             .socket_manager()
    //             .lock()
    //             .unwrap()
    //             .register_socket_with_endpoint(socket_uuid.clone(), endpoint, 1)
    //             .unwrap();
    //
    //         self.socket_channel_mapping
    //             .borrow_mut()
    //             .insert(route.to_string(), socket_uuid.clone());
    //
    //         spawn(async move {
    //             loop {
    //                 if let Some(data) = rx.recv().await {
    //                     debug!(
    //                         "Received data from socket {:?}: {}",
    //                         data.to_vec(),
    //                         socket_uuid
    //                     );
    //                     sender.start_send(data.to_vec()).unwrap();
    //                 }
    //             }
    //         });
    //     }
    // }
    //
    // pub async fn remove_channel(&mut self, route: &str) {
    //     let mapping = self.socket_channel_mapping.clone();
    //     let socket_uuid = {
    //         let mapping = mapping.borrow();
    //         if let Some(socket_uuid) = mapping.get(route) {
    //             socket_uuid.clone()
    //         } else {
    //             return;
    //         }
    //     };
    //
    //     self.com_interface
    //         .socket_manager()
    //         .lock()
    //         .unwrap()
    //         .remove_socket(socket_uuid);
    //
    //     let mut map = self.channels.write().await;
    //     if map.get(route).is_some() {
    //         map.remove(route);
    //     }
    // }

    async fn create_interface(
        self,
        com_interface_proxy: ComInterfaceProxy,
    ) -> Result<InterfaceProperties, InterfaceCreateError> {
        let address: String = format!("http://0.0.0.0:{}", self.port);
        let address = Url::parse(&address)
            .map_err(InterfaceCreateError::invalid_setup_data)?;

        info!("Spinning up server at {address}");

        let channels = Arc::new(RwLock::new(HashMap::new()));

        let state = HTTPServerState {
            channels: channels.clone(),
        };
        let app = Router::new()
            .route("/{route}/rx", get(server_to_client_handler))
            .route("/{route}/tx", post(client_to_server_handler))
            .with_state(state.clone());

        let addr: SocketAddr = address
            .socket_addrs(|| None)
            .map_err(InterfaceCreateError::invalid_setup_data)?
            .first()
            .cloned()
            .ok_or(InterfaceCreateError::invalid_setup_data(
                "Socket address invalid",
            ))?;

        println!("HTTP server starting on http://{addr}");
        spawn(async move {
            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let socket_channel_mapping = Rc::new(RefCell::new(HashMap::new()));

        spawn_with_panic_notify(&com_interface_proxy.async_context, Self::event_handler_task(
            socket_channel_mapping.clone(),
            channels.clone(),
            com_interface_proxy.event_receiver,
        ));

        Ok(Self::get_default_properties())
    }

    /// background task to handle com hub events (e.g. outgoing messages)
    async fn event_handler_task(
        socket_channel_mapping: Rc<
            RefCell<HashMap<String, ComInterfaceSocketUUID>>,
        >,
        channels: Arc<RwLock<HTTPChannelMap>>,
        mut receiver: UnboundedReceiver<ComInterfaceEvent>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceEvent::SendBlock(block, socket_uuid) => {
                    let route = socket_channel_mapping.borrow();
                    let route = route
                        .iter()
                        .find(|(_, v)| *v == &socket_uuid)
                        .map(|(k, _)| k);
                    if route.is_none() {
                        // TODO: handle
                        return;
                    }
                    let route = route.unwrap().to_string();
                    let map = channels.read().await;
                    if let Some((sender, _)) = map.get(&route) {
                        let _ = sender.send(Bytes::from(block));
                    } else {
                        // TODO: handle
                    }
                }
                _ => todo!(),
            }
        }
    }
}

impl ComInterfaceAsyncFactory for HTTPServerInterfaceSetupData {
    fn create_interface(
        self,
        com_interface_proxy: ComInterfaceProxy,
    ) -> ComInterfaceAsyncFactoryResult {
        Box::pin(async move {
            self.create_interface(com_interface_proxy).await
        })
    }

    fn get_default_properties() -> InterfaceProperties {
        InterfaceProperties {
            interface_type: "http-server".to_string(),
            channel: "http".to_string(),
            round_trip_time: Duration::from_millis(20),
            max_bandwidth: 1000,
            ..InterfaceProperties::default()
        }
    }
}
