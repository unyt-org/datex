use crate::{
    ast::expressions::DatexExpression,
    shared_values::{
        PointerAddress, ReferenceMutability, SharedContainerMutability,
    },
    types::type_definition_with_metadata::LocalReferenceMutability,
};

#[derive(Clone, Debug, PartialEq)]
pub struct DeriveRef {
    pub mutability: LocalReferenceMutability,
    pub expression: DatexExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeriveSharedRef {
    pub mutability: ReferenceMutability,
    pub expression: DatexExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateMut {
    pub expression: DatexExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestSharedRef {
    pub mutability: ReferenceMutability,
    pub address: PointerAddress,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateShared {
    pub mutability: SharedContainerMutability,
    pub expression: DatexExpression,
}
