use crate::values::core_values::decimal::{
    Decimal, typed_decimal::TypedDecimal,
};
use core::ops::{Add, AddAssign};
impl AddAssign for TypedDecimal {
    fn add_assign(&mut self, rhs: Self) {
        *self = TypedDecimal::add(self.clone(), rhs);
    }
}
impl AddAssign for Decimal {
    fn add_assign(&mut self, rhs: Self) {
        *self = Decimal::add(self.clone(), rhs);
    }
}
impl AddAssign<&Decimal> for Decimal {
    fn add_assign(&mut self, rhs: &Decimal) {
        *self = Decimal::add(self.clone(), rhs.clone());
    }
}
impl AddAssign<&TypedDecimal> for TypedDecimal {
    fn add_assign(&mut self, rhs: &TypedDecimal) {
        *self = TypedDecimal::add(self.clone(), rhs.clone());
    }
}
