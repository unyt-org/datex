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
            (ValueContainer::Shared(lhs), ValueContainer::Shared(rhs)) => {
                lhs.collapsed_value().borrow().as_ref()
                    - rhs.collapsed_value().borrow().as_ref()
            }
            (ValueContainer::Local(lhs), ValueContainer::Shared(rhs)) => {
                lhs - rhs.collapsed_value().borrow().as_ref()
            }
            (ValueContainer::Shared(lhs), ValueContainer::Local(rhs)) => {
                lhs.collapsed_value().borrow().as_ref() - rhs
            }
        }
        .map(ValueContainer::Local)
    }
}
