use core::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VariableKind {
    Const,
    Var,
}

impl Display for VariableKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VariableKind::Const => core::write!(f, "const"),
            VariableKind::Var => core::write!(f, "var"),
        }
    }
}
