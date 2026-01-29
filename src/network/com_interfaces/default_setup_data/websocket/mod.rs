use core::fmt::Display;
use url::Url;

pub mod websocket_client;
pub mod websocket_server;



#[derive(Debug)]
pub enum URLError {
    InvalidURL,
    InvalidScheme,
}
impl Display for URLError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            URLError::InvalidURL => core::write!(f, "URLError: Invalid URL"),
            URLError::InvalidScheme => {
                core::write!(f, "URLError: Invalid URL scheme")
            }
        }
    }
}

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
