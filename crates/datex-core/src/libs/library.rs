use crate::runtime::memory::Memory;

pub trait Library {
    /// Loads the library into the [Memory]
    /// The caller must guarantee that load is not called multiple times on the same memory for a library
    unsafe fn load(memory: &mut Memory);
}
