use crate::{
    traits::{get_datex_type::GetDatexType, value_access::ValueAccess},
    values::core_values::{native::DatexNativeBase, sender::Sender},
};

impl<T> ValueAccess for Sender<T> where T: DatexNativeBase + GetDatexType {}
