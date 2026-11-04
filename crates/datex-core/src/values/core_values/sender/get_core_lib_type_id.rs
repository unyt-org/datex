use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    prelude::*,
    traits::get_core_lib_type_id::GetCoreLibTypeId,
};
use crate::values::core_values::sender::Sender;

impl<T> GetCoreLibTypeId for Sender<T> {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Sender.into()
    }
}
