use crate::{
    traits::classification::Classification,
    values::core_values::{native::DatexNativeBase, sender::Sender},
};

impl<T> Classification for Sender<T> where T: DatexNativeBase + 'static {}
