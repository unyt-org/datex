use core::ops::Neg;

use crate::values::core_values::decimal::rational::Rational;

impl Neg for Rational {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Rational::from_big_rational(-self.big_rational)
    }
}
