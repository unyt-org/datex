use crate::shared_values::OwnedSharedContainer;
use crate::traits::clone_unsafe::CloneUnsafe;

impl CloneUnsafe for OwnedSharedContainer {
    unsafe fn clone_unsafe(&self) -> Self {
        OwnedSharedContainer {
            inner: self.inner.clone(),
        }
    }
}