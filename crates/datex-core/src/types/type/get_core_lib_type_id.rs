use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    traits::get_core_lib_type_id::GetCoreLibTypeId,
    types::r#type::Type,
};

impl GetCoreLibTypeId for Type {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Type.into()
    }
}
