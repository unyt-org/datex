use crate::{
    traits::{get_datex_type::GetDatexType, value_access::ValueAccess},
    values::{
        core_values::native::DatexNativeBase,
    },
};
use crate::values::core_values::receiver::Receiver;

impl<T> ValueAccess for Receiver<T>
where
    T: DatexNativeBase + GetDatexType,
{
}
