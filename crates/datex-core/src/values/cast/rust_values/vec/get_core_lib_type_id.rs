use crate::{
    prelude::*,
    preludes::derive::{CoreLibBaseTypeId, CoreLibTypeId},
    traits::get_core_lib_type_id::GetCoreLibTypeId,
};

impl<T> GetCoreLibTypeId for Vec<T> {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::List.into()
    }
}
