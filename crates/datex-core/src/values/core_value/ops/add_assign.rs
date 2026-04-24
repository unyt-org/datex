use crate::values::core_value::CoreValue;
use core::ops::AddAssign;

impl AddAssign<CoreValue> for CoreValue {
    fn add_assign(&mut self, rhs: CoreValue) {
        let res = self.clone() + rhs;
        if let Ok(value) = res {
            *self = value;
        } else {
            core::panic!("Failed to add value: {res:?}");
        }
    }
}
