//! This module contains the implementation of the shared values system.
//! It includes the [shared_container::SharedContainer], [shared_container_inner::SharedContainerInner], [SelfOwnedSharedContainer]/[ExternalSharedContainer] and finally [base_shared_value_container::BaseSharedValueContainer] which is the underlying data structure for shared values, as well as various types of shared containers such as owned, referenced, and self-owned shared containers.
pub mod base_shared_value_container;
pub mod collapsed_container_value;
pub mod errors;
mod external_shared_container;
mod owned_shared_container;
mod ownership;
mod pointer_address;
mod referenced_shared_container;
mod self_owned_shared_container;
mod shared_container;
mod shared_container_inner;
mod shared_container_mutability;
pub mod shared_mut;
mod subscriber;
pub mod traits;
pub mod weak_shared_container;

pub use external_shared_container::*;
pub use ownership::*;
pub use pointer_address::*;
pub use self_owned_shared_container::*;
pub use shared_container::*;
pub use shared_container_inner::*;
pub use shared_container_mutability::*;
pub use subscriber::*;
