use crate::{ast::resolved_variable::ResolvedVariable, prelude::*};

#[derive(Clone, Debug, PartialEq)]
pub struct VariantAccess {
    pub name: String,
    pub variant: String,
    pub base: ResolvedVariable,
}
