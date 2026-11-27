use crate::{
    libs::core::core_lib_id::CoreLibId, shared_values::PointerAddress,
};
use core::fmt::Display;

pub type VariableId = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedVariable {
    VariableId(usize),
    PointerAddress(PointerAddress),
    CoreLibId(CoreLibId),
}

impl Display for ResolvedVariable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ResolvedVariable::VariableId(id) => core::write!(f, "#{}", id),
            ResolvedVariable::PointerAddress(addr) => {
                core::write!(f, "{}", addr)
            }
            ResolvedVariable::CoreLibId(id) => {
                core::write!(f, "{}", id)
            }
        }
    }
}
