use crate::preludes::derive::CoreValue;

pub trait ConvertCoreValue {
    fn try_from_core_value(value: CoreValue) -> Result<Self, CoreValue>
    where
        Self: Sized;

    fn try_borrow_from_core_value(value: &CoreValue) -> Result<&Self, ()>;

    fn try_borrow_mut_from_core_value(
        value: &mut CoreValue,
    ) -> Result<&mut Self, ()>;
}
