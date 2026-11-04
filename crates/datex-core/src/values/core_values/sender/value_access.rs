use crate::{
    traits::{get_datex_type::GetDatexType, value_access::ValueAccess},
    values::{
        core_values::native::DatexNativeBase,
    },
};
use crate::values::core_values::sender::Sender;

impl<T> ValueAccess for Sender<T>
where
    T: DatexNativeBase + GetDatexType,
{
}
