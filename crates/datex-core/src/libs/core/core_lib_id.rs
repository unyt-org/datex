use crate::{
    libs::core::{type_id::CoreLibTypeId, value_id::CoreLibValueId},
    prelude::*,
    shared_values::{ExternalPointerAddress, PointerAddress},
};
use core::{fmt::Display, ops::Deref, str::FromStr};

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

impl CoreLibId {
    pub fn try_from_str(string: &str) -> Option<Self> {
        CoreLibTypeId::try_from_str(string)
            .map(CoreLibId::Type)
            .or_else(|| {
                CoreLibValueId::from_str(string).map(CoreLibId::Value).ok()
            })
    }
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

impl Display for CoreLibId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CoreLibId::Type(type_id) => write!(f, "{}", type_id),
            CoreLibId::Value(value_id) => write!(f, "{}", value_id),
        }
    }
}

impl From<CoreLibId> for CoreLibIdIndex {
    fn from(val: CoreLibId) -> Self {
        match val {
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
        let bytes: [u8; 2] =
            (core_lib_id).into().to_le_bytes().try_into().unwrap();
        ExternalPointerAddress::Builtin([bytes[0], bytes[1], 0])
    }
}

impl TryFrom<&ExternalPointerAddress> for CoreLibIdIndex {
    type Error = ();
    fn try_from(value: &ExternalPointerAddress) -> Result<Self, Self::Error> {
        if let ExternalPointerAddress::Builtin(bytes) = value {
            Ok(CoreLibIdIndex(u16::from_le_bytes(
                bytes[0..2].try_into().unwrap(),
            )))
        } else {
            Err(())
        }
    }
}

impl TryFrom<&PointerAddress> for CoreLibIdIndex {
    type Error = ();
    fn try_from(value: &PointerAddress) -> Result<Self, Self::Error> {
        if let PointerAddress::External(value) = value {
            CoreLibIdIndex::try_from(value)
        } else {
            Err(())
        }
    }
}

impl TryFrom<&ExternalPointerAddress> for CoreLibId {
    type Error = ();
    fn try_from(
        external: &ExternalPointerAddress,
    ) -> Result<Self, Self::Error> {
        CoreLibId::try_from(CoreLibIdIndex::try_from(external)?)
    }
}

impl TryFrom<&PointerAddress> for CoreLibId {
    type Error = ();

    fn try_from(pointer: &PointerAddress) -> Result<Self, Self::Error> {
        CoreLibId::try_from(CoreLibIdIndex::try_from(pointer)?)
    }
}
