use crate::{
    preludes::derive::SharedReferencesCache,
    shared_values::{
        OwnedSharedContainer, ReferencedSharedContainer, SharedContainer,
    },
    traits::datex_native_structural::DatexNativeStructural,
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
        core_value::CoreValue,
        core_values::{map::BorrowedMapKey, native::DatexNative},
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
use core::fmt::Debug;

pub trait AsBorrowed<'a> {
    fn as_borrowed(
        &'a self,
        cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainer<'a>;
}

default impl<'a, T> AsBorrowed<'a> for T
where
    T: DatexNative,
{
    fn as_borrowed(
        &'a self,
        cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::native_borrowed(self, cache)
    }
}

#[derive(Debug)]
pub enum BorrowedValueContainer<'a> {
    Local(BorrowedValue<'a>),
    Shared(SharedContainer),
}

impl<'a> BorrowedValueContainer<'a> {
    /// Creates a new `BorrowedValueContainer` from a reference to a native value.
    pub fn native_borrowed<T: DatexNative>(
        val: impl Into<Goat<'a, T>>,
        cache: &mut SharedReferencesCache,
    ) -> Self {
        BorrowedValueContainer::Local(BorrowedValue::native_borrowed(
            val, cache,
        ))
    }

    /// Creates a new `BorrowedValueContainer` from a reference to a native value.
    pub fn native_borrowed_structural<T: DatexNativeStructural>(
        val: impl Into<Goat<'a, T>>,
    ) -> Self {
        BorrowedValueContainer::Local(
            BorrowedValue::native_borrowed_structural(val),
        )
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
    fn as_borrowed_mut(
        &'a mut self,
        cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainerMut<'a>;
}

default impl<'a, T> AsBorrowedMut<'a> for T
where
    T: DatexNative,
{
    fn as_borrowed_mut(
        &'a mut self,
        cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainerMut<'a> {
        BorrowedValueContainerMut::native_borrowed(self, cache)
    }
}

pub enum BorrowedValueContainerMut<'a> {
    Local(BorrowedValueMut<'a>),
    Shared(SharedContainer),
}

impl<'a> BorrowedValueContainerMut<'a> {
    /// Creates a new `BorrowedValueContainer` from a reference to a native value.
    pub fn native_borrowed<T: DatexNative>(
        val: impl Into<GoatMut<'a, T>>,
        cache: &mut SharedReferencesCache,
    ) -> Self {
        BorrowedValueContainerMut::Local(BorrowedValueMut::native_borrowed(
            val, cache,
        ))
    }

    /// Creates a new `BorrowedValueContainer` from a reference to a native value.
    pub fn native_borrowed_structural<T: DatexNativeStructural>(
        val: impl Into<GoatMut<'a, T>>,
    ) -> Self {
        BorrowedValueContainerMut::Local(
            BorrowedValueMut::native_borrowed_structural(val),
        )
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
    fn as_borrowed(
        &'a self,
        _cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::Shared(self.clone())
    }
}
impl<'a> AsBorrowed<'a> for OwnedSharedContainer {
    fn as_borrowed(
        &'a self,
        _cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::Shared(SharedContainer::Referenced(
            self.derive_with_max_mutability(),
        ))
    }
}

impl<'a> AsBorrowed<'a> for ReferencedSharedContainer {
    fn as_borrowed(
        &'a self,
        _cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::Shared(SharedContainer::Referenced(
            self.clone(),
        ))
    }
}

impl<'a> AsBorrowed<'a> for Value {
    fn as_borrowed(
        &'a self,
        _cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::Local(self.into())
    }
}

impl<'a> From<&'a ValueContainer> for BorrowedValueContainer<'a> {
    fn from(value_container: &'a ValueContainer) -> Self {
        match value_container {
            ValueContainer::Shared(shared_container) => {
                BorrowedValueContainer::Shared(shared_container.clone())
            }
            ValueContainer::Local(local_value) => {
                BorrowedValueContainer::Local(BorrowedValue::from(local_value))
            }
        }
    }
}

impl<'a> From<&'a mut ValueContainer> for BorrowedValueContainerMut<'a> {
    fn from(value_container: &'a mut ValueContainer) -> Self {
        match value_container {
            ValueContainer::Shared(shared_container) => {
                BorrowedValueContainerMut::Shared(shared_container.clone())
            }
            ValueContainer::Local(local_value) => {
                BorrowedValueContainerMut::Local(BorrowedValueMut::from(
                    local_value,
                ))
            }
        }
    }
}

impl<'a> From<BorrowedMapKey<'a>> for BorrowedValueContainer<'a> {
    fn from(borrowed_map_key: BorrowedMapKey<'a>) -> Self {
        match borrowed_map_key {
            BorrowedMapKey::Text(_text) => todo!(), // BorrowedValueContainer::Local(BorrowedValue::native_borrowed_structural(text)),
            BorrowedMapKey::Value(value) => BorrowedValueContainer::from(value),
        }
    }
}

impl<'a> From<BorrowedValueMut<'a>> for BorrowedValueContainerMut<'a> {
    fn from(borrowed_value: BorrowedValueMut<'a>) -> Self {
        BorrowedValueContainerMut::Local(borrowed_value)
    }
}

impl<'a> AsBorrowedMut<'a> for SharedContainer {
    fn as_borrowed_mut(
        &'a mut self,
        _cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainerMut<'a> {
        BorrowedValueContainerMut::Shared(self.clone())
    }
}

impl<'a> AsBorrowedMut<'a> for OwnedSharedContainer {
    fn as_borrowed_mut(
        &'a mut self,
        _cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainerMut<'a> {
        BorrowedValueContainerMut::Shared(SharedContainer::Referenced(
            self.derive_with_max_mutability(),
        ))
    }
}

impl<'a> AsBorrowedMut<'a> for ReferencedSharedContainer {
    fn as_borrowed_mut(
        &'a mut self,
        _cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainerMut<'a> {
        BorrowedValueContainerMut::Shared(SharedContainer::Referenced(
            self.clone(),
        ))
    }
}

impl<'a> AsBorrowedMut<'a> for Value {
    fn as_borrowed_mut(
        &'a mut self,
        _cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainerMut<'a> {
        BorrowedValueContainerMut::Local(self.into())
    }
}

impl<'a> AsBorrowedMut<'a> for ValueContainer {
    fn as_borrowed_mut(
        &'a mut self,
        cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainerMut<'a> {
        match self {
            ValueContainer::Shared(shared_container) => {
                shared_container.as_borrowed_mut(cache)
            }
            ValueContainer::Local(local_value) => {
                local_value.as_borrowed_mut(cache)
            }
        }
    }
}
