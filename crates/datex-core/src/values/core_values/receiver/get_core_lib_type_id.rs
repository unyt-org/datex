use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    prelude::*,
    traits::get_core_lib_type_id::GetCoreLibTypeId,
};
use crate::values::core_values::receiver::Receiver;

impl<T> GetCoreLibTypeId for Receiver<T> {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Receiver.into()
    }
}
