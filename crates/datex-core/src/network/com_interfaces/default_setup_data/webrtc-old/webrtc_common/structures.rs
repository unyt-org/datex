use datex_macros_internal::Datex;

#[derive(Datex, Default, Debug, Clone, PartialEq, Hash)]
#[datex(structural_recursive)]
pub struct RTCIceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}
impl RTCIceServer {
    pub fn new(urls: Vec<String>) -> Self {
        Self {
            urls,
            username: None,
            credential: None,
        }
    }
}
impl RTCIceServer {
    pub fn with_username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }
    pub fn with_credential(mut self, credential: String) -> Self {
        self.credential = Some(credential);
        self
    }
}

#[derive(Datex, Default, Debug, Clone, PartialEq, Eq)]
#[datex(structural_recursive)]
pub struct RTCIceCandidateInitDX {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
    pub username_fragment: Option<String>,
}

#[derive(Datex, Default, Debug, Clone, PartialEq)]
#[datex(structural_recursive)]
pub enum RTCSdpTypeDX {
    #[default]
    Unspecified,
    #[datex(rename = "answer")]
    Answer,
    #[datex(rename = "offer")]
    Offer,
}

#[derive(Datex, Default, Debug, Clone)]
#[datex(structural_recursive)]
pub struct RTCSessionDescriptionDX {
    #[datex(rename = "type")]
    pub sdp_type: RTCSdpTypeDX,
    pub sdp: String,
}
