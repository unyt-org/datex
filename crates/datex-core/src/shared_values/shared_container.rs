use crate::{
    shared_values::shared_type_container::{NominalTypeDeclaration, SharedTypeContainer},
    values::core_value::CoreValue,
};
use core::result::Result;

use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    shared_values::shared_value_container::SharedValueContainer,
    traits::{
        apply::Apply, identity::Identity, structural_eq::StructuralEq,
        value_eq::ValueEq,
    },
    types::definition::TypeDefinition,
    values::{
        core_values::{map::MapAccessError, r#type::Type},
        value::Value,
        value_container::{ValueContainer, ValueKey},
    },
};
use core::{
    cell::RefCell,
    fmt::Display,
    hash::{Hash, Hasher},
    ops::FnOnce,
    option::Option,
    unreachable, write,
};
use core::cell::Ref;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use crate::shared_values::pointer::{Pointer, PointerReferenceMutability};
use crate::shared_values::pointer_address::PointerAddress;
use crate::values::core_values::r#type::TypeMetadata;

#[derive(Debug)]
pub struct IndexOutOfBoundsError {
    pub index: u32,
}

impl Display for IndexOutOfBoundsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Index out of bounds: {}", self.index)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyNotFoundError {
    pub key: ValueContainer,
}

impl Display for KeyNotFoundError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Property not found: {}", self.key)
    }
}

#[derive(Debug)]
pub enum AccessError {
    ImmutableReference,
    InvalidOperation(String),
    KeyNotFound(KeyNotFoundError),
    IndexOutOfBounds(IndexOutOfBoundsError),
    MapAccessError(MapAccessError),
    InvalidIndexKey,
}

impl From<IndexOutOfBoundsError> for AccessError {
    fn from(err: IndexOutOfBoundsError) -> Self {
        AccessError::IndexOutOfBounds(err)
    }
}

impl From<MapAccessError> for AccessError {
    fn from(err: MapAccessError) -> Self {
        AccessError::MapAccessError(err)
    }
}

impl From<KeyNotFoundError> for AccessError {
    fn from(err: KeyNotFoundError) -> Self {
        AccessError::KeyNotFound(err)
    }
}

impl Display for AccessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AccessError::MapAccessError(err) => {
                write!(f, "Map access error: {}", err)
            }
            AccessError::ImmutableReference => {
                write!(f, "Cannot modify an immutable reference")
            }
            AccessError::InvalidOperation(op) => {
                write!(f, "Invalid operation: {}", op)
            }
            AccessError::KeyNotFound(key) => {
                write!(f, "{}", key)
            }
            AccessError::IndexOutOfBounds(error) => {
                write!(f, "{}", error)
            }
            AccessError::InvalidIndexKey => {
                write!(f, "Invalid index key")
            }
        }
    }
}

#[derive(Debug)]
pub enum TypeError {
    TypeMismatch { expected: Type, found: Type },
}
impl Display for TypeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TypeError::TypeMismatch { expected, found } => write!(
                f,
                "Type mismatch: expected {}, found {}",
                expected, found
            ),
        }
    }
}

#[derive(Debug)]
pub enum AssignmentError {
    ImmutableReference,
    TypeError(Box<TypeError>),
}

impl Display for AssignmentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AssignmentError::ImmutableReference => {
                write!(f, "Cannot assign to an immutable reference")
            }
            AssignmentError::TypeError(e) => {
                write!(f, "Type error: {}", e)
            }
        }
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TryFromPrimitive,
)]
#[repr(u8)]
pub enum SharedContainerMutability {
    Mutable = 0,
    Immutable = 1,
}

pub mod mutability_as_int {
    use super::SharedContainerMutability;
    use crate::prelude::*;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(
        value: &SharedContainerMutability,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            SharedContainerMutability::Mutable => serializer.serialize_u8(0),
            SharedContainerMutability::Immutable => serializer.serialize_u8(1),
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<SharedContainerMutability, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = u8::deserialize(deserializer)?;
        Ok(match opt {
            0 => SharedContainerMutability::Mutable,
            1 => SharedContainerMutability::Immutable,
            x => {
                return Err(D::Error::custom(format!(
                    "invalid mutability code: {}",
                    x
                )));
            }
        })
    }
}
pub mod mutability_option_as_int {
    use super::SharedContainerMutability;

    use crate::prelude::*;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(
        value: &Option<SharedContainerMutability>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(SharedContainerMutability::Mutable) => serializer.serialize_u8(0),
            Some(SharedContainerMutability::Immutable) => serializer.serialize_u8(1),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<SharedContainerMutability>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<u8>::deserialize(deserializer)?;
        Ok(match opt {
            Some(0) => Some(SharedContainerMutability::Mutable),
            Some(1) => Some(SharedContainerMutability::Immutable),
            Some(x) => {
                return Err(D::Error::custom(format!(
                    "invalid mutability code: {}",
                    x
                )));
            }
            None => None,
        })
    }
}

impl Display for SharedContainerMutability {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SharedContainerMutability::Mutable => write!(f, "mut"),
            SharedContainerMutability::Immutable => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SharedContainer {
    Value(Rc<RefCell<SharedValueContainer>>),
    Type(Rc<RefCell<SharedTypeContainer>>),
}

impl Display for SharedContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SharedContainer::Value(vr) => {
                let vr = vr.borrow();
                write!(f, "{} {}", vr.mutability, vr.value_container)
            }
            SharedContainer::Type(tr) => {
                let tr = tr.borrow();
                write!(f, "{}", tr)
            }
        }
    }
}

impl From<SharedValueContainer> for SharedContainer {
    fn from(reference: SharedValueContainer) -> Self {
        SharedContainer::Value(Rc::new(RefCell::new(reference)))
    }
}
impl From<SharedTypeContainer> for SharedContainer {
    fn from(reference: SharedTypeContainer) -> Self {
        SharedContainer::Type(Rc::new(RefCell::new(reference)))
    }
}

/// Two references are identical if they point to the same data
impl Identity for SharedContainer {
    fn identical(&self, other: &Self) -> bool {
        match (self, other) {
            (SharedContainer::Value(a), SharedContainer::Value(b)) => {
                Rc::ptr_eq(a, b)
            }
            (SharedContainer::Type(a), SharedContainer::Type(b)) => {
                Rc::ptr_eq(a, b)
            }
            _ => false,
        }
    }
}

impl Eq for SharedContainer {}

/// PartialEq corresponds to pointer equality / identity for `Reference`.
impl PartialEq for SharedContainer {
    fn eq(&self, other: &Self) -> bool {
        self.identical(other)
    }
}

impl StructuralEq for SharedContainer {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SharedContainer::Type(a), SharedContainer::Type(b)) => {
                a.borrow().type_value.structural_eq(&b.borrow().type_value)
            }
            (SharedContainer::Value(a), SharedContainer::Value(b)) => a
                .borrow()
                .value_container
                .structural_eq(&b.borrow().value_container),
            _ => false,
        }
    }
}

impl ValueEq for SharedContainer {
    fn value_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SharedContainer::Type(a), SharedContainer::Type(b)) => {
                // FIXME #281: Implement value_eq for type and use here instead (recursive)
                a.borrow().type_value.structural_eq(&b.borrow().type_value)
            }
            (SharedContainer::Value(a), SharedContainer::Value(b)) => a
                .borrow()
                .value_container
                .value_eq(&b.borrow().value_container),
            _ => false,
        }
    }
}

// x = mut shared {x: 1};
// y = mut {a: 2};
// mut shared {x: 1};
// &mut shared {a: 3};

impl Hash for SharedContainer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            SharedContainer::Type(tr) => {
                let ptr = Rc::as_ptr(tr);
                ptr.hash(state); // hash the address
            }
            SharedContainer::Value(vr) => {
                let ptr = Rc::as_ptr(vr);
                ptr.hash(state); // hash the address
            }
        }
    }
}

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
                write!(f, "Cannot create mutable shared value for immutable value")
            }
        }
    }
}

impl SharedContainer {
    /// Runs a closure with the current value of this reference.
    pub(crate) fn with_value<R, F: FnOnce(&mut Value) -> R>(
        &self,
        f: F,
    ) -> Option<R> {
        let reference = self.collapse_reference_chain();
        match reference {
            SharedContainer::Value(vr) => {
                match &mut vr.borrow_mut().value_container {
                    ValueContainer::Local(value) => Some(f(value)),
                    ValueContainer::Shared(_) => {
                        unreachable!(
                            "Expected a ValueContainer::Value, but found a Reference"
                        )
                    }
                }
            }
            SharedContainer::Type(_) => None,
        }
    }

    // TODO #282: Mark as unsafe function
    /// Note: borrows the contained value. While in callback, no other borrows to the value are allowed.
    pub(crate) fn with_value_unchecked<R, F: FnOnce(&mut Value) -> R>(
        &self,
        f: F,
    ) -> R {
        unsafe { self.with_value(f).unwrap_unchecked() }
    }
}

impl SharedContainer {
    pub fn pointer_address(&self) -> PointerAddress {
        match self {
            SharedContainer::Value(vr) => {
                vr.borrow().pointer.address()
            }
            SharedContainer::Type(tr) => {
                tr.borrow().pointer.address()
            }
        }
    }

    pub fn pointer(&self) -> Ref<Pointer> {
        match self {
            SharedContainer::Value(vr) => {
                Ref::map(vr.borrow(), |vr| &vr.pointer)
            }
            SharedContainer::Type(tr) => {
                Ref::map(tr.borrow(), |tr| &tr.pointer)
            }
        }
    }

    /// Gets the mutability of the shared value.
    /// TypeReferences are always immutable.
    pub(crate) fn mutability(&self) -> SharedContainerMutability {
        match self {
            SharedContainer::Value(vr) => vr.borrow().mutability.clone(),
            SharedContainer::Type(_) => SharedContainerMutability::Immutable,
        }
    }
    
    /// Checks if the reference can be mutated by the current endpoint.
    pub(crate) fn can_mutate(&self) -> bool {
        match self.pointer().reference_mutability() {
            // is &mut reference to a pointer
            Some(PointerReferenceMutability::Mutable) => true,
            // is owned pointer, check if the shared container is mutable
            None => self.mutability() == SharedContainerMutability::Mutable,
            _ => false,
        }
    }

    /// Checks if the reference is mutable.
    /// A reference is mutable if it is a mutable ValueReference and all references in the chain are mutable.
    /// TypeReferences are always immutable.
    /// FIXME #284: Do we really need this? Probably we already collapse the ref and then change it's value and perform
    /// the mutability check on the most inner ref.
    pub fn is_mutable(&self) -> bool {
        match self {
            SharedContainer::Type(_) => false, // type references are always immutable
            SharedContainer::Value(vr) => {
                let vr_borrow = vr.borrow();
                // if the current reference is immutable, whole chain is immutable
                if vr_borrow.mutability != SharedContainerMutability::Mutable {
                    return false;
                }

                // otherwise, check if ref is pointing to another reference
                match &vr_borrow.value_container {
                    ValueContainer::Shared(inner) => inner.is_mutable(),
                    ValueContainer::Local(_) => true,
                }
            }
        }
    }

    /// Creates a new shared value containing the given value container
    pub fn try_boxed(
        value_container: ValueContainer,
        allowed_type: Option<TypeDefinition>,
        pointer: Pointer,
        mutability: SharedContainerMutability,
    ) -> Result<Self, SharedValueCreationError> {

        // immutable shared value cannot be contained in a mutable shared container, since we want interior mutability
        if let ValueContainer::Shared(ref shared) = value_container &&
            mutability == SharedContainerMutability::Mutable && shared.mutability() != SharedContainerMutability::Mutable {
                return Err(SharedValueCreationError::MutabilityMismatch);
        }
        let allowed_type = allowed_type.unwrap_or_else(|| {
            value_container.allowed_type()
        });

        // TODO #286: make sure allowed type is superset of reference's allowed type
        Ok(
            SharedContainer::Value(Rc::new(RefCell::new(
                SharedValueContainer::new(
                    value_container,
                    pointer,
                    allowed_type,
                    mutability,
                ),
            )))
        )
    }

    pub fn new_from_type(
        type_value: Type,
        pointer: Pointer,
        maybe_nominal_type_declaration: Option<NominalTypeDeclaration>,
    ) -> Self {
        let type_reference = SharedTypeContainer {
            pointer,
            nominal_type_declaration: maybe_nominal_type_declaration,
            type_value,
        };
        SharedContainer::Type(Rc::new(RefCell::new(type_reference)))
    }

    pub fn boxed_mut(
        value_container: ValueContainer,
        pointer: Pointer,
    ) -> Result<Self, SharedValueCreationError> {
        SharedContainer::try_boxed(
            value_container,
            None,
            pointer,
            SharedContainerMutability::Mutable,
        )
    }

    pub fn boxed(
        value_container: impl Into<ValueContainer>,
        pointer: Pointer,
    ) -> Self {
        SharedContainer::try_boxed(
            value_container.into(),
            None,
            pointer,
            SharedContainerMutability::Immutable,
        ).unwrap()
    }

    /// Collapses the reference chain to most inner reference to which this reference points.
    pub fn collapse_reference_chain(&self) -> SharedContainer {
        match self {
            // FIXME #288: Can we optimize this to avoid creating rc ref cells?
            SharedContainer::Type(tr) => SharedContainer::Type(Rc::new(
                RefCell::new(tr.borrow().collapse_reference_chain()),
            )),
            SharedContainer::Value(vr) => {
                match &vr.borrow().value_container {
                    ValueContainer::Shared(reference) => {
                        // If this is a reference, resolve it to its current value
                        reference.collapse_reference_chain()
                    }
                    ValueContainer::Local(_) => {
                        // If this is a value, return it directly
                        self.clone()
                    }
                }
            }
        }
    }

    /// Converts a reference to its current value, collapsing any reference chains and converting type references to type values.
    pub fn collapse_to_value(&self) -> Rc<RefCell<Value>> {
        let reference = self.collapse_reference_chain();
        match reference {
            SharedContainer::Value(vr) => match &vr.borrow().value_container
            {
                ValueContainer::Local(_) => {
                    vr.borrow().value_container.to_value()
                }
                ValueContainer::Shared(_) => unreachable!(
                    "Expected a ValueContainer::Value, but found a Reference"
                ),
            },
            // TODO #289: can we optimize this to avoid cloning the type value?
            SharedContainer::Type(tr) => Rc::new(RefCell::new(Value::from(
                CoreValue::Type(tr.borrow().type_value.clone()),
            ))),
        }
    }

    // TODO #290: no clone?
    pub fn value_container(&self) -> ValueContainer {
        match self {
            SharedContainer::Value(vr) => {
                vr.borrow().value_container.clone()
            }
            SharedContainer::Type(tr) => ValueContainer::Local(Value::from(
                CoreValue::Type(tr.borrow().type_value.clone()),
            )),
        }
    }

    pub fn allowed_type(&self) -> TypeDefinition {
        match self {
            SharedContainer::Value(vr) => vr.borrow().allowed_type.clone(),
            SharedContainer::Type(_) => core::todo!("#293 type Type"),
        }
    }

    pub fn actual_type(&self) -> TypeDefinition {
        match self {
            SharedContainer::Value(vr) => vr
                .borrow()
                .value_container
                .to_value()
                .borrow()
                .actual_type()
                .clone(),
            SharedContainer::Type(_tr) => core::todo!("#294 type Type"),
        }
    }

    pub fn is_type(&self) -> bool {
        match self {
            SharedContainer::Type(_) => true,
            SharedContainer::Value(vr) => {
                vr.borrow().resolve_current_value().borrow().is_type()
            }
        }
    }

    /// Returns a mutable reference to the ValueReference if this is a mutable ValueReference.
    pub fn mutable_reference(&self) -> Option<Rc<RefCell<SharedValueContainer>>> {
        match self {
            SharedContainer::Type(_) => None,
            SharedContainer::Value(vr) => {
                if vr.borrow().is_mutable() {
                    Some(vr.clone())
                } else {
                    None
                }
            }
        }
    }

    /// Sets the value container of the reference if it is mutable.
    /// If the reference is immutable, an error is returned.
    pub fn set_value_container(
        &self,
        new_value_container: ValueContainer,
    ) -> Result<(), AssignmentError> {
        match &self {
            SharedContainer::Type(_) => {
                Err(AssignmentError::ImmutableReference)
            }
            SharedContainer::Value(vr) => {
                if self.is_mutable() {
                    // TODO #295: check type compatibility, handle observers
                    vr.borrow_mut().value_container = new_value_container;
                    Ok(())
                } else {
                    Err(AssignmentError::ImmutableReference)
                }
            }
        }
    }
}
/// Getter for references
impl SharedContainer {
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
        self.with_value(|value| value.try_get_property(key))
            .unwrap_or(Err(AccessError::InvalidOperation(
                "Cannot get property on invalid reference".to_string(),
            )))
    }
}

impl Apply for SharedContainer {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        match self {
            SharedContainer::Type(tr) => tr.borrow().apply(args),
            SharedContainer::Value(vr) => {
                vr.borrow().resolve_current_value().borrow().apply(args)
            }
        }
    }

    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        match self {
            SharedContainer::Type(tr) => tr.borrow().apply_single(arg),
            SharedContainer::Value(vr) => vr
                .borrow()
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
        let reference = SharedContainer::boxed_mut(
            value,
            Pointer::NULL,
        ).unwrap();
        assert_eq!(reference.mutability(), SharedContainerMutability::Mutable);

        // creating a mutable reference from a type should fail
        let type_value = ValueContainer::Shared(SharedContainer::Type(
            SharedTypeContainer::anonymous(Type::UNIT, Pointer::NULL).as_ref_cell(),
        ));
        assert_matches!(
            SharedContainer::boxed_mut(type_value, Pointer::NULL),
            Err(SharedValueCreationError::MutabilityMismatch)
        );
    }

    #[test]
    fn property() {
        let mut map = Map::default();
        map.set("name", ValueContainer::from("Jonas"));
        map.set("age", ValueContainer::from(30));
        let reference = SharedContainer::boxed(ValueContainer::from(map), Pointer::NULL);
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
        let reference = SharedContainer::boxed(ValueContainer::from(struct_val), Pointer::NULL);
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
        let reference = SharedContainer::boxed(ValueContainer::from(list), Pointer::NULL);

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

        let text_ref = SharedContainer::boxed(ValueContainer::from("hello"), Pointer::NULL);
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
        let reference1 = SharedContainer::boxed(value, Pointer::NULL);
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
        assert_ne!(reference1, SharedContainer::boxed(value, Pointer::NULL));
    }

    #[test]
    fn reference_value_equality() {
        let value = 42;
        let reference1 = ValueContainer::Shared(SharedContainer::boxed(value, Pointer::NULL));
        let reference2 = ValueContainer::Shared(SharedContainer::boxed(value, Pointer::NULL));

        // different references should not be equal a.k.a. identical
        assert_ne!(reference1, reference2);
        // but their current resolved values should be equal
        assert_value_eq!(reference1, ValueContainer::from(value));
    }

    #[test]
    fn reference_structural_equality() {
        let reference1 = SharedContainer::boxed(42.0, Pointer::NULL);
        let reference2 = SharedContainer::boxed(42, Pointer::NULL);

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
        map_a.set("obj", ValueContainer::Shared(SharedContainer::boxed(Map::default(), Pointer::NULL)));

        // construct map_a as a value first
        let map_a_original_ref = ValueContainer::Shared(SharedContainer::boxed(map_a, Pointer::NULL));

        // create map_b as a reference
        let map_b_ref = SharedContainer::try_boxed(
            Map::default().into(),
            None,
            Pointer::NULL,
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
