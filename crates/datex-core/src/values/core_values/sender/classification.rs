use crate::{
    traits::classification::Classification,
    values::core_values::native::DatexNativeBase,
};
use crate::values::core_values::sender::Sender;

impl<T> Classification for Sender<T> where T: DatexNativeBase + 'static {}
