use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    traits::get_core_lib_type_id::GetCoreLibTypeId,
    values::core_values::boolean::Boolean,
};

impl GetCoreLibTypeId for Boolean {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Boolean.into()
    }
}
