use core::prelude::rust_2024::*;

use serde::Serialize;

use crate::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    global::dxb_block::DXBBlock,
    network::com_interfaces::{
        block_collector::BlockCollector,
        com_interface::{ComInterfaceUUID, properties::InterfaceDirection},
    },
    runtime::AsyncContext,
    stdlib::{string::String, string::ToString, vec::Vec},
    utils::{once_consumer::OnceConsumer, uuid::UUID},
    values::core_values::endpoint::Endpoint,
};
use core::fmt::Display;

#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm_runtime", tsify(type = "string"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComInterfaceSocketUUID(pub(crate) UUID);
impl Display for ComInterfaceSocketUUID {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::write!(f, "socket::{}", self.0)
    }
}

impl TryFrom<String> for ComInterfaceSocketUUID {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.strip_prefix("socket::").ok_or(())?;
        Ok(ComInterfaceSocketUUID(UUID::from_string(value.to_string())))
    }
}

impl Serialize for ComInterfaceSocketUUID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}
impl<'de> serde::Deserialize<'de> for ComInterfaceSocketUUID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ComInterfaceSocketUUID::try_from(s).map_err(|_| {
            serde::de::Error::custom("Invalid ComInterfaceSocketUUID")
        })
    }
}

#[derive(Debug)]
pub enum ComInterfaceSocketEvent {
    NewSocket(ComInterfaceSocket),
    /// indicates that the socket can no longer be used and should be removed
    /// optionally includes a block that could not be sent out since the socket can no longer be used
    /// This event may be triggered multiple times for the same socket close if asynchronously scheduled
    /// blocks are still being processed after the first close event.
    CloseSocket(ComInterfaceSocketUUID, Option<DXBBlock>),
}

#[derive(Debug)]
pub struct ComInterfaceSocket {
    pub direct_endpoint: Option<Endpoint>,
    pub uuid: ComInterfaceSocketUUID,
    pub interface_uuid: ComInterfaceUUID,
    pub connection_timestamp: u64,
    pub channel_factor: u32,
    pub direction: InterfaceDirection,
}

impl ComInterfaceSocket {
    pub fn can_send(&self) -> bool {
        self.direction == InterfaceDirection::Out
            || self.direction == InterfaceDirection::InOut
    }

    pub fn can_receive(&self) -> bool {
        self.direction == InterfaceDirection::In
            || self.direction == InterfaceDirection::InOut
    }

    /// Initializes a new ComInterfaceSocket, starts the BlockCollector task.
    pub fn init(
        interface_uuid: ComInterfaceUUID,
        direction: InterfaceDirection,
        channel_factor: u32,
        direct_endpoint: Option<Endpoint>,
        async_context: &AsyncContext,
    ) -> (ComInterfaceSocket, UnboundedSender<Vec<u8>>) {
        let (bytes_in_sender, block_in_receiver) =
            BlockCollector::init(async_context);
        (
            ComInterfaceSocket {
                direct_endpoint,
                uuid: ComInterfaceSocketUUID(UUID::new()),
                interface_uuid,
                connection_timestamp: 0,
                channel_factor,
                direction,
            },
            bytes_in_sender,
        )
    }
}
