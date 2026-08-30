use crate::preludes::derive::{CoreLibBaseTypeId, CoreLibTypeId};

// Returns the DATEX [CoreLibTypeId] for the target
pub trait GetCoreLibTypeId {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Any.into()
    }
}