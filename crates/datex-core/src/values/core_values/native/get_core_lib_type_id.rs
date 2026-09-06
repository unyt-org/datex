use crate::{
    libs::core::type_id::CoreLibTypeId,
    traits::get_core_lib_type_id::GetCoreLibTypeId,
    values::core_values::native::NativeCoreValue,
};

impl GetCoreLibTypeId for NativeCoreValue {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        self.value.core_lib_type_id()
    }
}
