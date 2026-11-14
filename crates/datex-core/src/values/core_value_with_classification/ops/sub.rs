use core::ops::Sub;

use crate::values::{
    core_value_with_classification::CoreValueWithClassification,
    value_container::error::ValueError,
};

impl Sub for &CoreValueWithClassification {
    type Output = Result<CoreValueWithClassification, ValueError>;
    fn sub(self, rhs: &CoreValueWithClassification) -> Self::Output {
        Ok((&self.inner - &rhs.inner)?.into())
    }
}

impl Sub for CoreValueWithClassification {
    type Output = Result<CoreValueWithClassification, ValueError>;
    fn sub(self, rhs: CoreValueWithClassification) -> Self::Output {
        &self - &rhs
    }
}
