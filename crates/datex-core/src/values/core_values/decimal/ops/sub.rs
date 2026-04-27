use crate::values::core_values::decimal::{
    Decimal, typed_decimal::TypedDecimal,
};
use core::ops::{Add, Neg, Sub};

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl Sub for &Decimal {
    type Output = Decimal;

    fn sub(self, rhs: Self) -> Self::Output {
        // FIXME #335: Avoid cloning, as sub should be applicable for refs only
        Decimal::sub(self.clone(), rhs.clone())
    }
}

impl Sub for TypedDecimal {
    type Output = TypedDecimal;

    fn sub(self, rhs: Self) -> Self::Output {
        // negate rhs
        let negated_rhs = match rhs {
            TypedDecimal::F32(value) => TypedDecimal::F32(value.neg()),
            TypedDecimal::F64(value) => TypedDecimal::F64(value.neg()),
            TypedDecimal::Decimal(value) => TypedDecimal::Decimal(value.neg()),
        };

        // perform addition with negated rhs
        TypedDecimal::add(self, negated_rhs)
    }
}
impl Sub for &TypedDecimal {
    type Output = TypedDecimal;

    fn sub(self, rhs: Self) -> Self::Output {
        // FIXME #340: Avoid cloning, as sub should be applicable for refs only
        TypedDecimal::sub(self.clone(), rhs.clone())
    }
}
