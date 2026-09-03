use core::hash::Hash;
use indexmap::IndexMap;
use crate::preludes::derive::{CoreLibBaseTypeId, CoreLibTypeId};
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
use crate::random::RandomState;

impl<K: Eq + Hash, V> GetCoreLibTypeId for IndexMap<K, V, RandomState> {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Map.into()
    }
}