use crate::{
    shared_values::shared_type_container::{
        NominalTypeDeclaration, SharedTypeContainer,
    },
    values::core_value::CoreValue,
};
use core::result::Result;

use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    shared_values::{
        pointer::{
            OwnedPointer, PointerReferenceMutability,
            ReferencedPointer,
        },
        pointer_address::{
            OwnedPointerAddress, PointerAddress, ReferencedPointerAddress,
        },
        shared_value_container::SharedValueContainer,
    },
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
    cell::{Ref, RefMut, RefCell},
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
    Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TryFromPrimitive, BinRead, BinWrite)]
#[brw(repr(u8))]
#[repr(u8)]
pub enum SharedContainerMutability {
    Immutable = 0,
    Mutable = 1,
}

pub mod mutability_as_int {
    use super::SharedContainerMutability;
    use crate::prelude::*;
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

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
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<S>(
        value: &Option<SharedContainerMutability>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(SharedContainerMutability::Mutable) => {
                serializer.serialize_u8(0)
            }
            Some(SharedContainerMutability::Immutable) => {
                serializer.serialize_u8(1)
            }
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

#[derive(Debug)]
pub(crate) struct SharedContainer {
    pub inner: Rc<RefCell<SharedContainerInner>>,
    pub reference_mutability: Option<PointerReferenceMutability>,
}


#[derive(Debug)]
pub struct OwnedSharedContainerInner {
    pub value_or_type: SharedContainerValueOrType,
    pub pointer: OwnedPointer,
}

#[derive(Debug)]
pub struct ReferencedSharedContainerInner {
    pub value_or_type: SharedContainerValueOrType,
    pub pointer: ReferencedPointer,
}

// FIXME: try to deprecate clone
impl Clone for SharedContainer {
    fn clone(&self) -> Self {
        SharedContainer {
            inner: self.inner.clone(),
            reference_mutability: Some(PointerReferenceMutability::Immutable),
        }
    }
}


#[derive(Debug)]
pub enum SharedContainerInner {
    Owned(OwnedSharedContainerInner),
    Referenced(ReferencedSharedContainerInner),
}

impl SharedContainerInner {
    pub fn value(&self) -> &SharedContainerValueOrType {
        match self {
            SharedContainerInner::Owned(owned) => &owned.value_or_type,
            SharedContainerInner::Referenced(referenced) => &referenced.value_or_type,
        }
    }

    pub fn value_mut(&mut self) -> &mut SharedContainerValueOrType {
        match self {
            SharedContainerInner::Owned(owned) => &mut owned.value_or_type,
            SharedContainerInner::Referenced(referenced) => &mut referenced.value_or_type,
        }
    }

    pub fn take_value(self) -> SharedContainerValueOrType {
        match self {
            SharedContainerInner::Owned(owned) => owned.value_or_type,
            SharedContainerInner::Referenced(referenced) => referenced.value_or_type,
        }
    }


    pub fn pointer_address(&self) -> PointerAddress {
        match self {
            SharedContainerInner::Owned(owned) => PointerAddress::Owned(owned.pointer.address().clone()),
            SharedContainerInner::Referenced(referenced) => PointerAddress::Referenced(referenced.pointer.address().clone()),
        }
    }

    pub fn change_to_referenced(&mut self, referenced_pointer: ReferencedPointer) {
        // mem replace workaround to get owned value_or_type
        let original_value_or_type = mem::replace(self, SharedContainerInner::Owned(OwnedSharedContainerInner {
            value_or_type: SharedContainerValueOrType::Value(SharedValueContainer {
                value_container: ValueContainer::Local(Value::null()),
                allowed_type: TypeDefinition::Unit,
                observers: Default::default(),
                mutability: SharedContainerMutability::Immutable,
            }),
            pointer: OwnedPointer::NULL,
        })).take_value();

        *self = SharedContainerInner::Referenced(ReferencedSharedContainerInner {
            value_or_type: original_value_or_type,
            pointer: referenced_pointer
        });
    }
}

impl Display for SharedContainerInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            SharedContainerInner::Owned(owned) => write!(f, "{}", owned.value_or_type),
            SharedContainerInner::Referenced(referenced) => write!(f, "{}", referenced.value_or_type),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SharedContainerValueOrType {
    Value(SharedValueContainer),
    Type(SharedTypeContainer),
}

impl SharedContainerValueOrType {
    /// Gets the mutability of the shared value.
    /// TypeReferences are always immutable.
    pub(crate) fn mutability(&self) -> SharedContainerMutability {
        match &self {
            SharedContainerValueOrType::Value(vr) => vr.mutability.clone(),
            SharedContainerValueOrType::Type(_) => {
                SharedContainerMutability::Immutable
            }
        }
    }
}

impl Display for SharedContainerValueOrType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // value
        match &self {
            SharedContainerValueOrType::Value(vr) => {
                write!(f, "{} {}", vr.mutability, vr.value_container)
            }
            SharedContainerValueOrType::Type(tr) => {
                write!(f, "{}", tr)
            }
        }
    }
}


impl Display for SharedContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // mutability
        match &self.reference_mutability {
            Some(PointerReferenceMutability::Mutable) => write!(f, "'mut ")?,
            Some(PointerReferenceMutability::Immutable) => write!(f, "'")?,
            None => {}
        }

        write!(f, "{}", &self.inner.borrow())
    }
}

/// Two references are identical if they point to the same data
impl Identity for SharedContainer {
    fn identical(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for SharedContainer {}

/// PartialEq corresponds to pointer equality / identity for `Reference`.
impl PartialEq for SharedContainer {
    fn eq(&self, other: &Self) -> bool {
        self.identical(other)
    }
}

impl StructuralEq for SharedContainerValueOrType {
    fn structural_eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (SharedContainerValueOrType::Type(a), SharedContainerValueOrType::Type(b)) => {
                a.type_value.structural_eq(&b.type_value)
            }
            (
                SharedContainerValueOrType::Value(a),
                SharedContainerValueOrType::Value(b),
            ) => a
                .value_container
                .structural_eq(&b.value_container),
            _ => false,
        }
    }
}

impl StructuralEq for SharedContainer {
    fn structural_eq(&self, other: &Self) -> bool {
        self.value().structural_eq(&other.value())
    }
}

impl ValueEq for SharedContainerValueOrType {
    fn value_eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (SharedContainerValueOrType::Type(a), SharedContainerValueOrType::Type(b)) => {
                // FIXME #281: Implement value_eq for type and use here instead (recursive)
                a.type_value.structural_eq(&b.type_value)
            }
            (
                SharedContainerValueOrType::Value(a),
                SharedContainerValueOrType::Value(b),
            ) => a
                .value_container
                .value_eq(&b.value_container),
            _ => false,
        }
    }
}

impl ValueEq for SharedContainer {
    fn value_eq(&self, other: &Self) -> bool {
        self.value().value_eq(&other.value())
    }
}

impl Hash for SharedContainer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Rc::as_ptr(&self.inner);
        ptr.hash(state); // hash the address
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
                write!(
                    f,
                    "Cannot create mutable shared value for immutable value"
                )
            }
        }
    }
}

impl SharedContainer {
    pub fn pointer_address(&self) -> PointerAddress {
        self.inner.borrow().pointer_address()
    }

    pub fn value(&self) -> Ref<'_, SharedContainerValueOrType> {
        Ref::map(self.inner.borrow(), |inner| inner.value())
    }

    pub fn value_mut(&self) -> RefMut<'_, SharedContainerValueOrType> {
        RefMut::map(self.inner.borrow_mut(), |inner| inner.value_mut())
    }

    /// Gets the mutability of the shared value.
    /// TypeReferences are always immutable.
    pub(crate) fn mutability(&self) -> SharedContainerMutability {
        self.value().mutability()
    }

    /// Checks if the reference can be mutated by the current endpoint.
    pub(crate) fn can_mutate(&self) -> bool {
        match &self.reference_mutability {
            // is &mut reference to a pointer
            Some(PointerReferenceMutability::Mutable) => true,
            // is owned pointer, check if the shared container is mutable
            None => self.mutability() == SharedContainerMutability::Mutable,
            _ => false,
        }
    }

    /// Moves an owned shared container by changing the pointer to a ReferencedPointerAddress
    /// The original owned SharedContainer is dropped
    pub(crate) fn move_to_remote(
        self,
        remote_address: ReferencedPointerAddress,
    ) -> Result<(), ()> {
        if !self.is_owned() {
            return Err(());
        }

        self.inner.borrow_mut().change_to_referenced(ReferencedPointer::new(remote_address));

        Ok(())
    }

    pub fn try_derive_mutable_reference(&self) -> Result<Self, ()> {
        if !self.can_mutate() {
            return Err(());
        }

        Ok(SharedContainer {
            inner: self.inner.clone(),
            reference_mutability: Some(PointerReferenceMutability::Mutable),
        })
    }

    /// Clones the shared container as a mutable reference if possible, otherwise as an immutable reference
    pub fn derive_with_max_mutability(&self) -> Self {
        self.try_derive_mutable_reference()
            .unwrap_or_else(|_| self.derive_reference())
    }

    /// Returns the shared container if it is owned (not a reference), otherwise returns an error.
    pub fn assert_owned(&self) -> Result<(), ()> {
        self.is_owned().then_some(()).ok_or(())
    }

    /// Returns whether the shared container is owned
    pub fn is_owned(&self) -> bool {
        let is_owned = self.reference_mutability.is_none();
        if is_owned {
            if !matches!(self.pointer_address(), PointerAddress::Owned(_)) {
                unreachable!()
            }
            true
        } else {
            false
        }
    }

    /// Returns the local pointer address if this is an owned pointer, otherwise returns None.
    pub fn try_get_owned_local_address(&self) -> Option<[u8; 5]> {
        self.assert_owned().ok()?;

        match self.pointer_address() {
            PointerAddress::Owned(OwnedPointerAddress { address }) => {
                Some(address)
            }
            _ => None,
        }
    }

    pub fn derive_reference(&self) -> Self {
        SharedContainer {
            inner: self.inner.clone(),
            reference_mutability: Some(PointerReferenceMutability::Immutable),
        }
    }

    /// Checks if the shared container is mutable.
    /// TypeReferences are always immutable.
    pub fn is_mutable(&self) -> bool {
        match &*self.value() {
            SharedContainerValueOrType::Type(_) => false, // type references are always immutable
            SharedContainerValueOrType::Value(vr) => {
                vr.mutability == SharedContainerMutability::Mutable
            }
        }
    }

    /// Creates a new shared owned value containing the given value container
    pub fn try_boxed_owned(
        value_container: ValueContainer,
        allowed_type: Option<TypeDefinition>,
        pointer: OwnedPointer,
        mutability: SharedContainerMutability,
    ) -> Result<Self, SharedValueCreationError> {
        let allowed_type =
            allowed_type.unwrap_or_else(|| value_container.allowed_type());

        // TODO #286: make sure allowed type is superset of reference's allowed type
        Ok(SharedContainer {
            inner: Rc::new(RefCell::new(SharedContainerInner::Owned(OwnedSharedContainerInner {
                value_or_type: SharedContainerValueOrType::Value(SharedValueContainer::new(
                    value_container,
                    allowed_type,
                    mutability,
                )),
                pointer,
            }))),
            reference_mutability: None,
        })
    }

    /// Creates a new shared ref value containing the given value container
    pub fn try_boxed_ref(
        value_container: ValueContainer,
        allowed_type: Option<TypeDefinition>,
        pointer: ReferencedPointer,
        mutability: SharedContainerMutability,
        reference_mutability: PointerReferenceMutability,
    ) -> Result<Self, SharedValueCreationError> {
        let allowed_type =
            allowed_type.unwrap_or_else(|| value_container.allowed_type());

        // TODO #286: make sure allowed type is superset of reference's allowed type
        Ok(SharedContainer {
            inner: Rc::new(RefCell::new(SharedContainerInner::Referenced(ReferencedSharedContainerInner {
                value_or_type: SharedContainerValueOrType::Value(SharedValueContainer::new(
                    value_container,
                    allowed_type,
                    mutability,
                )),
                pointer,
            }))),
            reference_mutability: Some(reference_mutability),
        })
    }

    /// Create a new shared type container
    /// The pointer must be an owned pointer, since we create a new shared value
    pub fn new_from_type(
        type_value: Type,
        pointer: OwnedPointer,
        maybe_nominal_type_declaration: Option<NominalTypeDeclaration>,
    ) -> Self {
        let type_reference = SharedTypeContainer::new(
            type_value,
            maybe_nominal_type_declaration,
        );
        SharedContainer {
            inner: Rc::new(RefCell::new(SharedContainerInner::Owned(OwnedSharedContainerInner {
                value_or_type: SharedContainerValueOrType::Type(type_reference),
                pointer,
            }))),
            reference_mutability: None,
        }
    }

    /// The pointer must be an owned pointer, since we create a new shared value
    pub fn boxed_owned_mut(
        value_container: impl Into<ValueContainer>,
        pointer: OwnedPointer,
    ) -> Self {
        SharedContainer::boxed_owned(
            value_container,
            pointer,
            SharedContainerMutability::Mutable,
        )
    }

    /// The pointer must be an owned pointer, since we create a new shared value
    pub fn boxed_owned_immut(
        value_container: impl Into<ValueContainer>,
        pointer: OwnedPointer,
    ) -> Self {
        SharedContainer::boxed_owned(
            value_container,
            pointer,
            SharedContainerMutability::Immutable,
        )
    }

    /// The pointer must be an owned pointer, since we create a new shared value
    pub fn boxed_owned(
        value_container: impl Into<ValueContainer>,
        pointer: OwnedPointer,
        mutability: SharedContainerMutability
    ) -> Self {
        SharedContainer::try_boxed_owned(
            value_container.into(),
            None,
            pointer,
            mutability
        )
            .unwrap() // always Ok, since we dont provide an allowed type that could mismatch
    }

    /// Creates an owned pointer (no ref), but for an internal pointer address, not an OwnedPointer
    pub(crate) fn boxed_owned_with_internal_pointer(
        value_container: impl Into<ValueContainer>,
        internal_pointer_address: [u8; 3],
    ) -> Self {
        let value_container = value_container.into();
        let allowed_type = value_container.allowed_type();

        SharedContainer {
            inner: Rc::new(RefCell::new(SharedContainerInner::Referenced(ReferencedSharedContainerInner {
                value_or_type: SharedContainerValueOrType::Value(SharedValueContainer::new(
                    value_container,
                    allowed_type,
                    SharedContainerMutability::Immutable,
                )),
                pointer: ReferencedPointer::new(
                    ReferencedPointerAddress::Internal(
                        internal_pointer_address,
                    ),
                )
            }))),
            reference_mutability: None,
        }
    }

    pub fn boxed_ref(
        value_container: impl Into<ValueContainer>,
        pointer: ReferencedPointer,
    ) -> Self {
        SharedContainer::try_boxed_ref(
            value_container.into(),
            None,
            pointer,
            SharedContainerMutability::Immutable,
            PointerReferenceMutability::Immutable,
        )
        .unwrap() // always Ok, since we dont provide an allowed type that could mismatch
    }

    pub fn boxed_mut_ref(
        value_container: impl Into<ValueContainer>,
        pointer: ReferencedPointer,
    ) -> Self {
        SharedContainer::try_boxed_ref(
            value_container.into(),
            None,
            pointer,
            SharedContainerMutability::Mutable,
            PointerReferenceMutability::Mutable,
        )
        .unwrap() // always Ok, since we don't provide an allowed type that could mismatch
    }

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

    pub fn allowed_type(&self) -> TypeDefinition {
        match &*self.value() {
            SharedContainerValueOrType::Value(vr) => vr.allowed_type.clone(),
            SharedContainerValueOrType::Type(_) => core::todo!("#293 type Type"),
        }
    }

    pub fn actual_type(&self) -> TypeDefinition {
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

impl Apply for SharedContainer {
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
            SharedContainer::boxed_owned_mut(value, OwnedPointer::NULL);
        assert_eq!(reference.mutability(), SharedContainerMutability::Mutable);
    }

    #[test]
    fn property() {
        let mut map = Map::default();
        map.set("name", ValueContainer::from("Jonas"));
        map.set("age", ValueContainer::from(30));
        let reference = SharedContainer::boxed_owned_immut(
            ValueContainer::from(map),
            OwnedPointer::NULL,
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
        let reference = SharedContainer::boxed_owned_immut(
            ValueContainer::from(struct_val),
            OwnedPointer::NULL,
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
        let reference = SharedContainer::boxed_owned_immut(
            ValueContainer::from(list),
            OwnedPointer::NULL,
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

        let text_ref = SharedContainer::boxed_owned_immut(
            ValueContainer::from("hello"),
            OwnedPointer::NULL,
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
            SharedContainer::boxed_owned_immut(value, OwnedPointer::NULL);
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
            SharedContainer::boxed_owned_immut(value, OwnedPointer::NULL)
        );
    }

    #[test]
    fn reference_value_equality() {
        let value = 42;
        let reference1 = ValueContainer::Shared(SharedContainer::boxed_owned_immut(
            value,
            OwnedPointer::NULL,
        ));
        let reference2 = ValueContainer::Shared(SharedContainer::boxed_owned_immut(
            value,
            OwnedPointer::NULL,
        ));

        // different references should not be equal a.k.a. identical
        assert_ne!(reference1, reference2);
        // but their current resolved values should be equal
        assert_value_eq!(reference1, ValueContainer::from(value));
    }

    #[test]
    fn reference_structural_equality() {
        let reference1 = SharedContainer::boxed_owned_immut(42.0, OwnedPointer::NULL);
        let reference2 = SharedContainer::boxed_owned_immut(42, OwnedPointer::NULL);

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
            ValueContainer::Shared(SharedContainer::boxed_owned_immut(
                Map::default(),
                OwnedPointer::NULL,
            )),
        );

        // construct map_a as a value first
        let map_a_original_ref = ValueContainer::Shared(
            SharedContainer::boxed_owned_immut(map_a, OwnedPointer::NULL),
        );

        // create map_b as a reference
        let map_b_ref = SharedContainer::try_boxed_owned(
            Map::default().into(),
            None,
            OwnedPointer::NULL,
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
