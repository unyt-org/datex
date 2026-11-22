use crate::shared_values::ReferenceMutability;

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

#[derive(Clone, Debug, PartialEq, Default, Hash, Eq)]
pub enum ValueAccessType {
    /// 'mut x
    SharedRefMut,
    /// 'x
    SharedRef,
    /// clone x
    Clone, // TODO: allow clone only for non-shared values, for shared values, do clone *x
    /// temp borrow for x (used before Unbox)
    Borrow,
    #[default]
    MoveOrCopy,
}

impl From<&ReferenceMutability> for ValueAccessType {
    fn from(mutability: &ReferenceMutability) -> Self {
        match mutability {
            ReferenceMutability::Mutable => ValueAccessType::SharedRefMut,
            ReferenceMutability::Immutable => ValueAccessType::SharedRef,
        }
    }
}
