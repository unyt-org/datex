use crate::values::{core_value::CoreValue, core_values::native::DatexNative};

/// This trait is completely auto derived for all types and must
/// not be implemented manually.
/// It is used to attempt to clone a value into a new [CoreValue].
pub trait TryClone {
    /// Attempts to clone [Self] into a new [CoreValue].
    fn try_clone(&self) -> Result<CoreValue, ()>;
}

/// Per default, types that do not implement Clone will return an error when try_clone is called.
impl<T> TryClone for T {
    default fn try_clone(&self) -> Result<CoreValue, ()> {
        Err(())
    }
}

/// For any [DatexNative] type that implements [Clone],
/// try_clone gets this default impl that returns a new [CoreValue] containing the cloned value.
default impl<T> TryClone for T
where
    T: Clone + DatexNative,
{
    fn try_clone(&self) -> Result<CoreValue, ()> {
        Ok(CoreValue::native(self.clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*, traits::try_clone::TryClone, values::core_value::CoreValue,
    };
    use crate::traits::convert_core_value::ConvertCoreValue;

    #[test]
    fn test_try_string() {
        let value = "Hello, world!".to_string();
        let cloned_value = value.try_clone().unwrap();
        assert_eq!(
            cloned_value,
            "Hello, world!".to_string().to_core_value()
        );
    }

    #[test]
    fn test_try_unclonable_type() {
        struct UnclonableType;
        let value = UnclonableType;
        let cloned_value = value.try_clone();
        assert!(cloned_value.is_err());
    }
}
