use crate::{
    libs::core::type_id::CoreLibTypeId, prelude::*,
    traits::get_core_lib_type_id::GetCoreLibTypeId,
};

impl<T: GetCoreLibTypeId> GetCoreLibTypeId for Box<T> {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        (**self).core_lib_type_id()
    }
}
