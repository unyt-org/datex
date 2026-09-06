use crate::preludes::derive::{CoreLibBaseTypeId, SharedReferencesCache, Type};

// Returns the DATEX [Type] for the target
pub trait GetDatexType {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::Any)
    }
}
