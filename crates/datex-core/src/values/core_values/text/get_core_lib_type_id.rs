use crate::libs::core::type_id::CoreLibTypeId;
use crate::libs::core::type_id::CoreLibBaseTypeId;
use crate::preludes::derive::Text;
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;

impl GetCoreLibTypeId for Text {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Text.into()
    }
}