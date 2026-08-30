use crate::libs::core::type_id::CoreLibTypeId;
use crate::libs::core::type_id::CoreLibBaseTypeId;
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
use crate::values::core_values::list::List;

impl GetCoreLibTypeId for List {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::List.into()
    }
}