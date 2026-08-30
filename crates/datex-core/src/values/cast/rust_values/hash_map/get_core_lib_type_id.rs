use core::hash::Hash;
use crate::collections::HashMap;
use crate::preludes::derive::{CoreLibBaseTypeId, CoreLibTypeId};
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;

impl<K: Eq + Hash, V> GetCoreLibTypeId for HashMap<K, V> {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Map.into()
    }
}