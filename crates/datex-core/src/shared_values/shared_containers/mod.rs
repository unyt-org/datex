mod owned_shared_container;
mod referenced_shared_container;
mod shared_container_inner;
mod ownership;
mod endpoint_owned_shared_container;
mod external_shared_container;
pub mod shared_type_container;
pub mod shared_value_container;
mod shared_container_mutability;

pub use owned_shared_container::*;
pub use referenced_shared_container::*;
pub use shared_container_inner::*;
pub use ownership::*;
pub use endpoint_owned_shared_container::*;
pub use external_shared_container::*;
pub use shared_container_mutability::*;

/// Top-level wrapper for any shared container,
/// which can either be an owned shared value or a reference to a shared value.
#[derive(Debug)]
pub enum SharedContainer {
    /// An owned shared value (`shared X`). This is always points to a [SharedContainerInner::EndpointOwned]
    Owned(OwnedSharedContainer),
    /// A referenced shared value (`'shared X` or `'mut shared X`).
    /// This can point to either a [SharedContainerInner::EndpointOwned] or a [SharedContainerInner::External]
    Referenced(ReferencedSharedContainer),
}