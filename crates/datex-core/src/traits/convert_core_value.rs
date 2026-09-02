use crate::preludes::derive::CoreValue;

pub trait ConvertCoreValue {
    fn try_from_core_value(value: CoreValue) -> Result<Self, ()>
    where
        Self: Sized;
}

pub trait TryIntoFromCoreValue {
    fn try_from_core_value(value: CoreValue) -> Result<Self, ()>
    where
        Self: Sized;
}