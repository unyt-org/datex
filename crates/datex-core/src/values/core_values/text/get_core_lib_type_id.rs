use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    traits::get_core_lib_type_id::GetCoreLibTypeId,
    values::core_values::text::Text,
};

impl GetCoreLibTypeId for Text {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Text.into()
    }
}
