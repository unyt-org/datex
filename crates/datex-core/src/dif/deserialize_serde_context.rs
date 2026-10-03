use core::marker::PhantomData;
use serde::de::DeserializeSeed;
use serde::Deserializer;
use crate::dif::deserialize_with_serde_context::DeserializeWithSerdeContext;
use crate::dif::serde_context::SerdeContext;

pub struct DeserializeSerdeContext<'a, 'ctx, T> {
    pub(crate) ctx: &'a SerdeContext<'ctx>,
    _marker: PhantomData<T>
}

impl<'a, 'ctx, T> DeserializeSerdeContext<'a, 'ctx, T> {
    pub fn new(ctx: &'a SerdeContext<'ctx>) -> Self {
        Self { ctx, _marker: PhantomData }
    }

    pub fn cast<U>(&self) -> DeserializeSerdeContext<'a, 'ctx, U> {
        DeserializeSerdeContext {
            ctx: self.ctx,
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for DeserializeSerdeContext<'_, '_, T> {
    fn clone(&self) -> Self { *self }
}
impl<T> Copy for DeserializeSerdeContext<'_, '_, T> {}


impl<'de, T: DeserializeWithSerdeContext<'de>> DeserializeSeed<'de> for DeserializeSerdeContext<'_, '_, T>
{
    type Value = T;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<T, D::Error> {
        T::deserialize_with_ctx(self.ctx, deserializer)
    }
}