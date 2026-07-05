use crate::{
    shared_values::SharedContainer, traits::clone_unsafe::CloneUnsafe,
};

impl CloneUnsafe for SharedContainer {
    /// Creates a new owned [SharedContainer] with the same contents.
    /// # Safety
    /// The caller must ensure that the original self is not used later
    /// or that the newly created shared container is only used internally and dropped afterward.
    unsafe fn clone_unsafe(&self) -> Self {
        match self {
            SharedContainer::Owned(owned) => {
                SharedContainer::Owned(unsafe { owned.clone_unsafe() })
            }
            SharedContainer::Referenced(referenced) => {
                SharedContainer::Referenced(referenced.clone())
            }
        }
    }
}
