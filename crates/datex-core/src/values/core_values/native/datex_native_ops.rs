use crate::prelude::*;
use crate::values::core_values::native::DatexNative;

pub trait DatexNativeOps {
    fn add_native(
        &self,
        _rhs: &dyn DatexNative,
    ) -> Option<Box<dyn DatexNative>> {
        None
    }
}
