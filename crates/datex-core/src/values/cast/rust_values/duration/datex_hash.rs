use crate::traits::datex_hash::impl_datex_hash;
use core::time::Duration;
use crate::dif::deserialize_serde_context::impl_serde_with_context;

impl_datex_hash!(Duration);
impl_serde_with_context!(Duration);