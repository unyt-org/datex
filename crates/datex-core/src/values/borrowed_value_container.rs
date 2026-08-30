use crate::{
    shared_values::{
        OwnedSharedContainer, ReferencedSharedContainer, SharedContainer,
    },
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
        core_value::CoreValue,
        core_values::native::DatexNative,
        value::{
            Value,
            borrowed_value::{
                BorrowedCoreValue, BorrowedCoreValueMut, BorrowedValue,
                BorrowedValueMut,
            },
        },
        value_container::ValueContainer,
    },
};
use core::{
    cell::{Ref, RefMut},
    fmt::Debug,
};
use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;

pub trait AsBorrowed<'a> {
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a>;
}

#[derive(Debug)]
pub enum BorrowedValueContainer<'a> {
    Local(BorrowedValue<'a>),
    Shared(SharedContainer),
}

impl<'a> BorrowedValueContainer<'a> {
    /// Creates a new `BorrowedValueContainer` from a reference to a native value.
    pub fn native_borrowed_only_structural<T: DatexNativeOnlyStructural>(val: &'a T) -> Self {
        BorrowedValueContainer::Local(BorrowedValue {
            inner: BorrowedCoreValue::Native(Goat::Borrowed(val)),
            custom_type: None,
        })
    }

    /// Creates a new `BorrowedValueContainer` from a reference to a native value wrapped in a `Ref`.
    pub fn native_ref<T: DatexNativeOnlyStructural>(val: Ref<'a, T>) -> Self {
        BorrowedValueContainer::Local(BorrowedValue {
            inner: BorrowedCoreValue::Native(Goat::Ref(val)),
            custom_type: None,
        })
    }

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
            BorrowedValueContainer::Local(value) => {
                Ok(ValueContainer::Local(value.try_clone_to_value()?))
            }
            BorrowedValueContainer::Shared(shared) => {
                Ok(ValueContainer::Shared(shared.clone()))
            }
        }
    }
}

pub trait AsBorrowedMut<'a> {
    fn as_borrowed_mut(&'a mut self) -> BorrowedValueContainerMut<'a>;
}

pub enum BorrowedValueContainerMut<'a> {
    Local(BorrowedValueMut<'a>),
    Shared(SharedContainer),
}

impl<'a> BorrowedValueContainerMut<'a> {
    /// Creates a new `BorrowedValueContainerMut` from a mutable reference to a native value.
    pub fn native_borrowed_only_structural<T: DatexNativeOnlyStructural>(val: &'a mut T) -> Self {
        BorrowedValueContainerMut::Local(BorrowedValueMut {
            inner: BorrowedCoreValueMut::Native(GoatMut::Borrowed(val)),
            custom_type: None,
        })
    }

    /// Creates a new `BorrowedValueContainerMut` from a mutable reference to a native value wrapped in a `RefMut`.
    pub fn native_ref_only_structural<T: DatexNativeOnlyStructural>(val: RefMut<'a, T>) -> Self {
        BorrowedValueContainerMut::Local(BorrowedValueMut {
            inner: BorrowedCoreValueMut::Native(GoatMut::Ref(val)),
            custom_type: None,
        })
    }

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

impl<'a> AsBorrowed<'a> for SharedContainer {
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::Shared(self.clone())
    }
}
impl<'a> AsBorrowed<'a> for OwnedSharedContainer {
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::Shared(SharedContainer::Referenced(
            self.derive_with_max_mutability(),
        ))
    }
}

impl<'a> AsBorrowed<'a> for ReferencedSharedContainer {
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::Shared(SharedContainer::Referenced(
            self.clone(),
        ))
    }
}

impl<'a> AsBorrowed<'a> for Value {
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::Local(self.into())
    }
}

impl<'a> AsBorrowed<'a> for ValueContainer {
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        match self {
            ValueContainer::Shared(shared_container) => {
                shared_container.as_borrowed()
            }
            ValueContainer::Local(local_value) => local_value.as_borrowed(),
        }
    }
}

impl<'a> From<BorrowedValueMut<'a>> for BorrowedValueContainerMut<'a> {
    fn from(borrowed_value: BorrowedValueMut<'a>) -> Self {
        BorrowedValueContainerMut::Local(borrowed_value)
    }
}

impl<'a> AsBorrowedMut<'a> for SharedContainer {
    fn as_borrowed_mut(&'a mut self) -> BorrowedValueContainerMut<'a> {
        BorrowedValueContainerMut::Shared(self.clone())
    }
}

impl<'a> AsBorrowedMut<'a> for OwnedSharedContainer {
    fn as_borrowed_mut(&'a mut self) -> BorrowedValueContainerMut<'a> {
        BorrowedValueContainerMut::Shared(SharedContainer::Referenced(
            self.derive_with_max_mutability(),
        ))
    }
}

impl<'a> AsBorrowedMut<'a> for ReferencedSharedContainer {
    fn as_borrowed_mut(&'a mut self) -> BorrowedValueContainerMut<'a> {
        BorrowedValueContainerMut::Shared(SharedContainer::Referenced(
            self.clone(),
        ))
    }
}

impl<'a> AsBorrowedMut<'a> for Value {
    fn as_borrowed_mut(&'a mut self) -> BorrowedValueContainerMut<'a> {
        BorrowedValueContainerMut::Local(self.into())
    }
}

impl<'a> AsBorrowedMut<'a> for ValueContainer {
    fn as_borrowed_mut(&'a mut self) -> BorrowedValueContainerMut<'a> {
        match self {
            ValueContainer::Shared(shared_container) => {
                shared_container.as_borrowed_mut()
            }
            ValueContainer::Local(local_value) => local_value.as_borrowed_mut(),
        }
    }
}
