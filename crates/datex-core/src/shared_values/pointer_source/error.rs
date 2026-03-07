use crate::prelude::*;
#[derive(Debug)]
pub enum PointerSourceError {
    NotFound,
    Unavailable,
    Unsupported,
    Conflict,
    InvalidPointer,
    Backend(String),
}
impl core::fmt::Display for PointerSourceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PointerSourceError::NotFound => {
                core::write!(f, "Pointer not found")
            }
            PointerSourceError::Unavailable => {
                core::write!(f, "Pointer source unavailable")
            }
            PointerSourceError::Unsupported => {
                core::write!(f, "Operation not supported by pointer source")
            }
            PointerSourceError::Conflict => {
                core::write!(f, "Pointer conflict")
            }
            PointerSourceError::InvalidPointer => {
                core::write!(f, "Invalid pointer")
            }
            PointerSourceError::Backend(e) => {
                core::write!(f, "Backend error: {}", e)
            }
        }
    }
}
