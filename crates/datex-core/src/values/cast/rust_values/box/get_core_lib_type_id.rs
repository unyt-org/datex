use core::ops::Deref;
use crate::preludes::derive::{CoreLibTypeId};
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
use crate::prelude::*;

impl<T: GetCoreLibTypeId> GetCoreLibTypeId for Box<T> {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        self.deref().core_lib_type_id()
    }
}