use crate::{
    traits::datex_hash::impl_datex_hash,
    values::core_values::{endpoint::Endpoint, instant::Instant},
};

impl_datex_hash!(Instant);
