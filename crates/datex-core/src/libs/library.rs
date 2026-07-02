use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

pub trait Library {
    /// Loads the library into the [SharedReferencesCache]
    /// # Safety
    /// The caller must guarantee that load is not called multiple times on the same memory for a library
    unsafe fn load(memory: &mut SharedReferencesCache);
}
