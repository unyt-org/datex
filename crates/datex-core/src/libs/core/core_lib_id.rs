use core::ops::Deref;

use crate::{
    libs::core::{type_id::CoreLibTypeId, value_id::CoreLibValueId},
    prelude::*,
    shared_values::pointer_address::{ExternalPointerAddress, PointerAddress},
};
pub const TYPE_SPACE_BASE: u16 = 0;
pub const TYPE_VARIANT_SPACE_BASE: u16 = 500;
pub const VALUE_SPACE_BASE: u16 = 1000;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CoreLibIdIndex(pub u16);

impl Deref for CoreLibIdIndex {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
pub trait CoreLibIdTrait:
    TryFrom<CoreLibIdIndex> + Into<CoreLibIdIndex>
{
    fn name(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreLibId {
    Type(CoreLibTypeId),
    Value(CoreLibValueId),
}
impl From<CoreLibTypeId> for CoreLibId {
    fn from(type_id: CoreLibTypeId) -> Self {
        CoreLibId::Type(type_id)
    }
}
impl From<CoreLibValueId> for CoreLibId {
    fn from(value_id: CoreLibValueId) -> Self {
        CoreLibId::Value(value_id)
    }
}

impl Into<CoreLibIdIndex> for CoreLibId {
    fn into(self) -> CoreLibIdIndex {
        match self {
            CoreLibId::Type(type_id) => type_id.into(),
            CoreLibId::Value(value_id) => value_id.into(),
        }
    }
}
impl TryFrom<CoreLibIdIndex> for CoreLibId {
    type Error = ();

    fn try_from(bytes: CoreLibIdIndex) -> Result<Self, Self::Error> {
        if let Ok(type_id) = CoreLibTypeId::try_from(bytes) {
            Ok(CoreLibId::Type(type_id))
        } else if let Ok(value_id) = CoreLibValueId::try_from(bytes) {
            Ok(CoreLibId::Value(value_id))
        } else {
            Err(())
        }
    }
}

impl CoreLibIdTrait for CoreLibId {
    fn name(&self) -> String {
        match self {
            CoreLibId::Type(type_id) => type_id.name(),
            CoreLibId::Value(value_id) => value_id.name(),
        }
    }
}

impl<T: CoreLibIdTrait> From<T> for PointerAddress {
    fn from(core_lib_id: T) -> Self {
        PointerAddress::External(ExternalPointerAddress::from(core_lib_id))
    }
}
impl<T: CoreLibIdTrait> From<T> for ExternalPointerAddress {
    fn from(core_lib_id: T) -> Self {
        ExternalPointerAddress::Builtin(
            core_lib_id.into().to_le_bytes()[0..3].try_into().unwrap(),
        )
    }
}
