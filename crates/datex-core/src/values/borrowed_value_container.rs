use crate::shared_values::SharedContainer;
use crate::utils::goat::Goat;
use crate::utils::goat_mut::GoatMut;
use crate::values::core_value::CoreValue;
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut, BorrowedValue, BorrowedValueMut};
use crate::values::value_container::ValueContainer;

pub enum BorrowedValueContainer<'a> {
    Local(BorrowedValue<'a>),
    Shared(&'a SharedContainer),
}

impl<'a> BorrowedValueContainer<'a> {
    /// Tries to get an immutable reference to the value as a specified type.
    /// Does not perform any type conversion.
    /// This only works for local values, not for shared values.
    pub fn try_as<T>(self) -> Option<Goat<'a, T>>
    where
        Goat<'a, T>: TryFrom<BorrowedCoreValue<'a>>,
    {
        match self {
            BorrowedValueContainer::Local(value) => value.inner.try_as(),
            BorrowedValueContainer::Shared(_) => None,
        }
    }

    pub fn try_clone_to_value_container(self) -> Result<ValueContainer, ()>
    where
        CoreValue: Clone,
    {
        match self {
            BorrowedValueContainer::Local(value) => Ok(ValueContainer::Local(value.try_clone_to_value()?)),
            BorrowedValueContainer::Shared(shared) => Ok(ValueContainer::Shared(shared.clone())),
        }
    }
}


pub enum BorrowedValueContainerMut<'a> {
    Local(BorrowedValueMut<'a>),
    Shared(&'a mut SharedContainer),
}

impl<'a> BorrowedValueContainerMut<'a> {

    /// Tries to get an immutable reference to the value as a specified type.
    /// Does not perform any type conversion.
    /// This only works for local values, not for shared values.
    pub fn try_as<T>(self) -> Option<Goat<'a, T>>
    where
        Goat<'a, T>: TryFrom<BorrowedCoreValueMut<'a>>,
    {
        match self {
            BorrowedValueContainerMut::Local(value) => value.inner.try_as(),
            BorrowedValueContainerMut::Shared(_) => None,
        }
    }

    /// Tries to get a mutable reference to the value as a specified type.
    /// Does not perform any type conversion.
    /// This only works for local values, not for shared values.
    pub fn try_as_mut<T>(self) -> Option<GoatMut<'a, T>>
    where
        GoatMut<'a, T>: TryFrom<BorrowedCoreValueMut<'a>>,
    {
        match self {
            BorrowedValueContainerMut::Local(value) => value.inner.try_as_mut(),
            BorrowedValueContainerMut::Shared(_) => None,
        }
    }
}

impl<'a> From<BorrowedValue<'a>> for BorrowedValueContainer<'a> {
    fn from(borrowed_value: BorrowedValue<'a>) -> Self {
        BorrowedValueContainer::Local(borrowed_value)
    }
}

impl<'a> From<&'a SharedContainer> for BorrowedValueContainer<'a> {
    fn from(shared_container: &'a SharedContainer) -> Self {
        BorrowedValueContainer::Shared(shared_container)
    }
}

impl<'a> From<&'a ValueContainer> for BorrowedValueContainer<'a> {
    fn from(value_container: &'a ValueContainer) -> Self {
        match value_container {
            ValueContainer::Shared(shared_container) => {
                BorrowedValueContainer::Shared(shared_container)
            }
            ValueContainer::Local(local_value) => {
                BorrowedValueContainer::Local(local_value.into())
            }
        }
    }
}

impl<'a> From<BorrowedValueMut<'a>> for BorrowedValueContainerMut<'a> {
    fn from(borrowed_value: BorrowedValueMut<'a>) -> Self {
        BorrowedValueContainerMut::Local(borrowed_value)
    }
}

impl<'a> From<&'a mut SharedContainer> for BorrowedValueContainerMut<'a> {
    fn from(shared_container: &'a mut SharedContainer) -> Self {
        BorrowedValueContainerMut::Shared(shared_container)
    }
}

impl<'a> From<&'a mut ValueContainer> for BorrowedValueContainerMut<'a> {
    fn from(value_container: &'a mut ValueContainer) -> Self {
        match value_container {
            ValueContainer::Shared(shared_container) => {
                BorrowedValueContainerMut::Shared(shared_container)
            }
            ValueContainer::Local(local_value) => {
                BorrowedValueContainerMut::Local(local_value.into())
            }
        }
    }
}
