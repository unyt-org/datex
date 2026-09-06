use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    preludes::derive::Text,
    traits::get_core_lib_type_id::GetCoreLibTypeId,
};

impl GetCoreLibTypeId for Text {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Text.into()
    }
}
