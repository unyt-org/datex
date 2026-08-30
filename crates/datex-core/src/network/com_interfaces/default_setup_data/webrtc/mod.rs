use crate::{
    macros::Datex,
    network::com_interfaces::com_interface::properties::ComInterfaceProperties,
    prelude::*,
};

/// Represents an ICE candidate initialization message in WebRTC.
#[derive(Datex, Default, Debug)]
#[datex(only_structural)]
pub struct RTCIceCandidateInitDX {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
    pub username_fragment: Option<String>,
}

/// Represents the type of a WebRTC session description.
#[derive(Datex, Default, Debug, PartialEq, Eq)]
#[datex(only_structural)]
pub enum RTCSdpTypeDX {
    #[default]
    Unspecified,
    #[datex(rename = "answer")]
    Answer,
    #[datex(rename = "offer")]
    Offer,
}

/// Represents a WebRTC session description.
#[derive(Datex, Default, Debug, PartialEq, Eq)]
#[datex(only_structural)]
pub struct RTCSessionDescriptionDX {
    #[datex(rename = "type")]
    pub sdp_type: RTCSdpTypeDX,
    pub sdp: String,
}

/// Represents an ICE server configuration for WebRTC.
#[derive(Datex, Default, Clone, Debug)]
#[datex(only_structural)]
pub struct RTCIceServerDX {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

/// Represents the role of a WebRTC participant in a connection.
#[derive(Datex, Debug, Default, PartialEq, Eq)]
#[datex(only_structural)]
pub enum WebRTCRoleDX {
    #[default]
    Offerer,
    Answerer,
}

/// Represents the setup data required for establishing a WebRTC interface.
#[derive(Datex, Debug)]
#[datex(only_structural)]
pub struct WebRTCInterfaceSetupData {
    /// The role of the WebRTC participant (Offerer or Answerer).
    pub role: WebRTCRoleDX,
    /// The label for the data channel to be used in the WebRTC connection (default is datex)
    pub data_channel_label: String,
    /// A list of ICE servers to be used for establishing the WebRTC connection.
    pub ice_servers: Vec<RTCIceServerDX>,
    /// The negotiated data channel ID, if any. If None, a new data channel will be created.
    pub negotiated_data_channel_id: Option<u16>,
    /// If true, the data channel will be ordered. If false, the data channel will be unordered.
    pub ordered: bool,
}

impl Default for WebRTCInterfaceSetupData {
    fn default() -> Self {
        Self {
            role: WebRTCRoleDX::Offerer,
            data_channel_label: "datex".to_string(),
            ice_servers: vec![RTCIceServerDX {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            }],
            negotiated_data_channel_id: None,
            ordered: true,
        }
    }
}

impl WebRTCInterfaceSetupData {
    pub fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "webrtc".to_string(),
            channel: "webrtc".to_string(),
            round_trip_time: 40,
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }
}

#[derive(Datex, Debug)]
#[datex(only_structural)]
pub enum WebRTCSignalDX {
    Description(RTCSessionDescriptionDX),
    IceCandidate(RTCIceCandidateInitDX),
    EndOfCandidates,
}

use core::pin::Pin;
pub type WebRTCSignalResult<T> = Result<T, String>;

/// FIXME: Replace with pointer callable once functions work
pub trait WebRTCSignaling: Send + Sync + 'static {
    fn send(
        &self,
        signal: WebRTCSignalDX,
    ) -> Pin<Box<dyn Future<Output = WebRTCSignalResult<()>> + Send>>;
    fn receive(
        &self,
    ) -> Pin<Box<dyn Future<Output = WebRTCSignalResult<WebRTCSignalDX>> + Send>>;
}
