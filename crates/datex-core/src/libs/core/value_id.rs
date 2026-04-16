use crate::{
    libs::core::core_lib_id::{CoreLibIdTrait, VALUE_SPACE_BASE},
    prelude::*,
    shared_values::pointer_address::{ExternalPointerAddress, PointerAddress},
};
use datex_macros_internal::CoreLibString;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, CoreLibString)]
pub enum CoreLibValueId {
    Core,  // #core
    Print, // #core.print (function, might be removed later)
}

impl CoreLibIdTrait for CoreLibValueId {
    fn to_u16(&self) -> u16 {
        match VALUE_SPACE_BASE + self {
            CoreLibValueId::Core => 0,
            CoreLibValueId::Print => 1,
        }
    }

    fn try_from_u16(id: u16) -> Option<Self> {
        match id - VALUE_SPACE_BASE {
            0 => Some(CoreLibValueId::Core),
            1 => Some(CoreLibValueId::Print),
            _ => None,
        }
    }
    fn name(&self) -> String {
        Self::to_string(self)
    }
}

impl From<CoreLibValueId> for PointerAddress {
    fn from(id: CoreLibValueId) -> Self {
        PointerAddress::External(ExternalPointerAddress::from(&id))
    }
}

impl From<&CoreLibValueId> for ExternalPointerAddress {
    fn from(id: &CoreLibValueId) -> Self {
        ExternalPointerAddress::Builtin(id.to_bytes())
    }
}

impl TryFrom<&ExternalPointerAddress> for CoreLibValueId {
    type Error = ();
    fn try_from(address: &ExternalPointerAddress) -> Result<Self, Self::Error> {
        match address {
            ExternalPointerAddress::Builtin(bytes) => {
                let mut id_array = [0u8; 4];
                id_array[0..3].copy_from_slice(bytes);
                let id = u32::from_le_bytes(id_array);
                match CoreLibValueId::try_from_u16(id as u16) {
                    Some(core_id) => Ok(core_id),
                    None => Err(()),
                }
            }
            _ => Err(()),
        }
    }
}
