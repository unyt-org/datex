use crate::{
    prelude::*, preludes::derive::CoreLibTypeId,
    traits::get_core_lib_type_id::GetCoreLibTypeId,
};
use core::ops::Deref;

impl<T: GetCoreLibTypeId> GetCoreLibTypeId for Box<T> {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        self.deref().core_lib_type_id()
    }
}
