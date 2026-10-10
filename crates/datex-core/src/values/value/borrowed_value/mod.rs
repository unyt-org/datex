pub mod borrowed_core_value;
pub mod borrowed_core_value_with_classification;

use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::{
        classification::Classification,
        datex_native_structural::DatexNativeStructural,
    },
    types::{
        entities::entity_type_definition::EntityTypeDefinition, r#type::Type,
    },
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
        core_value::CoreValue,
        core_values::{
            boolean::Boolean,
            callable::Callable,
            decimal::{Decimal, typed_decimal::TypedDecimal},
            endpoint::Endpoint,
            integer::{Integer, typed_integer::TypedInteger},
            list::List,
            map::Map,
            native::DatexNative,
            range::Range,
            text::Text,
        },
        value::{
            Value,
            borrowed_value::{
                borrowed_core_value::{
                    BorrowedCoreValue, BorrowedCoreValueMut,
                },
                borrowed_core_value_with_classification::{
                    BorrowedCoreValueWithClassification,
                    BorrowedCoreValueWithClassificationMut,
                },
            },
            value_classification::ValueClassification,
        },
        value_container::ValueContainer,
    },
};
use core::{
    cell::{Ref, RefMut},
    fmt::Debug,
    ops::{Deref, DerefMut},
};

/// Similar to [Value], but contains a [BorrowedCoreValue] instead of a [CoreValue].
/// It is used to represent a potentially borrowed reference to a [CoreValue] variant instead of owning it.
#[derive(Debug)]
pub enum BorrowedValue<'a> {
    Core(BorrowedCoreValueWithClassification<'a>),
    Native(Goat<'a, dyn DatexNative>),
}

impl<'a> BorrowedValue<'a> {
    pub fn core(core: impl Into<BorrowedCoreValue<'a>>) -> Self {
        BorrowedValue::Core(BorrowedCoreValueWithClassification {
            inner: core.into(),
            classification: ValueClassification::default(),
        })
    }
}

/// Converts a [Goat] of a native value into a [Goat] of a dynamic [DatexNative] trait object.
pub fn into_dyn_goat<'a, T: DatexNative>(
    val: Goat<'a, T>,
) -> Goat<'a, dyn DatexNative> {
    match val {
        Goat::Ref(value) => {
            Goat::Ref(Ref::map(value, |value| value as &dyn DatexNative))
        }
        Goat::Borrowed(value) => Goat::Borrowed(value),
    }
}

/// Converts a [GoatMut] of a native value into a [GoatMut] of a dynamic [DatexNative] trait object.
pub fn into_dyn_goat_mut<'a, T: DatexNative>(
    val: GoatMut<'a, T>,
) -> GoatMut<'a, dyn DatexNative> {
    match val {
        GoatMut::Ref(value) => GoatMut::Ref(RefMut::map(value, |value| {
            value as &mut dyn DatexNative
        })),
        GoatMut::Borrowed(value) => GoatMut::Borrowed(value),
    }
}

impl<'a> BorrowedValue<'a> {
    pub fn new<T: DatexNative>(val: impl Into<Goat<'a, T>>) -> Self {
        let val = val.into();
        let val = into_dyn_goat(val);
        BorrowedValue::Native(val)
    }

    /// Creates a new [BorrowedValue] from a reference to a native value.
    pub fn native_borrowed<T: DatexNative>(
        val: impl Into<Goat<'a, T>>,
        cache: &mut SharedReferencesCache,
    ) -> Self {
        let val = val.into();
        let val = into_dyn_goat(val);
        BorrowedValue::Native(val)
    }

    /// Creates a new [BorrowedValue] from a reference to a native value.
    pub fn native_borrowed_structural<T: DatexNativeStructural>(
        val: impl Into<Goat<'a, T>>,
    ) -> Self {
        let val = val.into();
        let val = into_dyn_goat(val);
        BorrowedValue::Native(val)
    }

    pub fn try_clone_to_value(self) -> Result<Value, ()>
    where
        CoreValue: Clone,
    {
        match self {
            BorrowedValue::Core(borrowed_core_value) => borrowed_core_value
                .try_clone_to_core_value_with_classification()
                .map(Value::Core),
            BorrowedValue::Native(native) => native.deref().try_clone(),
        }
    }

    /// Tries to get a borrow of the current value as the specified type.
    /// Does not perform any type conversion.
    pub fn try_as<T: ?Sized>(self) -> Option<Goat<'a, T>>
    where
        Goat<'a, T>: TryFrom<BorrowedValue<'a>>,
    {
        Goat::try_from(self).ok()
    }
}

impl<'a> From<&'a Value> for BorrowedValue<'a> {
    fn from(value: &'a Value) -> Self {
        match value {
            Value::Core(core_value) => BorrowedValue::Core(
                BorrowedCoreValueWithClassification::from(core_value),
            ),
            Value::Native(native_value) => BorrowedValue::Native(
                Goat::Borrowed(native_value.value.deref()),
            ),
        }
    }
}

/// Similar to [Value], but contains a [BorrowedCoreValueMut] instead of a [CoreValue].
/// It is used to represent a potentially borrowed mutable reference to a [CoreValue] variant instead of owning it.
pub enum BorrowedValueMut<'a> {
    Core(BorrowedCoreValueWithClassificationMut<'a>),
    Native(GoatMut<'a, dyn DatexNative>),
}

impl<'a> BorrowedValueMut<'a> {
    pub fn core(core: impl Into<BorrowedCoreValueMut<'a>>) -> Self {
        BorrowedValueMut::Core(BorrowedCoreValueWithClassificationMut {
            inner: core.into(),
            classification: ValueClassification::default(),
        })
    }

    /// Creates a new [BorrowedValueMut] from a reference to a native value.
    pub fn native_borrowed<T: DatexNative>(
        val: impl Into<GoatMut<'a, T>>,
        cache: &mut SharedReferencesCache,
    ) -> Self {
        let val = val.into();
        let val = into_dyn_goat_mut(val);
        BorrowedValueMut::Native(val)
    }

    /// Creates a new [BorrowedValueMut] from a reference to a native value.
    pub fn native_borrowed_structural<T: DatexNativeStructural>(
        val: impl Into<GoatMut<'a, T>>,
    ) -> Self {
        let val = val.into();
        let val = into_dyn_goat_mut(val);
        BorrowedValueMut::Native(val)
    }

    /// Tries to get a borrow of the current value as the specified type.
    /// Does not perform any type conversion.
    pub fn try_as<T: ?Sized>(self) -> Option<Goat<'a, T>>
    where
        Goat<'a, T>: TryFrom<BorrowedValueMut<'a>>,
    {
        Goat::try_from(self).ok()
    }

    /// Tries to get a mutable borrow of the current value as the specified type.
    /// Does not perform any type conversion.
    pub fn try_as_mut<T>(self) -> Option<GoatMut<'a, T>>
    where
        GoatMut<'a, T>: TryFrom<BorrowedValueMut<'a>>,
    {
        GoatMut::try_from(self).ok()
    }
}

impl<'a> From<&'a mut Value> for BorrowedValueMut<'a> {
    fn from(value: &'a mut Value) -> Self {
        match value {
            Value::Core(core_value) => BorrowedValueMut::Core(
                BorrowedCoreValueWithClassificationMut::from(core_value),
            ),
            Value::Native(native_value) => BorrowedValueMut::Native(
                GoatMut::Borrowed(native_value.value.deref_mut()),
            ),
        }
    }
}
