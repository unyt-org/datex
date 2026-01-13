use core::{
    fmt::{self, Debug, Display},
    str::FromStr,
};
use datex_core::{
    network::{
        com_hub::{
            InterfacePriority,
            network_tracing::TraceOptions,
        },
        com_interfaces::com_interface::{
            properties::InterfaceDirection,
        },
    },
    runtime::{AsyncContext, Runtime, RuntimeConfig},
    serde::serializer::to_value_container,
    task::{UnboundedReceiver, UnboundedSender, create_unbounded_channel},
    values::{
        core_values::endpoint::Endpoint, value_container::ValueContainer,
    },
};
use log::info;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, env, fs, path::Path, rc::Rc, sync::mpsc,
};
use tokio::task::yield_now;
use datex_core::network::com_interfaces::com_interface::{ComInterfaceEvent, ComInterfaceProxy};
use datex_core::network::com_interfaces::com_interface::properties::InterfaceProperties;
use datex_core::task::spawn_with_panic_notify_default;

// FIXME: refactor and rename
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MockupInterfaceSetupData {
    pub name: String,
    pub endpoint: Option<Endpoint>,
    pub direction: InterfaceDirection,
}

impl MockupInterfaceSetupData {
    pub fn new(name: &str) -> Self {
        MockupInterfaceSetupData {
            name: name.to_string(),
            endpoint: None,
            direction: InterfaceDirection::InOut,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InterfaceConnection {
    interface_type: String,
    priority: InterfacePriority,
    pub setup_data: Option<MockupInterfaceSetupData>,
    pub endpoint: Option<Endpoint>,
}

impl InterfaceConnection {
    pub fn new(
        interface_type: &str,
        priority: InterfacePriority,
        setup_data: MockupInterfaceSetupData,
    ) -> Self {
        InterfaceConnection {
            interface_type: interface_type.to_string(),
            priority,
            setup_data: Some(setup_data),
            endpoint: None,
        }
    }

    pub fn with_endpoint(mut self, endpoint: Endpoint) -> Self {
        self.endpoint = Some(endpoint);
        self
    }
}

#[derive(Debug)]
pub struct Node {
    pub endpoint: Endpoint,
    pub connections: Vec<InterfaceConnection>,
    pub runtime: Option<Runtime>,
}

impl Node {
    pub fn new(endpoint: impl Into<Endpoint>) -> Self {
        Node {
            endpoint: endpoint.into(),
            connections: Vec::new(),
            runtime: None,
        }
    }

    pub fn with_connection(mut self, connection: InterfaceConnection) -> Self {
        self.connections.push(connection);
        self
    }
}

pub struct MockupInterfaceChannelEndpoint {
    sender: UnboundedSender<Vec<u8>>,
    receiver: UnboundedReceiver<Vec<u8>>,
}

type MockupInterfaceChannels =
    HashMap<String, Option<MockupInterfaceChannelEndpoint>>;

pub struct Network {
    pub is_initialized: bool,
    pub endpoints: Vec<Node>,
}
#[derive(Clone)]
pub struct Route {
    pub receiver: Endpoint,
    pub hops: Vec<(Endpoint, Option<String>, Option<String>)>,
    // temp remember last fork
    pub next_fork: Option<String>,
}

impl Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (endpoint, channel, _fork)) in self.hops.iter().enumerate() {
            // Write the endpoint
            core::write!(f, "{endpoint}")?;

            // If not the last, write the arrow + optional channel
            if i + 1 < self.hops.len() {
                if let Some(chan) = channel {
                    core::write!(f, " -({chan})-> ")?;
                } else {
                    core::write!(f, " --> ")?;
                }
            }
        }
        Ok(())
    }
}

impl Route {
    pub fn between<R>(source: R, receiver: R) -> Self
    where
        R: TryInto<Endpoint>,
        R::Error: Debug,
    {
        Route {
            receiver: receiver.try_into().expect("Invalid receiver endpoint"),
            hops: vec![(
                source.try_into().expect("Invalid source endpoint"),
                None,
                None,
            )],
            next_fork: None,
        }
    }

    pub fn hop<R>(mut self, target: R) -> Self
    where
        R: TryInto<Endpoint>,
        R::Error: Debug,
    {
        self.add_hop(target.try_into().expect("Invalid target endpoint"));
        self
    }

    pub fn fork(mut self, fork_nr: &str) -> Self {
        self.next_fork = Some(fork_nr.to_string());
        self
    }

    pub fn to_via<R>(mut self, target: R, channel: &str) -> Self
    where
        R: TryInto<Endpoint>,
        R::Error: Debug,
    {
        let len = self.hops.len();
        if len > 0 {
            self.hops[len - 1].1 = Some(channel.to_string());
        }
        self.add_hop(target.try_into().expect("Invalid target endpoint"));
        self
    }

    pub fn back(mut self) -> Self {
        if self.hops.len() >= 2 {
            let len = self.hops.len();
            let to = self.hops[len - 2].0.clone();
            let channel = self.hops[len - 2].1.clone();
            self.hops[len - 1].1 = Some(channel.clone().unwrap_or_default());
            self.add_hop(to);
        }
        self
    }
    pub fn back_via(mut self, channel: &str) -> Self {
        if self.hops.len() >= 2 {
            let len = self.hops.len();
            let to = self.hops[len - 2].0.clone();
            self.hops[len - 1].1 = Some(channel.to_string());
            self.add_hop(to);
        }
        self
    }

    fn add_hop(&mut self, to: impl Into<Endpoint>) {
        let fork = self.next_fork.take();
        self.hops.push((to.into(), None, fork));
    }

    /// Converts the Route into a sequence of (from, channel, to) triples
    pub fn to_segments(&self) -> Vec<(Endpoint, String, Endpoint)> {
        let mut segments = Vec::new();
        for w in self.hops.windows(2) {
            if let [(from, Some(chan), _), (to, _, _)] = &w {
                segments.push((from.clone(), chan.clone(), to.clone()));
            }
        }
        segments
    }

    pub async fn test(
        &self,
        network: &Network,
    ) -> Result<(), RouteAssertionError> {
        self.test_with_options(network, TraceOptions::default())
            .await
    }

    pub async fn test_with_options(
        &self,
        network: &Network,
        options: TraceOptions,
    ) -> Result<(), RouteAssertionError> {
        test_routes(core::slice::from_ref(self), network, options).await
    }
}

pub async fn test_routes(
    routes: &[Route],
    network: &Network,
    options: TraceOptions,
) -> Result<(), RouteAssertionError> {
    let start = routes[0].hops[0].0.clone();
    let ends = routes
        .iter()
        .map(|r| r.hops.last().unwrap().0.clone())
        .collect::<Vec<_>>();

    // make sure the start endpoint for all routes is the same
    for route in routes {
        if route.hops[0].0 != start {
            core::panic!(
                "Route start endpoints must all be the same. Found {} instead of {}",
                route.hops[0].0,
                start
            );
        }
    }

    for end in ends {
        if start != end {
            core::panic!(
                "Route start {} does not match receiver {}",
                start,
                end
            );
        }
    }

    let network_traces = network
        .get_runtime(start)
        .com_hub()
        .record_trace_multiple_with_options(TraceOptions {
            endpoints: routes.iter().map(|r| r.receiver.clone()).collect(),
            ..options
        })
        .await;

    // combine received traces with original routes
    let route_pairs = routes
        .iter()
        .map(|route| {
            // find matching route with the same receiver in network_traces
            network_traces
                .iter()
                .find(|t| t.receiver == route.receiver)
                .ok_or_else(|| {
                    RouteAssertionError::MissingResponse(route.receiver.clone())
                })
                .map(|trace| (trace, route))
        })
        .collect::<Result<Vec<_>, _>>()?;

    for (trace, route) in route_pairs {
        // print network trace
        info!("Network trace:\n{trace}");

        // combine original and expected hops
        let hop_pairs = trace
            .hops
            .iter()
            .enumerate()
            .filter_map(
                |(i, h)| {
                    if i % 2 == 1 || i == 0 { Some(h) } else { None }
                },
            )
            .zip(route.hops.iter());

        for (index, (original, (expected_endpoint, expected_channel, expected_fork))) in hop_pairs.enumerate()
        {
            // check endpoint
            if original.endpoint != expected_endpoint.clone() {
                return Err(RouteAssertionError::InvalidEndpointOnHop(
                    index as i32,
                    expected_endpoint.clone(),
                    original.endpoint.clone(),
                ));
            }
            // check channel
            if let Some(channel) = &expected_channel
                && original.socket.interface_name != Some(channel.clone())
            {
                return Err(RouteAssertionError::InvalidChannelOnHop(
                    index as i32,
                    channel.clone(),
                    original
                        .socket
                        .interface_name
                        .clone()
                        .unwrap_or("None".to_string()),
                ));
            }
            // check fork
            if let Some(fork) = expected_fork && &original.fork_nr != fork {
                return Err(RouteAssertionError::InvalidForkOnHop(
                    index as i32,
                    fork.clone(),
                    original.fork_nr.clone(),
                ));
            }
        }
    }

    Ok(())
}

#[derive(PartialEq)]
pub enum RouteAssertionError {
    InvalidEndpointOnHop(i32, Endpoint, Endpoint),
    InvalidChannelOnHop(i32, String, String),
    InvalidForkOnHop(i32, String, String),
    MissingResponse(Endpoint),
}
impl Debug for RouteAssertionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        core::write!(f, "{}", self)
    }
}
impl Display for RouteAssertionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouteAssertionError::InvalidEndpointOnHop(
                index,
                expected,
                actual,
            ) => {
                core::write!(
                    f,
                    "Expected hop {index} to be {expected} but was {actual}"
                )
            }
            RouteAssertionError::InvalidChannelOnHop(
                index,
                expected,
                actual,
            ) => {
                core::write!(
                    f,
                    "Expected hop {index} to be a channel {expected} but was a {actual}"
                )
            }
            RouteAssertionError::InvalidForkOnHop(index, expected, actual) => {
                core::write!(
                    f,
                    "Expected hop {index} to be a fork {expected} but was a {actual}"
                )
            }
            RouteAssertionError::MissingResponse(endpoint) => {
                core::write!(f, "No response received for endpoint {endpoint}")
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct NetworkNode {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    pub priority: i16,
    pub endpoint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NetworkData {
    pub nodes: Vec<NetworkNode>,
    pub edges: Vec<Edge>,
}

impl Network {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let current_dir =
            env::current_dir().expect("Failed to get current directory");
        let path = current_dir
            .join("tests/network/network-builder/networks/")
            .join(path);
        info!("Loading network from {}", path.display());

        let file_content =
            fs::read_to_string(path).expect("Failed to read the file");
        let network_data: NetworkData = serde_json::from_str(&file_content)
            .expect("Failed to deserialize the JSON");

        let mut nodes = Vec::new();
        let channel_names = network_data
            .edges
            .iter()
            .map(|edge| {
                let mut channel = [edge.source.clone(), edge.target.clone()];
                channel.sort();
                format!("{}_{}_{}", channel[0], edge.edge_type, channel[1])
            })
            .collect::<Vec<_>>();

        for network_node in network_data.nodes.iter() {
            let endpoint =
                Endpoint::from_str(&network_node.label.clone()).unwrap();
            let mut node = Node::new(endpoint);

            for edge in network_data.edges.iter() {
                let mut channel = [edge.source.clone(), edge.target.clone()];
                channel.sort();
                let channel =
                    format!("{}_{}_{}", channel[0], edge.edge_type, channel[1]);
                let is_bidirectional = channel_names
                    .iter()
                    .filter(|&item| item == &channel)
                    .count()
                    == 2;
                let is_outgoing = edge.source == network_node.id;

                if is_outgoing
                    || (edge.target == network_node.id && !is_bidirectional)
                {
                    info!(
                        "{} is_outgoing: {}, is_bidirectional: {}",
                        network_node.id, is_outgoing, is_bidirectional
                    );

                    let interface_direction = if is_bidirectional {
                        InterfaceDirection::InOut
                    } else if is_outgoing {
                        InterfaceDirection::Out
                    } else {
                        InterfaceDirection::In
                    };

                    let prio = {
                        if edge.priority >= 0
                            && interface_direction != InterfaceDirection::In
                        {
                            InterfacePriority::Priority(edge.priority as u16)
                        } else {
                            InterfacePriority::None
                        }
                    };

                    if edge.edge_type == "mockup" {
                        info!(
                            "Channel: {channel:?}, Direction: {interface_direction:?}"
                        );

                        let other_endpoint = edge
                            .endpoint
                            .as_deref()
                            .map(Endpoint::from_str)
                            .map(|e| e.unwrap());

                        if let Some(endpoint) = other_endpoint {
                            node = node.with_connection(InterfaceConnection::new(
                                &edge.edge_type,
                                prio,
                                MockupInterfaceSetupData {
                                    name: channel,
                                    endpoint: Some(endpoint.clone()),
                                    direction: interface_direction,
                                }
                            ));
                        } else {
                            node = node
                                .with_connection(InterfaceConnection::new(
                                &edge.edge_type,
                                prio,
                                MockupInterfaceSetupData {
                                    name: channel,
                                    endpoint: None,
                                    direction: interface_direction,
                                }
                            ));
                        }
                    }
                }
            }
            nodes.push(node);
        }
        Network::create(nodes)
    }

    pub fn create(endpoints: Vec<Node>) -> Self {
        // let mut mockup_interface_channels = HashMap::new();
        //
        // // iterate over all endpoints and handle mockup endpoints
        // for endpoint in endpoints.iter_mut() {
        //     for connection in endpoint.connections.iter_mut() {
        //         if connection.interface_type == "mockup" {
        //             Network::init_mockup_endpoint(
        //                 connection,
        //                 &mut mockup_interface_channels,
        //             );
        //         }
        //     }
        // }
        // info!(
        //     "Mockup channels: {:?}",
        //     mockup_interface_channels
        //         .values()
        //         .map(|c| c.is_some())
        //         .collect::<Vec<_>>()
        // );

        Network {
            is_initialized: false,
            endpoints,
        }
    }

    // fn init_mockup_endpoint(
    //     connection: &mut InterfaceConnection,
    //     mockup_interface_channels: &mut MockupInterfaceChannels,
    // ) {
    //     // get setup data as MockupInterfaceSetupData
    //     if let Some(setup_data) = &mut connection.setup_data {
    //         let channel = Network::get_mockup_interface_channel(
    //             mockup_interface_channels,
    //             setup_data.name.clone(),
    //         );
    //         match setup_data.direction {
    //             InterfaceDirection::In => {
    //                 setup_data.receiver_in = Some(channel.receiver);
    //             }
    //             InterfaceDirection::Out => {
    //                 setup_data.sender_out = Some(channel.sender);
    //             }
    //             InterfaceDirection::InOut => {
    //                 setup_data.sender_out = Some(channel.sender);
    //                 setup_data.receiver_in = Some(channel.receiver);
    //             }
    //         }
    //
    //         info!("setup_data: {:?}", setup_data);
    //     }
    // }

    fn get_mockup_interface_channel(
        mockup_interface_channels: &mut MockupInterfaceChannels,
        name: String,
    ) -> MockupInterfaceChannelEndpoint {
        if !mockup_interface_channels.contains_key(&name) {
            let (sender_a, receiver_a) = create_unbounded_channel::<Vec<u8>>();
            let (sender_b, receiver_b) = create_unbounded_channel::<Vec<u8>>();
            mockup_interface_channels.insert(
                name,
                Some(MockupInterfaceChannelEndpoint {
                    sender: sender_b,
                    receiver: receiver_a,
                }),
            );

            MockupInterfaceChannelEndpoint {
                sender: sender_a,
                receiver: receiver_b,
            }
        } else {
            match mockup_interface_channels.get_mut(&name).unwrap().take() {
                Some(channel) => channel,
                _ => {
                    core::panic!("Channel {name} is already used");
                }
            }
        }
    }

    pub async fn start(&mut self) {
        if self.is_initialized {
            core::panic!("Network already initialized");
        }
        self.is_initialized = true;

        let mut channel_pairs: HashMap<String, (Option<(Runtime, InterfaceConnection)>, Option<(Runtime, InterfaceConnection)>)> = HashMap::new();

        // iterate over all endpoints and set up runtimes
        for node in self.endpoints.iter_mut() {
            info!("creating runtime for endpoint {}", node.endpoint);
            let runtime = Runtime::create_native(
                RuntimeConfig::new_with_endpoint(node.endpoint.clone()),
            ).await;
            node.runtime = Some(runtime.clone());

            for connection in node.connections.iter() {
                // save in channel pairs
                let pairs = channel_pairs.entry(connection.setup_data.as_ref().unwrap().name.clone()).or_insert((None, None));
                if pairs.0.is_some() && pairs.1.is_some() {
                    panic!("Channel {} already has two endpoints", connection.setup_data.as_ref().unwrap().name);
                }
                else if pairs.0.is_none() {
                    *pairs = (Some((runtime.clone(), connection.clone())), None);
                }
                else {
                    *pairs = (pairs.0.take(), Some((runtime.clone(), connection.clone())));
                }
            }
        }

        // iterate over all connection pairs and set up
        let mut iterator = channel_pairs.iter();
        while let Some((channel_name, (Some((runtime_a, connection_a)), Some((runtime_b, connection_b))))) = iterator.next() {
            info!("{}: ({} : {:?} , {} : {:?})", channel_name, runtime_a.endpoint(), connection_a.priority, runtime_b.endpoint(), connection_b.priority);

            Network::create_connection(
                (runtime_a.clone(), connection_a.clone()),
                (runtime_b.clone(), connection_b.clone()),
            );
            yield_now().await;
        }

        // print com hub status of each runtime
        for endpoint in self.endpoints.iter() {
            let runtime = endpoint.runtime.as_ref().unwrap();
            runtime.com_hub().print_metadata();
        }
    }

    // create connection between two runtimes via a defined channel
    // TODO: cleanup this mess, works but ugly
    fn create_connection(
        runtime_a: (Runtime, InterfaceConnection),
        runtime_b: (Runtime, InterfaceConnection),
    ) {
        let interface_a_direction = runtime_a.1.setup_data.as_ref().unwrap().direction.clone();
        let (proxy_a, com_interface_a) = ComInterfaceProxy::create_interface(
            InterfaceProperties {
                interface_type: "mockup".to_string(),
                direction: interface_a_direction.clone(),
                name: Some(runtime_a.1.setup_data.as_ref().unwrap().name.clone()),
                ..Default::default()
            }
        );
        let interface_a_can_send = com_interface_a.0.properties.borrow().can_send();
        runtime_a.0.com_hub().register_com_interface(com_interface_a, runtime_a.1.priority).expect("Failed to register interface A");

        let interface_b_direction = runtime_b.1.setup_data.as_ref().unwrap().direction.clone();
        let (proxy_b, com_interface_b) = ComInterfaceProxy::create_interface(
            InterfaceProperties {
                interface_type: "mockup".to_string(),
                direction: interface_b_direction.clone(),
                name: Some(runtime_b.1.setup_data.as_ref().unwrap().name.clone()),
                ..Default::default()
            }
        );
        let interface_b_can_send = com_interface_b.0.properties.borrow().can_send();
        runtime_b.0.com_hub().register_com_interface(com_interface_b, runtime_b.1.priority).expect("Failed to register interface B");

        // connect the two interfaces
        let remote_endpoint_a = runtime_b.1.endpoint.clone();
        let remote_endpoint_b = runtime_a.1.endpoint.clone();


        // Forward events from proxy A to proxy B
        let shutdown_signal_a = proxy_a.shutdown_signal();
        let (_, mut socket_a_sender) = proxy_a
            .create_and_init_socket_with_optional_endpoint(
                interface_a_direction,
                1,
                remote_endpoint_a,
            );

        // Forward events from proxy B to proxy A
        let shutdown_signal_b = proxy_b.shutdown_signal();
        let (_, mut socket_b_sender) = proxy_b
            .create_and_init_socket_with_optional_endpoint(
                interface_b_direction,
                1,
                remote_endpoint_b,
            );

        if interface_a_can_send {
            spawn_with_panic_notify_default(async move {
                let mut event_receiver_a = proxy_a.event_receiver;
                loop {
                    tokio::select! {
                        Some(event) = event_receiver_a.next() => {
                            if let ComInterfaceEvent::SendBlock(block, _socket_uuid) = event {
                                // directly send the block to socket B
                                socket_b_sender.start_send(block).unwrap();
                            }
                        }
                        _ = shutdown_signal_a.notified() => {
                            break;
                        }
                    }
                }
            });
        }

        if interface_b_can_send {
            spawn_with_panic_notify_default(async move {
                let mut event_receiver_b = proxy_b.event_receiver;
                loop {
                    tokio::select! {
                        Some(event) = event_receiver_b.next() => {
                            if let ComInterfaceEvent::SendBlock(block, _socket_uuid) = event {
                                // directly send the block to socket A
                                socket_a_sender.start_send(block).unwrap();
                            }
                        }
                        _ = shutdown_signal_b.notified() => {
                            break;
                        }
                    }
                }
            });
        }

    }


    pub fn get_runtime(&self, endpoint: impl Into<Endpoint>) -> &Runtime {
        let endpoint = endpoint.into();
        for node in self.endpoints.iter() {
            if node.endpoint == endpoint {
                return node.runtime.as_ref().unwrap();
            }
        }
        core::panic!("Endpoint {endpoint} not found in network");
    }
}
