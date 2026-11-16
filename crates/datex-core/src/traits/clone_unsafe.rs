pub trait CloneUnsafe {
    /// Creates a new value with the same contents.
    /// This is used to allow exact clones of [SharedContainer]s, preserving ownership even for owned containers.
    /// # Safety
    /// The caller must ensure that the original self is not used later
    /// or that the newly created shared container is only used internally and dropped afterward.
    unsafe fn clone_unsafe(&self) -> Self;
}
