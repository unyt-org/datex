use crate::{
    traits::{structural_eq::StructuralEq, value_eq::ValueEq},
    values::core_values::decimal::Decimal,
};

impl StructuralEq for Decimal {
    fn structural_eq(&self, other: &Self) -> bool {
        if self.is_zero() && other.is_zero() {
            return true; // +0.0 == -0.0
        }
        match (self, other) {
            (Decimal::Finite(a), Decimal::Finite(b)) => a == b,
            (Decimal::Infinity, Decimal::Infinity) => true,
            (Decimal::NegInfinity, Decimal::NegInfinity) => true,
            (Decimal::Nan, Decimal::Nan) => false,
            _ => false,
        }
    }
}

impl ValueEq for Decimal {
    fn value_eq(&self, other: &Self) -> bool {
        self.structural_eq(other)
    }
}
