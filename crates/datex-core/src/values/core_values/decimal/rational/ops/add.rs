use core::ops::Add;

use crate::values::core_values::decimal::rational::Rational;

impl Add for Rational {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Rational::from_big_rational(self.big_rational + rhs.big_rational)
    }
}
