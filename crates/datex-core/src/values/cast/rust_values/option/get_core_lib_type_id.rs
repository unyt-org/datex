use crate::{
    preludes::derive::{CoreLibBaseTypeId, CoreLibTypeId},
    traits::get_core_lib_type_id::GetCoreLibTypeId,
};

impl<T: GetCoreLibTypeId> GetCoreLibTypeId for Option<T> {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        match self {
            Some(value) => value.core_lib_type_id(),
            None => CoreLibBaseTypeId::Null.into(),
        }
    }
}
