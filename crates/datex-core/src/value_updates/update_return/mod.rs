use crate::values::value_container::ValueContainer;
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
