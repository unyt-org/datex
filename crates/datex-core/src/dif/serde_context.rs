use crate::dif::cache::DIFSharedContainerCache;
use core::marker::PhantomData;

pub struct SerdeContext<'ctx, T> {
    pub shared_container_cache: &'ctx mut DIFSharedContainerCache,
    _marker: PhantomData<T>,
}

impl<'ctx, T> SerdeContext<'ctx, T> {
    pub fn new(
        shared_container_cache: &'ctx mut DIFSharedContainerCache,
    ) -> Self {
        Self {
            shared_container_cache,
            _marker: PhantomData,
        }
    }

    // Converts this deserialization context to a deserialization context for another type U
    pub fn cast<U>(&mut self) -> SerdeContext<'_, U> {
        SerdeContext::new(self.shared_container_cache)
    }
}
