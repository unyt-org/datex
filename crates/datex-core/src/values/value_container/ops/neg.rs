use crate::values::value_container::{ValueContainer, error::ValueError};
use core::{ops::Neg, result::Result};

impl Neg for ValueContainer {
    type Output = Result<ValueContainer, ValueError>;

    fn neg(self) -> Self::Output {
        match self {
            ValueContainer::Local(value) => (-value).map(ValueContainer::Local),
            ValueContainer::Shared(reference) => reference
                .with_collapsed_value_mut(|value| {
                    (-value.clone()).map(ValueContainer::Local)
                }),
        }
    }
}
