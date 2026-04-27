use crate::{
    libs::core::core_lib_id::{
        CoreLibIdIndex, CoreLibIdTrait, VALUE_SPACE_BASE,
    },
    prelude::*,
};
use num_enum::TryFromPrimitive;
use strum::EnumIter;
use strum_macros::{Display, EnumString};

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    EnumString,
    Display,
    TryFromPrimitive,
)]
#[strum(serialize_all = "snake_case")]
#[repr(u16)]
pub enum CoreLibValueId {
    Core,  // #core
    Print, // #core.print (function, might be removed later)
}

impl From<CoreLibValueId> for CoreLibIdIndex {
    fn from(value_id: CoreLibValueId) -> Self {
        CoreLibIdIndex((value_id as u16) + VALUE_SPACE_BASE)
    }
}

impl CoreLibIdTrait for CoreLibValueId {
    fn name(&self) -> String {
        Self::to_string(self)
    }
}
impl TryFrom<CoreLibIdIndex> for CoreLibValueId {
    type Error = ();

    fn try_from(id: CoreLibIdIndex) -> Result<Self, Self::Error> {
        let id = id.0.checked_sub(VALUE_SPACE_BASE).ok_or(())?;
        CoreLibValueId::try_from(id).map_err(|_| ())
    }
}
