use core::marker::PhantomData;
use crate::runtime::memory::Memory;

pub struct DeserializationContext<'ctx, T> {
    pub memory: &'ctx Memory,
    _marker: PhantomData<T>,
}

impl<'ctx, T> DeserializationContext<'ctx, T> {
    pub fn new(memory: &'ctx Memory) -> Self {
        Self { memory, _marker: PhantomData }
    }

    // Converts this deserialization context to a deserialization context for another type U
    pub fn cast<U>(&self) -> DeserializationContext<'ctx, U> {
        DeserializationContext::new(self.memory)
    }
}


