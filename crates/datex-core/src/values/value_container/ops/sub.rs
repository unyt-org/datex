use crate::values::value_container::{ValueContainer, error::ValueError};
use core::{ops::Sub, result::Result};

impl Sub<ValueContainer> for ValueContainer {
    type Output = Result<ValueContainer, ValueError>;

    fn sub(self, rhs: ValueContainer) -> Self::Output {
        (&self).sub(&rhs)
    }
}

impl Sub<&ValueContainer> for &ValueContainer {
    type Output = Result<ValueContainer, ValueError>;

    fn sub(self, rhs: &ValueContainer) -> Self::Output {
        match (self, rhs) {
            (ValueContainer::Local(lhs), ValueContainer::Local(rhs)) => {
                lhs - rhs
            }
            (ValueContainer::Shared(lhs), ValueContainer::Shared(rhs)) => lhs
                .with_collapsed_value_mut(|lhs| {
                    rhs.with_collapsed_value_mut(|rhs| {
                        lhs.clone() - rhs.clone()
                    })
                }),
            (ValueContainer::Local(lhs), ValueContainer::Shared(rhs)) => {
                rhs.with_collapsed_value_mut(|rhs| lhs - rhs)
            }
            (ValueContainer::Shared(lhs), ValueContainer::Local(rhs)) => {
                lhs.with_collapsed_value_mut(|lhs| lhs.clone() - rhs.clone())
            }
        }
        .map(ValueContainer::Local)
    }
}
