use crate::{
    preludes::derive::CoreLibTypeId,
    traits::get_core_lib_type_id::GetCoreLibTypeId, values::value::Value,
};

impl GetCoreLibTypeId for Value {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        (&self.inner).into()
    }
}
