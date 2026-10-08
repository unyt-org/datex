use crate::{
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
        core_value::CoreValue,
        core_values::integer::typed_integer::TypedInteger,
    },
};
use crate::traits::convert_value::ConvertValue;
use crate::values::core_value_with_classification::CoreValueWithClassification;
use crate::values::value::Value;
use crate::values::value::borrowed_value::{BorrowedValue, BorrowedValueMut};
use crate::values::value::borrowed_value::borrowed_core_value::{BorrowedCoreValue, BorrowedCoreValueMut};
use crate::values::value::borrowed_value::borrowed_core_value_with_classification::{BorrowedCoreValueWithClassification, BorrowedCoreValueWithClassificationMut};

macro_rules! impl_integer_core_value_conversions {
    ($($ty:ident => $variant:ident, $borrow:ident, $borrow_mut:ident;)* $(,)?) => {
        $(
            impl ConvertValue for $ty {
                fn to_value(self) -> Value {
                    CoreValue::TypedInteger(TypedInteger::$variant(self)).into()
                }
                fn try_from_value(value: Value) -> Result<Self, Value> {
                    match value {
                        Value::Core(CoreValueWithClassification {inner: CoreValue::TypedInteger(TypedInteger::$variant(v)), ..}) => Ok(v),
                        Value::Native(native) => native.try_into_value().map_err(Value::Native),
                        _ => Err(value),
                    }
                }

                fn try_borrow_from_value(value: &Value) -> Result<&Self, ()> {
                    match value {
                        Value::Core(CoreValueWithClassification {inner: CoreValue::TypedInteger(TypedInteger::$variant(v)), ..}) => Ok(v),
                        Value::Native(native) => native.try_as().ok_or(()),
                        _ => Err(()),
                    }
                }

                fn try_borrow_mut_from_value(value: &mut Value) -> Result<&mut Self, ()> {
                    match value {
                        Value::Core(CoreValueWithClassification {inner: CoreValue::TypedInteger(TypedInteger::$variant(v)), ..}) => Ok(v),
                        Value::Native(native) => native.try_as_mut().ok_or(()),
                        _ => Err(()),
                    }
                }
            }

            impl<'a> TryFrom<BorrowedValue<'a>> for Goat<'a, $ty> {
                type Error = ();
                fn try_from(value: BorrowedValue<'a>) -> Result<Self, Self::Error> {
                    match value {
                        BorrowedValue::Core(BorrowedCoreValueWithClassification {inner: BorrowedCoreValue::TypedInteger(v), ..}) => {
                            v.filter_map(|v| v.$borrow()).ok_or(())
                        }
                        BorrowedValue::Native(native) => native
                            .filter_map(|v| v.as_any().downcast_ref::<$ty>())
                            .ok_or(()),
                        _ => Err(()),
                    }
                }
            }

            impl<'a> TryFrom<BorrowedValueMut<'a>> for GoatMut<'a, $ty> {
                type Error = ();
                fn try_from(value: BorrowedValueMut<'a>) -> Result<Self, Self::Error> {
                    match value {
                        BorrowedValueMut::Core(BorrowedCoreValueWithClassificationMut {inner: BorrowedCoreValueMut::TypedInteger(v), ..}) => {
                            v.filter_map(|v| v.$borrow_mut()).ok_or(())
                        }
                        BorrowedValueMut::Native(native) => native
                            .filter_map(|v| v.as_any_mut().downcast_mut::<$ty>())
                            .ok_or(()),
                        _ => Err(()),
                    }
                }
            }
        )*
    };
}

impl_integer_core_value_conversions! {
    u8   => U8,   borrow_as_u8,   borrow_mut_as_u8;
    u16  => U16,  borrow_as_u16,  borrow_mut_as_u16;
    u32  => U32,  borrow_as_u32,  borrow_mut_as_u32;
    u64  => U64,  borrow_as_u64,  borrow_mut_as_u64;
    u128 => U128, borrow_as_u128, borrow_mut_as_u128;
    i8   => I8,   borrow_as_i8,   borrow_mut_as_i8;
    i16  => I16,  borrow_as_i16,  borrow_mut_as_i16;
    i32  => I32,  borrow_as_i32,  borrow_mut_as_i32;
    i64  => I64,  borrow_as_i64,  borrow_mut_as_i64;
    i128 => I128, borrow_as_i128, borrow_mut_as_i128;
}

#[cfg(test)]
mod tests {
    use crate::{
        utils::{goat::Goat, goat_mut::GoatMut},
        values::{
            core_value::CoreValue,
            core_values::integer::typed_integer::TypedInteger,
        },
    };
    use crate::traits::convert_value::ConvertValue;
    use crate::values::value::borrowed_value::{BorrowedValue, BorrowedValueMut};

    #[test]
    fn try_integer_from_core_value() {
        let core_value = CoreValue::TypedInteger(TypedInteger::U32(42)).to_value();

        let result = core_value.try_as::<u32>();
        assert_eq!(*result.unwrap(), 42);

        let result = core_value.try_into_value::<u32>();
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn try_borrow_integer_from_core_value() {
        let core_value = CoreValue::TypedInteger(TypedInteger::U32(42)).to_value();
        let result = core_value.try_as::<u32>();
        assert_eq!(*result.unwrap(), 42);
    }

    #[test]
    fn try_borrow_mut_integer_from_core_value() {
        let mut value = CoreValue::TypedInteger(TypedInteger::U32(42)).to_value();
        let result = value.try_as_mut::<u32>();
        *result.unwrap() = 100;
        assert_eq!(value.try_into_core_value(), Ok(CoreValue::TypedInteger(TypedInteger::U32(100))));
    }

    #[test]
    fn try_owned_integer_from_core_value() {
        let core_value = CoreValue::TypedInteger(TypedInteger::U64(123)).to_value();
        let result = core_value.try_into_value::<u64>();
        assert_eq!(result.unwrap(), 123);
    }

    #[test]
    fn try_invalid_type() {
        let mut core_value = CoreValue::TypedInteger(TypedInteger::U32(42)).to_value();
        assert!(core_value.try_as::<u64>().is_none());
        assert!(core_value.try_as::<i32>().is_none());
        assert!(core_value.try_as_mut::<u64>().is_none());
        assert!(core_value.try_as_mut::<i32>().is_none());

        assert!(core_value.try_into_value::<u64>().is_err());

        let core_value = CoreValue::TypedInteger(TypedInteger::U32(42)).to_value();
        assert!(core_value.try_into_value::<i32>().is_err());
    }

    #[test]
    fn try_integer_from_native_core_value() {
        let core_value = 42u32.to_value();
        let result = core_value.try_as::<u32>();
        assert_eq!(*result.unwrap(), 42);
    }

    #[test]
    fn try_borrow_mut_integer_from_native_core_value() {
        let mut core_value = 42u32.to_value();
        let result = core_value.try_as_mut::<u32>();
        *result.unwrap() = 99;
        assert_eq!(*core_value.try_as::<u32>().unwrap(), 99);
    }

    #[test]
    fn try_owned_integer_from_native_core_value() {
        let core_value = 42u32.to_value();
        let result = core_value.try_into_value::<u32>();
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn try_borrowed_core_value_integer() {
        let core_value = CoreValue::TypedInteger(TypedInteger::I32(42)).to_value();
        let borrowed = BorrowedValue::from(&core_value);
        let result = Goat::<i32>::try_from(borrowed);
        assert_eq!(*result.unwrap(), 42);
    }

    #[test]
    fn try_borrowed_core_value_mut_integer() {
        let mut core_value = CoreValue::TypedInteger(TypedInteger::I32(42)).to_value();
        let borrowed = BorrowedValueMut::from(&mut core_value);
        let mut result = GoatMut::<i32>::try_from(borrowed).unwrap();
        *result = 123;
        drop(result);
        assert_eq!(core_value, CoreValue::TypedInteger(TypedInteger::I32(123)).to_value());
    }
}
