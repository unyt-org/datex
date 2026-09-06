use crate::{
    random::RandomState,
    traits::convert_core_value::ConvertCoreValue,
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
        core_value::CoreValue,
        core_values::native::DatexNativeBase,
        value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut},
    },
};
use core::hash::Hash;
use indexmap::IndexMap;

impl<K: DatexNativeBase + Eq + Hash + 'static, V: DatexNativeBase + 'static>
    ConvertCoreValue for IndexMap<K, V, RandomState>
{
    fn try_from_core_value(value: CoreValue) -> Result<Self, CoreValue> {
        match value {
            CoreValue::Native(native) => {
                native.try_into_value().map_err(CoreValue::Native)
            }
            _ => Err(value),
        }
    }

    fn try_borrow_from_core_value(value: &CoreValue) -> Result<&Self, ()> {
        match value {
            CoreValue::Native(native) => native.try_as().ok_or(()),
            _ => Err(()),
        }
    }

    fn try_borrow_mut_from_core_value(
        value: &mut CoreValue,
    ) -> Result<&mut Self, ()> {
        match value {
            CoreValue::Native(native) => native.try_as_mut().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a, K: DatexNativeBase + Eq + Hash + 'static, V: DatexNativeBase + 'static>
    TryFrom<BorrowedCoreValue<'a>> for Goat<'a, IndexMap<K, V, RandomState>>
{
    type Error = ();
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::Native(native) => native
                .filter_map(|v| {
                    v.as_any().downcast_ref::<IndexMap<K, V, RandomState>>()
                })
                .ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a, K: DatexNativeBase + Eq + Hash + 'static, V: DatexNativeBase + 'static>
    TryFrom<BorrowedCoreValueMut<'a>>
    for GoatMut<'a, IndexMap<K, V, RandomState>>
{
    type Error = ();
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::Native(native) => native
                .filter_map(|v| {
                    v.as_any_mut().downcast_mut::<IndexMap<K, V, RandomState>>()
                })
                .ok_or(()),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::core_value::CoreValue;

    #[test]
    #[cfg(feature = "std")]
    fn try_hash_map_from_core_value() {
        let mut core_value = CoreValue::native(IndexMap::<i32, i32>::default());
        let result = core_value.try_as::<IndexMap<i32, i32>>();
        assert!(result.is_some());

        let result_mut = core_value.try_as_mut::<IndexMap<i32, i32>>();
        assert!(result_mut.is_some());

        let result_into = core_value.try_into_value::<IndexMap<i32, i32>>();
        assert!(result_into.is_ok());
    }
}
