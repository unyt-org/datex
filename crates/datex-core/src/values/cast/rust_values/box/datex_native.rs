use core::any::Any;
use crate::preludes::derive::{DatexNative, HasClassification};
use crate::traits::get_datex_type::GetDatexType;

impl<T: DatexNative + GetDatexType + HasClassification> DatexNative
for Box<T>
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
