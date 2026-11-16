use crate::{prelude::*, values::value_container::ValueContainer};
pub mod serde_dif;
#[derive(Clone, Debug, PartialEq)]
pub enum UpdateReturn {
    None,
    SingleValue(ValueContainer),
    MultipleValues(Vec<ValueContainer>),
}

impl From<()> for UpdateReturn {
    fn from(_: ()) -> Self {
        UpdateReturn::None
    }
}

impl From<ValueContainer> for UpdateReturn {
    fn from(value: ValueContainer) -> Self {
        UpdateReturn::SingleValue(value)
    }
}

impl From<Vec<ValueContainer>> for UpdateReturn {
    fn from(items: Vec<ValueContainer>) -> Self {
        UpdateReturn::MultipleValues(items)
    }
}

impl From<Option<ValueContainer>> for UpdateReturn {
    fn from(value: Option<ValueContainer>) -> Self {
        match value {
            Some(value) => UpdateReturn::SingleValue(value),
            None => UpdateReturn::None,
        }
    }
}
impl TryFrom<UpdateReturn> for Option<ValueContainer> {
    type Error = ();

    fn try_from(value: UpdateReturn) -> Result<Self, Self::Error> {
        match value {
            UpdateReturn::None => Ok(None),
            UpdateReturn::SingleValue(value) => Ok(Some(value)),
            UpdateReturn::MultipleValues(_) => Err(()),
        }
    }
}
impl TryFrom<UpdateReturn> for Vec<ValueContainer> {
    type Error = ();

    fn try_from(value: UpdateReturn) -> Result<Self, Self::Error> {
        match value {
            UpdateReturn::MultipleValues(values) => Ok(values),
            _ => Err(()),
        }
    }
}
impl TryFrom<UpdateReturn> for ValueContainer {
    type Error = ();

    fn try_from(value: UpdateReturn) -> Result<Self, Self::Error> {
        match value {
            UpdateReturn::SingleValue(value) => Ok(value),
            _ => Err(()),
        }
    }
}
impl TryFrom<UpdateReturn> for () {
    type Error = ();

    fn try_from(value: UpdateReturn) -> Result<Self, Self::Error> {
        match value {
            UpdateReturn::None => Ok(()),
            _ => Err(()),
        }
    }
}
