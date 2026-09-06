use crate::{
    preludes::derive::{CoreLibBaseTypeId, CoreLibTypeId},
    random::RandomState,
    traits::get_core_lib_type_id::GetCoreLibTypeId,
};
use core::hash::Hash;
use indexmap::IndexMap;

impl<K: Eq + Hash, V> GetCoreLibTypeId for IndexMap<K, V, RandomState> {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibBaseTypeId::Map.into()
    }
}
