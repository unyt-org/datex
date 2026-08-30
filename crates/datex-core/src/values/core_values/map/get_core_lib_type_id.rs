use crate::libs::core::type_id::CoreLibTypeId;
use crate::libs::core::type_id::CoreLibBaseTypeId;
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
use crate::values::core_values::map::Map;

impl GetCoreLibTypeId for Map {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Map.into()
    }
}