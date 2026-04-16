use crate::{
    libs::core::{type_id::CoreLibBaseTypeId, value_id::CoreLibValueId},
    prelude::*,
};
pub const TYPE_SPACE_BASE: u16 = 0;
pub const TYPE_VARIANT_SPACE_BASE: u16 = 500;
pub const VALUE_SPACE_BASE: u16 = 1000;

pub trait CoreLibIdTrait {
    fn to_u16(&self) -> u16;
    fn try_from_u16(id: u16) -> Option<Self>
    where
        Self: Sized;
    fn name(&self) -> String;
    fn to_bytes(&self) -> [u8; 3] {
        (self.to_u16() as u32).to_le_bytes()[0..3]
            .try_into()
            .unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreLibId {
    Type(CoreLibBaseTypeId),
    Value(CoreLibValueId),
}
impl Into<CoreLibId> for CoreLibBaseTypeId {
    fn into(self) -> CoreLibId {
        CoreLibId::Type(self)
    }
}
impl Into<CoreLibId> for CoreLibValueId {
    fn into(self) -> CoreLibId {
        CoreLibId::Value(self)
    }
}

impl CoreLibIdTrait for CoreLibId {
    fn to_u16(&self) -> u16 {
        match self {
            CoreLibId::Type(type_id) => type_id.to_u16(),
            CoreLibId::Value(value_id) => value_id.to_u16(),
        }
    }

    fn try_from_u16(id: u16) -> Option<Self> {
        if let Some(type_id) = CoreLibBaseTypeId::try_from_u16(id) {
            Some(CoreLibId::Type(type_id))
        } else if let Some(value_id) = CoreLibValueId::try_from_u16(id) {
            Some(CoreLibId::Value(value_id))
        } else {
            None
        }
    }

    fn name(&self) -> String {
        match self {
            CoreLibId::Type(type_id) => type_id.name(),
            CoreLibId::Value(value_id) => value_id.name(),
        }
    }
}
