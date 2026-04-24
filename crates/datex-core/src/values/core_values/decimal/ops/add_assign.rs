use crate::values::core_values::decimal::typed_decimal::TypedDecimal;
use core::ops::{Add, AddAssign};
impl AddAssign for TypedDecimal {
    fn add_assign(&mut self, rhs: Self) {
        *self = TypedDecimal::add(self.clone(), rhs);
    }
}
