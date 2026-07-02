use core::ops::AddAssign;

use crate::values::core_values::integer::typed_integer::TypedInteger;
use core::ops::Add;
impl AddAssign for TypedInteger {
    // FIXME #345 error handling / wrapping if out of bounds
    fn add_assign(&mut self, rhs: Self) {
        *self = TypedInteger::add(self.clone(), rhs).expect("Failed to add");
    }
}
