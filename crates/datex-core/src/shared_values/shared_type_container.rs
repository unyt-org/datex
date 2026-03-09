use core::{
    cell::RefCell,
    fmt::{Display, Formatter},
    result::Result,
};
use serde::{Deserialize, Serialize};

use crate::{
    libs::core::CoreLibPointerId,
    prelude::*,
    runtime::execution::ExecutionError,
    shared_values::{pointer::Pointer, pointer_address::PointerAddress},
    traits::apply::Apply,
    types::{
        definition::TypeDefinition,
        structural_type_definition::StructuralTypeDefinition,
    },
    values::{
        core_values::r#type::{Type, TypeMetadata},
        value_container::ValueContainer,
    },
};
use core::option::Option;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NominalTypeDeclaration {
    pub name: String,
    pub variant: Option<String>,
}
impl From<String> for NominalTypeDeclaration {
    fn from(name_and_variant: String) -> Self {
        NominalTypeDeclaration::from(name_and_variant.as_str())
    }
}
impl From<&str> for NominalTypeDeclaration {
    fn from(name_and_variant: &str) -> Self {
        let mut parts = name_and_variant.split('/');
        NominalTypeDeclaration {
            name: unsafe {
                // rationale: at least one part always exists
                parts.next().unwrap_unchecked().to_string()
            },
            variant: parts.next().map(|s| s.to_string()),
        }
    }
}

impl Display for NominalTypeDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if let Some(variant) = &self.variant {
            core::write!(f, "{}/{}", self.name, variant)
        } else {
            core::write!(f, "{}", self.name)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SharedTypeContainer {
    /// the value that contains the type declaration
    pub type_value: Type,
    /// optional nominal type declaration
    pub nominal_type_declaration: Option<NominalTypeDeclaration>,
    pointer: Pointer,
}

impl SharedTypeContainer {
    
    pub fn new(
        type_value: Type,
        nominal_type_declaration: Option<NominalTypeDeclaration>,
        pointer: Pointer,
    ) -> Self {
        SharedTypeContainer {
            type_value,
            nominal_type_declaration,
            pointer,
        }
    }
    
    pub fn nominal<T>(
        type_value: Type,
        nominal_type_declaration: T,
        pointer: Pointer,
    ) -> Self
    where
        T: Into<NominalTypeDeclaration>,
    {
        SharedTypeContainer {
            type_value,
            nominal_type_declaration: Some(nominal_type_declaration.into()),
            pointer,
        }
    }
    pub fn anonymous(type_value: Type, pointer: Pointer) -> Self {
        SharedTypeContainer {
            type_value,
            nominal_type_declaration: None,
            pointer,
        }
    }
    pub fn as_ref_cell(self) -> Rc<RefCell<SharedTypeContainer>> {
        Rc::new(RefCell::new(self))
    }

    /// Convert this TypeReference into a Type representing a reference to the underlying type
    pub fn as_type(self) -> Type {
        Type::shared_reference(self.as_ref_cell(), TypeMetadata::default())
    }

    pub fn collapse_reference_chain(&self) -> SharedTypeContainer {
        match &self.type_value.type_definition {
            TypeDefinition::SharedReference(reference) => {
                // If this is a reference type, resolve it to its current reference
                reference.borrow().collapse_reference_chain()
            }
            _ => {
                // If this is not a reference type, return it directly
                self.clone()
            }
        }
    }
    
    pub fn pointer(&self) -> &Pointer {
        &self.pointer
    }
}

impl SharedTypeContainer {
    pub fn structural_type_definition(
        &self,
    ) -> Option<&StructuralTypeDefinition> {
        self.type_value.structural_type_definition()
    }

    pub fn base_type(&self) -> Option<Rc<RefCell<SharedTypeContainer>>> {
        self.type_value.base_type_reference()
    }

    pub fn matches_reference(
        &self,
        _other: Rc<RefCell<SharedTypeContainer>>,
    ) -> bool {
        core::todo!("#300 implement type matching");
    }

    pub fn matches_type(&self, other: &Type) -> bool {
        if let Some(base) = other.base_type_reference() {
            return *self == *base.borrow();
        }

        core::todo!("#301 implement type matching");
    }
}

impl Apply for SharedTypeContainer {
    fn apply(
        &self,
        _args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        core::todo!("#302 Undescribed by author.")
    }

    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        // TODO #303: ensure that we can guarantee that pointer_address is always Some here
        let core_lib_id = CoreLibPointerId::try_from(&self.pointer.address());
        if let Ok(core_lib_id) = core_lib_id {
            match core_lib_id {
                CoreLibPointerId::Integer(None) => arg
                    .to_value()
                    .borrow()
                    .cast_to_integer()
                    .map(|i| Some(ValueContainer::from(i)))
                    .ok_or_else(|| ExecutionError::InvalidTypeCast),
                CoreLibPointerId::Integer(Some(variant)) => arg
                    .to_value()
                    .borrow()
                    .cast_to_typed_integer(variant)
                    .map(|i| Some(ValueContainer::from(i)))
                    .ok_or_else(|| ExecutionError::InvalidTypeCast),
                CoreLibPointerId::Decimal(None) => arg
                    .to_value()
                    .borrow()
                    .cast_to_decimal()
                    .map(|d| Some(ValueContainer::from(d)))
                    .ok_or_else(|| ExecutionError::InvalidTypeCast),
                CoreLibPointerId::Decimal(Some(variant)) => arg
                    .to_value()
                    .borrow()
                    .cast_to_typed_decimal(variant)
                    .map(|d| Some(ValueContainer::from(d)))
                    .ok_or_else(|| ExecutionError::InvalidTypeCast),
                _ => core::todo!("#304 Undescribed by author."),
            }
        } else {
            core::todo!("#305 Undescribed by author.")
        }
    }
}

impl Display for SharedTypeContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if let Some(nominal) = &self.nominal_type_declaration {
            // special exception: for Unit, display "()"
            if self.pointer.address()
                == PointerAddress::from(CoreLibPointerId::Unit)
            {
                return core::write!(f, "()");
            }
            core::write!(f, "{}", nominal)
        } else {
            core::write!(f, "{}", self.type_value)
        }
    }
}
