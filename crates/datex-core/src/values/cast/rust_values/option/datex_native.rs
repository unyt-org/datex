use core::any::Any;
use crate::preludes::derive::{DatexNative, SharedReferencesCache, Type};
use crate::traits::get_datex_type::GetDatexType;

impl<T: DatexNative + GetDatexType> DatexNative
for Option<T>
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn value_datex_type(&self, cache: &mut SharedReferencesCache) -> Type {
        <Self as GetDatexType>::datex_type(cache)
    }
}
