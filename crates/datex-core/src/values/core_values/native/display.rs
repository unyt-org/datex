use crate::utils::impl_display_for_datex_value::impl_display_for_datex_value;
use crate::values::core_values::native::NativeCoreValue;

impl_display_for_datex_value!(
    NativeCoreValue,
    impl core::fmt::Display for NativeCoreValue {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "[[native value]]")
        }
    }
);