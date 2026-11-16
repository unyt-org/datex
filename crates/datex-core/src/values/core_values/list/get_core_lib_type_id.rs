use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    traits::get_core_lib_type_id::GetCoreLibTypeId,
    values::core_values::list::List,
};

impl GetCoreLibTypeId for List {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::List.into()
    }
}
