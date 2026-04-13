use crate::values::core_value::CoreValue;
use core::result::Result;

pub use super::shared_containers::*;
pub use super::errors::*;

use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    shared_values::{
        pointer::{
            EndpointOwnedPointer, ExternalPointer,
        },
        pointer_address::{
            SelfOwnedPointerAddress, ExternalPointerAddress, PointerAddress,
        },
    },
    traits::{
        apply::Apply, identity::Identity, structural_eq::StructuralEq,
        value_eq::ValueEq,
    },
    types::structural_type_definition::StructuralTypeDefinition,
    values::{
        core_values::r#type::Type,
        value::Value,
        value_container::{ValueContainer, ValueKey},
    },
};
use core::{
    cell::{Ref, RefCell, RefMut},
    fmt::Display,
    hash::{Hash, Hasher},
    ops::FnOnce,
    option::Option,
    unreachable, write,
};
use core::fmt::Formatter;
use std::mem;
use binrw::{BinRead, BinWrite};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use crate::shared_values::shared_containers::shared_value_container::SharedValueContainer;



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SharedValueCreationError {
    InvalidType,
    MutabilityMismatch,
}

impl Display for SharedValueCreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SharedValueCreationError::InvalidType => {
                write!(
                    f,
                    "Cannot create shared value from value container: invalid type"
                )
            }
            SharedValueCreationError::MutabilityMismatch => {
                write!(
                    f,
                    "Cannot create mutable shared value for immutable value"
                )
            }
        }
    }
}

impl SharedContainerValueOrType {

    // /// Creates a new shared owned value containing the given value container
    // pub fn try_boxed_owned(
    //     shared_value_container: SharedValueContainer,
    //     pointer: EndpointOwnedPointer,
    // ) -> Result<Self, SharedValueCreationError> {
    //     let allowed_type =
    //         allowed_type.unwrap_or_else(|| value_container.allowed_type());
    //
    //     // TODO #286: make sure allowed type is superset of reference's allowed type
    //     Ok(SharedContainer {
    //         inner: Rc::new(RefCell::new(SharedContainerInner::EndpointOwned(EndpointOwnedSharedContainerInner {
    //             value_or_type: SharedContainerValueOrType::Value(SharedValueContainer::new(
    //                 value_container,
    //                 allowed_type,
    //                 mutability,
    //             )),
    //             pointer,
    //         }))),
    //         reference_mutability: None,
    //     })
    // }
    //
    // /// Creates a new shared ref value containing the given value container
    // pub fn try_boxed_ref(
    //     value_container: ValueContainer,
    //     allowed_type: Option<TypeDefinition>,
    //     pointer: ExternalPointer,
    //     mutability: SharedContainerMutability,
    //     reference_mutability: ReferenceMutability,
    // ) -> Result<Self, SharedValueCreationError> {
    //     let allowed_type =
    //         allowed_type.unwrap_or_else(|| value_container.allowed_type());
    //
    //     // TODO #286: make sure allowed type is superset of reference's allowed type
    //     Ok(SharedContainer {
    //         inner: Rc::new(RefCell::new(SharedContainerInner::External(ExternalSharedContainerInner {
    //             value_or_type: SharedContainerValueOrType::Value(SharedValueContainer::new(
    //                 value_container,
    //                 allowed_type,
    //                 mutability,
    //             )),
    //             pointer,
    //         }))),
    //         reference_mutability: Some(reference_mutability),
    //     })
    // }
    //
    // /// Create a new shared type container
    // /// The pointer must be an owned pointer, since we create a new shared value
    // pub fn new_from_type(
    //     type_value: Type,
    //     pointer: EndpointOwnedPointer,
    //     maybe_nominal_type_declaration: Option<NominalTypeDeclaration>,
    // ) -> Self {
    //     let type_reference = SharedContainerContainingType::new(
    //         type_value,
    //         maybe_nominal_type_declaration,
    //     );
    //     SharedContainer {
    //         inner: Rc::new(RefCell::new(SharedContainerInner::EndpointOwned(EndpointOwnedSharedContainerInner {
    //             value_or_type: SharedContainerValueOrType::Type(type_reference),
    //             pointer,
    //         }))),
    //         reference_mutability: None,
    //     }
    // }
    //
    // /// The pointer must be an owned pointer, since we create a new shared value
    // pub fn boxed_owned_mut(
    //     value_container: impl Into<ValueContainer>,
    //     pointer: EndpointOwnedPointer,
    // ) -> Self {
    //     SharedContainer::boxed_owned(
    //         value_container,
    //         pointer,
    //         SharedContainerMutability::Mutable,
    //     )
    // }
    //
    // /// The pointer must be an owned pointer, since we create a new shared value
    // pub fn boxed_owned_immut(
    //     value_container: impl Into<ValueContainer>,
    //     pointer: EndpointOwnedPointer,
    // ) -> Self {
    //     SharedContainer::boxed_owned(
    //         value_container,
    //         pointer,
    //         SharedContainerMutability::Immutable,
    //     )
    // }
    //
    // /// The pointer must be an owned pointer, since we create a new shared value
    // pub fn boxed_owned(
    //     value_container: impl Into<ValueContainer>,
    //     pointer: EndpointOwnedPointer,
    //     mutability: SharedContainerMutability
    // ) -> Self {
    //     SharedContainer::try_boxed_owned(
    //         value_container.into(),
    //         None,
    //         pointer,
    //         mutability
    //     )
    //         .unwrap() // always Ok, since we dont provide an allowed type that could mismatch
    // }
    //
    // /// Creates an owned pointer (no ref), but for an internal pointer address, not an OwnedPointer
    // pub(crate) fn boxed_owned_with_internal_pointer(
    //     value_container: impl Into<ValueContainer>,
    //     internal_pointer_address: [u8; 3],
    // ) -> Self {
    //     let value_container = value_container.into();
    //     let allowed_type = value_container.allowed_type();
    //
    //     SharedContainer {
    //         inner: Rc::new(RefCell::new(SharedContainerInner::External(ExternalSharedContainerInner {
    //             value_or_type: SharedContainerValueOrType::Value(SharedValueContainer::new(
    //                 value_container,
    //                 allowed_type,
    //                 SharedContainerMutability::Immutable,
    //             )),
    //             pointer: ExternalPointer::new(
    //                 ExternalPointerAddress::Builtin(
    //                     internal_pointer_address,
    //                 ),
    //             )
    //         }))),
    //         reference_mutability: None,
    //     }
    // }
    //
    // pub fn boxed_ref(
    //     value_container: impl Into<ValueContainer>,
    //     pointer: ExternalPointer,
    // ) -> Self {
    //     SharedContainer::try_boxed_ref(
    //         value_container.into(),
    //         None,
    //         pointer,
    //         SharedContainerMutability::Immutable,
    //         ReferenceMutability::Immutable,
    //     )
    //     .unwrap() // always Ok, since we dont provide an allowed type that could mismatch
    // }
    //
    // pub fn boxed_mut_ref(
    //     value_container: impl Into<ValueContainer>,
    //     pointer: ExternalPointer,
    // ) -> Self {
    //     SharedContainer::try_boxed_ref(
    //         value_container.into(),
    //         None,
    //         pointer,
    //         SharedContainerMutability::Mutable,
    //         ReferenceMutability::Mutable,
    //     )
    //     .unwrap() // always Ok, since we don't provide an allowed type that could mismatch
    // }

    /// Calls a fn with a mutable reference to the current inner collapsed value of the shared container
    pub(crate) fn with_collapsed_value_mut<R, F: FnOnce(&mut Value) -> R>(
        &self,
        f: F,
    ) -> R {
        match &mut *self.value_mut() {
            // FIXME #288: Can we optimize this to avoid creating rc ref cells?
            SharedContainerValueOrType::Type(tr) => {
                tr.with_collapsed_value(f)
            }
            SharedContainerValueOrType::Value(vr) => {
                match &mut vr.value_container {
                    ValueContainer::Shared(reference) => {
                        // If this is a reference, resolve it to its current value
                        reference.with_collapsed_value_mut(f)
                    }
                    ValueContainer::Local(value) => {
                        f(value)
                    }
                }
            }
        }
    }

    /// Calls a fn with a reference to the current inner collapsed value of the shared container
    pub(crate) fn with_collapsed_value<R, F: FnOnce(&Value) -> R>(
        &self,
        f: F,
    ) -> R {
        self.with_collapsed_value_mut(|value| f(value))
    }


    // TODO #290: no clone?
    pub fn value_container(&self) -> ValueContainer {
        match &*self.value() {
            SharedContainerValueOrType::Value(vr) => {
                vr.value_container.clone()
            }
            SharedContainerValueOrType::Type(tr) => ValueContainer::Local(
                Value::from(CoreValue::Type(tr.type_value.clone())),
            ),
        }
    }

    pub fn allowed_type(&self) -> StructuralTypeDefinition {
        match &*self.value() {
            SharedContainerValueOrType::Value(vr) => vr.allowed_type.clone(),
            SharedContainerValueOrType::Type(_) => core::todo!("#293 type Type"),
        }
    }

    pub fn actual_type(&self) -> StructuralTypeDefinition {
        match &*self.value() {
            SharedContainerValueOrType::Value(vr) => vr
                .value_container
                .to_cloned_value()
                .borrow()
                .actual_type()
                .clone(),
            SharedContainerValueOrType::Type(_tr) => core::todo!("#294 type Type"),
        }
    }

    pub fn is_type(&self) -> bool {
        match &*self.value() {
            SharedContainerValueOrType::Type(_) => true,
            SharedContainerValueOrType::Value(vr) => {
                vr.resolve_current_value().borrow().is_type()
            }
        }
    }

    /// Sets the value container of the reference if it is mutable.
    /// If the reference is immutable, an error is returned.
    pub fn set_value_container(
        &self,
        new_value_container: ValueContainer,
    ) -> Result<(), AssignmentError> {
        match &mut *self.value_mut() {
            SharedContainerValueOrType::Type(_) => {
                Err(AssignmentError::ImmutableReference)
            }
            SharedContainerValueOrType::Value(vr) => {
                if self.can_mutate() {
                    // TODO #295: check type compatibility, handle observers
                    vr.value_container = new_value_container;
                    Ok(())
                } else {
                    Err(AssignmentError::ImmutableReference)
                }
            }
        }
    }
}
/// Getter for references
impl SharedContainerValueOrType {
    /// Gets a property on the value if applicable (e.g. for map and structs)
    // FIXME #296 make this return a reference to a value container
    // Just for later as myRef.x += 1
    // key_ref = myRef.x // myRef.try_get_property("x".into())
    // key_val = &key_ref.value()
    // &key_ref.set_value(key_val + 1)
    // -> we could avoid some clones if so (as get, addition, set would all be a clone)
    pub fn try_get_property<'a>(
        &self,
        key: impl Into<ValueKey<'a>>,
    ) -> Result<ValueContainer, AccessError> {
        self.with_collapsed_value_mut(|value| value.try_get_property(key))
    }

    // Takes (removes) a property from the value if applicable
    pub fn try_take_property<'a>(
        &self,
        key: impl Into<ValueKey<'a>>,
    ) -> Result<ValueContainer, AccessError> {
        self.with_collapsed_value_mut(|value| value.try_take_property(key))
    }
}

impl Apply for SharedContainerValueOrType {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        match &*self.value() {
            SharedContainerValueOrType::Type(tr) => tr.apply(args),
            SharedContainerValueOrType::Value(vr) => {
                vr.resolve_current_value().borrow().apply(args)
            }
        }
    }

    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        match &*self.value() {
            SharedContainerValueOrType::Type(tr) => tr.apply_single(arg),
            SharedContainerValueOrType::Value(vr) => vr
                .resolve_current_value()
                .borrow()
                .apply_single(arg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assert_identical, assert_structural_eq, assert_value_eq, prelude::*,
        runtime::memory::Memory, traits::value_eq::ValueEq,
        values::core_values::map::Map,
    };
    use core::assert_matches;

    #[test]
    fn try_mut_from() {
        // creating a mutable shared container from a value should work
        let value = ValueContainer::from(42);
        let reference =
            SharedContainerValueOrType::boxed_owned_mut(value, EndpointOwnedPointer::NULL);
        assert_eq!(reference.mutability(), SharedContainerMutability::Mutable);
    }

    #[test]
    fn property() {
        let mut map = Map::default();
        map.set("name", ValueContainer::from("Jonas"));
        map.set("age", ValueContainer::from(30));
        let reference = SharedContainerValueOrType::boxed_owned_immut(
            ValueContainer::from(map),
            EndpointOwnedPointer::NULL,
        );
        assert_eq!(
            reference.try_get_property("name").unwrap(),
            ValueContainer::from("Jonas")
        );
        assert_eq!(
            reference.try_get_property("age").unwrap(),
            ValueContainer::from(30)
        );
        assert!(reference.try_get_property("nonexistent").is_err());
        assert_matches!(
            reference.try_get_property("nonexistent"),
            Err(AccessError::KeyNotFound(_))
        );
    }

    #[test]
    fn text_property() {
        let struct_val = Map::from(vec![
            ("name".to_string(), ValueContainer::from("Jonas")),
            ("age".to_string(), ValueContainer::from(30)),
        ]);
        let reference = SharedContainerValueOrType::boxed_owned_immut(
            ValueContainer::from(struct_val),
            EndpointOwnedPointer::NULL,
        );
        assert_eq!(
            reference.try_get_property("name").unwrap(),
            ValueContainer::from("Jonas")
        );
        assert_eq!(
            reference.try_get_property("age").unwrap(),
            ValueContainer::from(30)
        );
        assert!(reference.try_get_property("nonexistent").is_err());
        assert_matches!(
            reference.try_get_property("nonexistent"),
            Err(AccessError::KeyNotFound(_))
        );
    }

    #[test]
    fn numeric_property() {
        let list = vec![
            ValueContainer::from(1),
            ValueContainer::from(2),
            ValueContainer::from(3),
        ];
        let reference = SharedContainerValueOrType::boxed_owned_immut(
            ValueContainer::from(list),
            EndpointOwnedPointer::NULL,
        );

        assert_eq!(
            reference.try_get_property(0).unwrap(),
            ValueContainer::from(1)
        );
        assert_eq!(
            reference.try_get_property(1).unwrap(),
            ValueContainer::from(2)
        );
        assert_eq!(
            reference.try_get_property(2).unwrap(),
            ValueContainer::from(3)
        );
        assert!(reference.try_get_property(3).is_err());

        assert_matches!(
            reference.try_get_property(100),
            Err(AccessError::IndexOutOfBounds(IndexOutOfBoundsError {
                index: 100
            }))
        );

        let text_ref = SharedContainerValueOrType::boxed_owned_immut(
            ValueContainer::from("hello"),
            EndpointOwnedPointer::NULL,
        );
        assert_eq!(
            text_ref.try_get_property(1).unwrap(),
            ValueContainer::from("e".to_string())
        );
        assert!(text_ref.try_get_property(5).is_err());
        assert_matches!(
            text_ref.try_get_property(100),
            Err(AccessError::IndexOutOfBounds(IndexOutOfBoundsError {
                index: 100
            }))
        );
    }

    #[test]
    fn reference_identity() {
        let value = 42;
        let reference1 =
            SharedContainerValueOrType::boxed_owned_immut(value, EndpointOwnedPointer::NULL);
        let reference2 = reference1.clone();

        // cloned reference should be equal (identical)
        assert_eq!(reference1, reference2);
        // value containers containing the references should also be equal
        assert_eq!(
            ValueContainer::Shared(reference1.clone()),
            ValueContainer::Shared(reference2.clone())
        );
        // assert_identical! should also confirm identity
        assert_identical!(reference1.clone(), reference2);
        // separate reference containing the same value should not be equal
        assert_ne!(
            reference1,
            SharedContainerValueOrType::boxed_owned_immut(value, EndpointOwnedPointer::NULL)
        );
    }

    #[test]
    fn reference_value_equality() {
        let value = 42;
        let reference1 = ValueContainer::Shared(SharedContainerValueOrType::boxed_owned_immut(
            value,
            EndpointOwnedPointer::NULL,
        ));
        let reference2 = ValueContainer::Shared(SharedContainerValueOrType::boxed_owned_immut(
            value,
            EndpointOwnedPointer::NULL,
        ));

        // different references should not be equal a.k.a. identical
        assert_ne!(reference1, reference2);
        // but their current resolved values should be equal
        assert_value_eq!(reference1, ValueContainer::from(value));
    }

    #[test]
    fn reference_structural_equality() {
        let reference1 = SharedContainerValueOrType::boxed_owned_immut(42.0, EndpointOwnedPointer::NULL);
        let reference2 = SharedContainerValueOrType::boxed_owned_immut(42, EndpointOwnedPointer::NULL);

        // different references should not be equal a.k.a. identical
        assert_ne!(reference1, reference2);
        // but their current resolved values should be structurally equal
        assert!(!reference1.structural_eq(&reference2));
    }

    #[test]
    fn nested_references() {
        let memory = &RefCell::new(Memory::default());

        let mut map_a = Map::default();
        map_a.set("number", ValueContainer::from(42));
        map_a.set(
            "obj",
            ValueContainer::Shared(SharedContainerValueOrType::boxed_owned_immut(
                Map::default(),
                EndpointOwnedPointer::NULL,
            )),
        );

        // construct map_a as a value first
        let map_a_original_ref = ValueContainer::Shared(
            SharedContainerValueOrType::boxed_owned_immut(map_a, EndpointOwnedPointer::NULL),
        );

        // create map_b as a reference
        let map_b_ref = SharedContainerValueOrType::try_boxed_owned(
            Map::default().into(),
            None,
            EndpointOwnedPointer::NULL,
            SharedContainerMutability::Mutable,
        )
        .unwrap();

        // set map_a as property of b. This should create a reference to a clone of map_a that
        // is upgraded to a reference
        map_b_ref
            .try_set_property(0, None, "a", map_a_original_ref.clone())
            .unwrap();

        // assert that the reference to map_a is set correctly
        let map_a_ref = map_b_ref.try_get_property("a").unwrap();
        assert_structural_eq!(map_a_ref, map_a_original_ref);
        assert_eq!(map_a_ref, map_a_original_ref);
        assert_identical!(map_a_ref, map_a_original_ref);
        // map_a_ref should be a reference
        assert_matches!(map_a_ref, ValueContainer::Shared(_));
        map_a_ref.with_maybe_shared(|a_ref| {
            // map_a_ref.number should be a value
            assert_matches!(
                a_ref.try_get_property("number"),
                Ok(ValueContainer::Local(_))
            );
            // map_a_ref.obj should be a reference
            assert_matches!(
                a_ref.try_get_property("obj"),
                Ok(ValueContainer::Shared(_))
            );
        });
    }
}
