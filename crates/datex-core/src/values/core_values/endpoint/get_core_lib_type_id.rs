use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    traits::get_core_lib_type_id::GetCoreLibTypeId,
    values::core_values::endpoint::Endpoint,
};

impl GetCoreLibTypeId for Endpoint {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Endpoint.into()
    }
}
