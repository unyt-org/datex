use core::marker::PhantomData;
use crate::dif::cache::DIFSharedContainerCache;

pub struct DeserializationContext<'ctx, T> {
    pub shared_container_cache: &'ctx mut DIFSharedContainerCache,
    _marker: PhantomData<T>,
}

impl<'ctx, T> DeserializationContext<'ctx, T> {
    pub fn new(shared_container_cache: &'ctx mut DIFSharedContainerCache) -> Self {
        Self { shared_container_cache, _marker: PhantomData }
    }

    // Converts this deserialization context to a deserialization context for another type U
    pub fn cast<U>(&mut self) -> DeserializationContext<'_, U> {
        DeserializationContext::new(self.shared_container_cache)
    }
}


