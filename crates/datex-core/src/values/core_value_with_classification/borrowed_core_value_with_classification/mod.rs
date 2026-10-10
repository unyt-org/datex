use crate::values::{
    core_value_with_classification::CoreValueWithClassification,
    value::{
        value_classification::ValueClassification,
    },
};
use crate::values::core_value::borrowed_core_value::{BorrowedCoreValue, BorrowedCoreValueMut};

#[cfg(feature = "ast")]
mod to_datex_expression_data;
pub mod classification;
pub mod hash;

/// Similar to [CoreValueWithClassification], but contains a [BorrowedCoreValue] instead of a [CoreValue].
/// It is used to represent a potentially borrowed reference to a [CoreValue] variant instead of owning it.
#[derive(Debug)]
pub struct BorrowedCoreValueWithClassification<'a> {
    pub inner: BorrowedCoreValue<'a>,
    pub classification: ValueClassification,
}

impl<'a> BorrowedCoreValueWithClassification<'a> {
    pub(crate) fn try_clone_to_core_value_with_classification(
        self,
    ) -> Result<CoreValueWithClassification, ()> {
        let inner = self.inner.try_clone_to_core_value()?;
        Ok(CoreValueWithClassification {
            inner,
            classification: self.classification.clone(),
        })
    }
}

impl<'a> From<&'a CoreValueWithClassification>
    for BorrowedCoreValueWithClassification<'a>
{
    fn from(value: &'a CoreValueWithClassification) -> Self {
        let inner = BorrowedCoreValue::from(&value.inner);
        let classification = value.classification.clone();
        BorrowedCoreValueWithClassification {
            inner,
            classification,
        }
    }
}

impl<'a> From<&'a mut CoreValueWithClassification>
    for BorrowedCoreValueWithClassification<'a>
{
    fn from(value: &'a mut CoreValueWithClassification) -> Self {
        let inner = BorrowedCoreValue::from(&value.inner);
        let classification = value.classification.clone();
        BorrowedCoreValueWithClassification {
            inner,
            classification,
        }
    }
}

/// Similar to [CoreValueWithClassification], but contains a [BorrowedCoreValueMut] instead of a [CoreValue].
/// It is used to represent a potentially borrowed reference to a [CoreValue] variant instead of owning it.
#[derive(Debug)]
pub struct BorrowedCoreValueWithClassificationMut<'a> {
    pub inner: BorrowedCoreValueMut<'a>,
    pub classification: ValueClassification,
}

impl<'a> BorrowedCoreValueWithClassificationMut<'a> {
    pub(crate) fn try_clone_to_core_value_with_classification(
        self,
    ) -> Result<CoreValueWithClassification, ()> {
        let inner = self.inner.try_clone_to_core_value()?;
        Ok(CoreValueWithClassification {
            inner,
            classification: self.classification.clone(),
        })
    }
}

impl<'a> From<&'a mut CoreValueWithClassification>
    for BorrowedCoreValueWithClassificationMut<'a>
{
    fn from(value: &'a mut CoreValueWithClassification) -> Self {
        let inner = BorrowedCoreValueMut::from(&mut value.inner);
        let classification = value.classification.clone();
        BorrowedCoreValueWithClassificationMut {
            inner,
            classification,
        }
    }
}
