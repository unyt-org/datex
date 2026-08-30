use crate::preludes::derive::CoreLibTypeId;
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
use crate::values::value::Value;

impl GetCoreLibTypeId for Value {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        (&self.inner).into()
    }
}