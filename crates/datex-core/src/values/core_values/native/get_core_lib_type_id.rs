use crate::libs::core::type_id::CoreLibTypeId;
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
use crate::values::core_values::native::NativeCoreValue;

impl GetCoreLibTypeId for NativeCoreValue {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        self.value.core_lib_type_id()
    }
}