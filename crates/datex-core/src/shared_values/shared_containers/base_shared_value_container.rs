use crate::{
    shared_values::shared_containers::observers::Observer,
    traits::value_eq::ValueEq,
    types::type_definition::TypeDefinition,
    utils::freemap::FreeHashMap,
    values::{value::Value, value_container::ValueContainer},
};

use crate::{
    prelude::*,
    shared_values::{
        errors::SharedValueCreationError,
        shared_containers::SharedContainerMutability,
    },
};
use core::{
    cell::RefCell,
    fmt::{Debug, Display},
    prelude::rust_2024::*,
};
use crate::runtime::memory::Memory;
use crate::types::r#type::Type;

pub struct BaseSharedValueContainer {
    // pub(crate) pointer: Pointer,
    /// the value that this reference points to
    pub value_container: ValueContainer,
    /// custom type for the pointer that the Datex value is allowed to reference
    pub allowed_type: Type,
    /// list of observer callbacks
    /// TODO: move observers to ValueContainer?
    pub observers: FreeHashMap<u32, Observer>,
    pub mutability: SharedContainerMutability,
}

impl BaseSharedValueContainer {
    /// Tries to create a new [BaseSharedValueContainer] with an initial [ValueContainer],
    /// an allowed type and a [SharedContainerMutability].
    /// If the allowed [TypeDefinition] is not a superset of the [ValueContainer]'s allowed type,
    /// an error is returned
    pub fn try_new(
        value_container: ValueContainer,
        allowed_type: Type,
        mutability: SharedContainerMutability,
    ) -> Result<Self, SharedValueCreationError> {
        // TODO #286: make sure allowed type is superset of reference's allowed type

        Ok(BaseSharedValueContainer {
            value_container,
            allowed_type,
            observers: FreeHashMap::new(),
            mutability,
        })
    }

    /// Creates a new [BaseSharedValueContainer] with an initial [ValueContainer] and
    /// a [SharedContainerMutability].
    /// The allowed type is inferred from the value_container's allowed type.
    pub fn new_with_inferred_allowed_type<T: Into<ValueContainer>>(
        value_container: T,
        mutability: SharedContainerMutability,
        memory: &mut Memory
    ) -> Self {
        let value_container = value_container.into();
        let allowed_type = value_container.allowed_type(memory);
        BaseSharedValueContainer {
            value_container,
            allowed_type,
            observers: FreeHashMap::new(),
            mutability,
        }
    }

    /// Sets the currently assigned [ValueContainer] of the shared container to a new value container.
    /// Returns the [ValueContainer] as an error if the new value container's allowed type is not compatible with the allowed type of the shared container
    pub fn try_set_value_container(
        &mut self,
        new_value_container: ValueContainer,
    ) -> Result<(), ValueContainer> {
        // TODO do type checking to ensure new value container's allowed type is compatible with self.allowed_type
        self.value_container = new_value_container;
        Ok(())
    }
}

impl Debug for BaseSharedValueContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReferenceData")
            .field("value_container", &self.value_container)
            .field("allowed_type", &self.allowed_type)
            .field("observers", &self.observers.len())
            .finish()
    }
}

impl Display for BaseSharedValueContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "shared {}{}",
            self.value_container,
            match &self.mutability {
                SharedContainerMutability::Mutable => "mut ",
                SharedContainerMutability::Immutable => "",
            }
        )
    }
}

impl PartialEq for BaseSharedValueContainer {
    fn eq(&self, other: &Self) -> bool {
        // Two value references are equal if their value containers are equal
        self.value_container.value_eq(&other.value_container)
    }
}

impl BaseSharedValueContainer {
    pub fn current_value_container(&self) -> &ValueContainer {
        &self.value_container
    }

    pub fn resolve_current_value(&self) -> Rc<RefCell<Value>> {
        self.value_container.to_cloned_value()
    }

    pub fn is_mutable(&self) -> bool {
        core::matches!(self.mutability, SharedContainerMutability::Mutable)
    }
}
