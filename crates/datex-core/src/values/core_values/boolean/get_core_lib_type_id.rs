use crate::libs::core::type_id::CoreLibTypeId;
use crate::libs::core::type_id::CoreLibBaseTypeId;
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
use crate::values::core_values::boolean::Boolean;

impl GetCoreLibTypeId for Boolean {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Boolean.into()
    }
}