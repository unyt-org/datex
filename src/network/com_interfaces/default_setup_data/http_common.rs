use alloc::format;
use serde::Serialize;
use url::Url;
use crate::network::com_hub::errors::ComInterfaceCreateError;
use crate::network::com_interfaces::default_setup_data::websocket::URLError;
use crate::runtime::RuntimeConfigInterface;
use crate::serde::Deserialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub enum TLSMode {
    /// The TLS certificate is handled externally (e.g., by a reverse proxy or load balancer).
    HandledExternally,
    /// The server must handle TLS using the provided certificate.
    WithCertificate {
        private_key: Vec<u8>,
        certificate: Vec<u8>,
    },
}

pub type AcceptAddress = (String, Option<TLSMode>);
pub type AcceptAddresses = Vec<AcceptAddress>;

/// Parses a WebSocket URL and returns a `Url` object.
/// If no protocol is specified, it defaults to `ws` or `wss` based on the `secure` parameter.
pub fn parse_url(address: &str) -> Result<Url, URLError> {
    let mut url = Url::parse(address).map_err(|_| URLError::InvalidURL)?;
    match url.scheme() {
        "https" => url.set_scheme("wss").unwrap(),
        "http" => url.set_scheme("ws").unwrap(),
        "wss" | "ws" => (),
        _ => return Err(URLError::InvalidScheme),
    }
    Ok(url)
}


/// Generates the setup data for client interfaces based on the server's accept addresses.
pub fn get_clients_setup_data<T: Serialize>(
    accept_addresses: Option<AcceptAddresses>,
    protocols: (String, String),
    interface_type: String,
    generate_client_interface: fn(String) -> T,
) -> Result<Option<Vec<RuntimeConfigInterface>>, ComInterfaceCreateError> {
    accept_addresses.map(|addrs| {
        addrs
            .into_iter()
            .map(|(address, tls_mode)| {
                let url = format!(
                    "{}://{}",
                    if tls_mode.is_some() { protocols.1.clone() } else { protocols.0.clone() },
                    address
                );
                // parse and validate URL
                parse_url(&url).map_err(|_| {
                    ComInterfaceCreateError::invalid_setup_data(
                        format!("Invalid URL for WebSocket connection: {url}")
                    )
                })?;
                RuntimeConfigInterface::new(
                    interface_type.as_str(),
                    generate_client_interface(url),
                ).map_err(|e| {
                    ComInterfaceCreateError::invalid_setup_data(
                        format!("Failed to create connectable interface for WebSocket client: {e}")
                    )
                })
            })
            .collect::<_>()
    }).transpose()
}