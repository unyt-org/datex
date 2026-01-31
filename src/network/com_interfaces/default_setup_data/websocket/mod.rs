use core::fmt::Display;

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