use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    traits::get_core_lib_type_id::GetCoreLibTypeId,
    values::core_values::map::Map,
};

impl GetCoreLibTypeId for Map {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Map.into()
    }
}
