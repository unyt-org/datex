use crate::{
    traits::classification::Classification,
    values::core_values::{native::DatexNativeBase, receiver::Receiver},
};

impl<T> Classification for Receiver<T> where T: DatexNativeBase + 'static {}
