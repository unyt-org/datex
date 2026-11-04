use crate::{
    traits::classification::Classification,
    values::core_values::native::DatexNativeBase,
};
use crate::values::core_values::receiver::Receiver;

impl<T> Classification for Receiver<T> where T: DatexNativeBase + 'static {}
