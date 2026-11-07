//! Instruction to bytes conversion helpers.

/// Trait for converting instructions into a vector of bytes.
pub trait ToBytes {
    /// Converts the implementing type into a vector of bytes.
    fn to_bytes(&self) -> Vec<u8>;
}
