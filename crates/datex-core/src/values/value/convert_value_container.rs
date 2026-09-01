use crate::preludes::derive::{BorrowedValueContainer, SharedReferencesCache, Value, ValueContainer};
use crate::traits::convert_value_container::ConvertValueContainer;
use crate::values::value::borrowed_value::BorrowedValue;

impl ConvertValueContainer for Value {
    fn to_value_container(
        self,
        cache: &mut SharedReferencesCache,
    ) -> ValueContainer {
        ValueContainer::Local(self)
    }

    fn as_borrowed_value_container(&self, _cache: &mut SharedReferencesCache) -> BorrowedValueContainer {
        BorrowedValueContainer::Local(BorrowedValue::from(self))
    }

    fn try_from_value_container(value_container: ValueContainer) -> Result<Self, ()>
    where
        Self: Sized
    {
        match value_container {
            ValueContainer::Local(value) => Ok(value),
            _ => Err(()),
        }
    }
}
