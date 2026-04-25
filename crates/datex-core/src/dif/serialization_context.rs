use crate::dif::{cache::DIFSharedContainerCache, dif_interface::DIFInterface};
use alloc::rc::Rc;
use core::{cell::RefCell, marker::PhantomData};

pub struct SerializationContext<T> {
    pub dif_interface: Rc<RefCell<DIFInterface>>,
    _marker: PhantomData<T>,
}

impl<T> SerializationContext<T> {
    pub fn new(dif_interface: Rc<RefCell<DIFInterface>>) -> Self {
        Self {
            dif_interface,
            _marker: PhantomData,
        }
    }

    // Converts this deserialization context to a deserialization context for another type U
    pub fn cast<U>(&mut self) -> SerializationContext<U> {
        SerializationContext::new(self.dif_interface.clone())
    }
}
