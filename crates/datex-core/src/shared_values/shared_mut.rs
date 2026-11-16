use crate::shared_values::SharedContainer;
use core::{
    cell::RefMut,
    ops::{Deref, DerefMut},
};

/// Wrapper around a RefMut used for accessing mutable references to values of [SharedContainer]s
pub struct SharedMut<'a, T> {
    ref_mut: Option<RefMut<'a, T>>,
    container: &'a SharedContainer,
}

impl<'a, T> SharedMut<'a, T> {
    pub fn new(ref_mut: RefMut<'a, T>, container: &'a SharedContainer) -> Self {
        Self {
            ref_mut: Some(ref_mut),
            container,
        }
    }
}

impl<T> Drop for SharedMut<'_, T> {
    fn drop(&mut self) {
        // drop the ref_mut directly and notify the container that the borrow has been dropped
        self.ref_mut.take();
        self.container.notify_borrow_dropped();
    }
}

impl<T> Deref for SharedMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.ref_mut.as_ref().unwrap()
    }
}

impl<T> DerefMut for SharedMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ref_mut.as_mut().unwrap()
    }
}
