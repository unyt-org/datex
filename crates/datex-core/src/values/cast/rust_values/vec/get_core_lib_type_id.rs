use crate::preludes::derive::{CoreLibBaseTypeId, CoreLibTypeId};
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
use crate::prelude::*;

impl<T> GetCoreLibTypeId for Vec<T> {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::List.into()
    }
}