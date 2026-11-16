use core::ops::{Sub, SubAssign};

use crate::values::core_values::integer::typed_integer::TypedInteger;
impl SubAssign for TypedInteger {
    // FIXME error handling / wrapping if out of bounds
    fn sub_assign(&mut self, rhs: Self) {
        *self = TypedInteger::sub(self.clone(), rhs).expect("Failed to add");
    }
}
