use core::fmt::Display;
use crate::shared_values::shared_containers::{ReferenceMutability, SharedContainerMutability, SharedContainerOwnership};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnexpectedSharedContainerOwnershipError {
    pub expected: SharedContainerOwnership,
    pub actual: SharedContainerOwnership,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnexpectedImmutableSharedContainerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnexpectedImmutableReferenceError;

impl Display for UnexpectedSharedContainerOwnershipError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Unexpected shared container ownership: expected {:?}, actual {:?}", self.expected, self.actual)
    }
}

impl Display for UnexpectedImmutableSharedContainerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Unexpected immutable shared container")
    }
}

impl Display for UnexpectedImmutableReferenceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Unexpected immutable reference")
    }
}