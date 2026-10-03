use crate::dif::serde_context::SerdeContext;
use crate::preludes::derive::SerializeWithSerdeContext;

#[derive(Debug)]
pub struct ValueWithSerdeContext<'a, 'ctx, V: ?Sized> {
    pub(crate) value: &'a V,
    pub(crate) ctx: &'a SerdeContext<'ctx>,
}


impl<'a, 'ctx, V: ?Sized + SerializeWithSerdeContext> ValueWithSerdeContext<'a, 'ctx, V> {
    pub fn new(value: &'a V, ctx: &'a SerdeContext<'ctx>) -> Self {
        Self { value, ctx }
    }
}
