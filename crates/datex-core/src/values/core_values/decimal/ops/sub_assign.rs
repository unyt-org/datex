use crate::values::core_values::decimal::{
    Decimal, typed_decimal::TypedDecimal,
};
use core::ops::{Sub, SubAssign};
impl SubAssign for TypedDecimal {
    fn sub_assign(&mut self, rhs: Self) {
        *self = TypedDecimal::sub(self.clone(), rhs);
    }
}
impl SubAssign for Decimal {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Decimal::sub(self.clone(), rhs);
    }
}
impl SubAssign<&Decimal> for Decimal {
    fn sub_assign(&mut self, rhs: &Decimal) {
        *self = Decimal::sub(self.clone(), rhs.clone());
    }
}
impl SubAssign<&TypedDecimal> for TypedDecimal {
    fn sub_assign(&mut self, rhs: &TypedDecimal) {
        *self = TypedDecimal::sub(self.clone(), rhs.clone());
    }
}
