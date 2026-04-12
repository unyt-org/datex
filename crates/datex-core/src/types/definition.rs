use crate::{
    traits::structural_eq::StructuralEq,
    types::{
        collection_type_definition::CollectionTypeDefinition,
        structural_type_definition::StructuralTypeDefinition,
    },
    values::core_values::{callable::CallableSignature, r#type::Type},
};
use core::{fmt::Display, hash::Hash, prelude::rust_2024::*};

use crate::{
    prelude::*, shared_values::pointer_address::PointerAddress,
    values::core_values::r#type::TypeMetadata,
};
use crate::shared_values::shared_containers::SharedContainerContainingType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDefinition {
    /// { x: integer, y: text }
    Structural(StructuralTypeDefinition),

    // TODO #371: Rename to generic?
    /// e.g. [integer], [integer; 5], Map<string, integer>
    Collection(CollectionTypeDefinition),

    /// type A = B
    SharedReference(SharedContainerContainingType), // integer

    /// type, used for nested types with references (e.g. &mut & x)
    Type(Box<Type>),

    /// a callable type definition (signature)
    Callable(CallableSignature),

    /// innerType + Marker1 + Marker2
    /// A special type that behaves like `innerType` but is marked with additional
    /// pointer addresses that represent meta information about the type.
    /// The type is treated as equivalent to `innerType` for most operations,
    /// but the impl markers can be used to enforce additional constraints during
    /// type checking or runtime behavior.
    ImplType(Box<Type>, Vec<PointerAddress>),

    /// NOTE: all the types below can never exist as actual types of a runtime value - they are only
    /// relevant for type space definitions and type checking.

    /// A & B & C
    Intersection(Vec<Type>),

    /// A | B | C
    Union(Vec<Type>),

    /// () - e.g. if a function has no return type
    Unit,

    /// never type
    Never,

    /// unknown type
    Unknown,
}

impl Hash for TypeDefinition {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            TypeDefinition::Collection(value) => {
                value.hash(state);
            }
            TypeDefinition::Structural(value) => {
                value.hash(state);
            }
            TypeDefinition::SharedReference(reference) => {
                reference.borrow().hash(state);
            }
            TypeDefinition::Type(value) => {
                value.hash(state);
            }

            TypeDefinition::Unit => 0_u8.hash(state),
            TypeDefinition::Unknown => 1_u8.hash(state),
            TypeDefinition::Never => 2_u8.hash(state),

            TypeDefinition::Union(types) => {
                for ty in types {
                    ty.hash(state);
                }
            }
            TypeDefinition::Intersection(types) => {
                for ty in types {
                    ty.hash(state);
                }
            }
            TypeDefinition::Callable(callable) => {
                callable.kind.hash(state);
                for (name, ty) in callable.parameter_types.iter() {
                    name.hash(state);
                    ty.hash(state);
                }
                callable.rest_parameter_type.hash(state);
                callable.return_type.hash(state);
                callable.yeet_type.hash(state);
            }
            TypeDefinition::ImplType(ty, impls) => {
                ty.hash(state);
                for marker in impls {
                    marker.hash(state);
                }
            }
        }
    }
}

impl Display for TypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TypeDefinition::Collection(value) => core::write!(f, "{}", value),
            TypeDefinition::Structural(value) => core::write!(f, "{}", value),
            TypeDefinition::SharedReference(reference) => {
                core::write!(f, "{}", reference.borrow())
            }
            TypeDefinition::Type(ty) => core::write!(f, "{}", ty),
            TypeDefinition::Unit => core::write!(f, "()"),
            TypeDefinition::Unknown => core::write!(f, "unknown"),
            TypeDefinition::Never => core::write!(f, "never"),
            TypeDefinition::ImplType(ty, impls) => {
                core::write!(f, "{}", ty)?;
                for marker in impls {
                    core::write!(f, " + {}", marker)?;
                }
                Ok(())
            }

            TypeDefinition::Union(types) => {
                let is_level_zero = types.iter().all(|t| {
                    core::matches!(
                        t.type_definition,
                        TypeDefinition::Structural(_)
                            | TypeDefinition::SharedReference(_)
                    )
                });
                let types_str: Vec<String> =
                    types.iter().map(|t| t.to_string()).collect();
                if is_level_zero {
                    core::write!(f, "{}", types_str.join(" | "))
                } else {
                    core::write!(f, "({})", types_str.join(" | "))
                }
            }
            TypeDefinition::Intersection(types) => {
                let types_str: Vec<String> =
                    types.iter().map(|t| t.to_string()).collect();
                core::write!(f, "({})", types_str.join(" & "))
            }
            TypeDefinition::Callable(callable) => {
                let mut params_code: Vec<String> = callable
                    .parameter_types
                    .iter()
                    .map(|(param_name, param_type)| match param_name {
                        Some(name) => format!("{}: {}", name, param_type),
                        None => format!("{}", param_type),
                    })
                    .collect();
                // handle rest parameter
                if let Some((param_name, param_type)) =
                    &callable.rest_parameter_type
                {
                    params_code.push(match param_name {
                        Some(name) => format!("...{}: {}", name, param_type),
                        None => format!("...{}", param_type),
                    });
                }

                let return_type_code = match &callable.return_type {
                    Some(return_type) => format!(" -> {}", return_type),
                    None => " -> ()".to_string(),
                };

                let yeet_type_code = match &callable.yeet_type {
                    Some(yeet_type) => format!(" yeets {}", yeet_type),
                    None => "".to_string(),
                };

                core::write!(
                    f,
                    "{} ({}){}{}",
                    callable.kind,
                    params_code.join(", "),
                    return_type_code,
                    yeet_type_code
                )
            }
        }
    }
}

impl StructuralEq for TypeDefinition {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypeDefinition::Structural(a), TypeDefinition::Structural(b)) => {
                a.structural_eq(b)
            }
            (TypeDefinition::Union(a), TypeDefinition::Union(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (item_a, item_b) in a.iter().zip(b.iter()) {
                    if !item_a.structural_eq(item_b) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}

impl TypeDefinition {
    /// Creates a new structural type.
    pub fn structural(
        structural_type: impl Into<StructuralTypeDefinition>,
    ) -> Self {
        TypeDefinition::Structural(structural_type.into())
    }

    /// Creates a new structural list type.
    pub fn list(element_types: Vec<Type>) -> Self {
        TypeDefinition::Structural(StructuralTypeDefinition::List(
            element_types,
        ))
    }

    /// Creates a new union type.
    pub fn union<T>(types: Vec<T>) -> Self
    where
        T: Into<Type>,
    {
        let types = types.into_iter().map(|t| t.into()).collect();
        TypeDefinition::Union(types)
    }

    /// Creates a new intersection type.
    pub fn intersection<T>(types: Vec<T>) -> Self
    where
        T: Into<Type>,
    {
        let types = types.into_iter().map(|t| t.into()).collect();
        TypeDefinition::Intersection(types)
    }

    /// Creates a new reference type.
    pub fn shared_reference(
        reference: SharedContainerContainingType,
    ) -> Self {
        TypeDefinition::SharedReference(reference)
    }

    /// Creates a new callable type.
    pub fn callable(signature: CallableSignature) -> Self {
        TypeDefinition::Callable(signature)
    }

    /// Creates a new type with impls.
    pub fn impl_type(ty: impl Into<Type>, impls: Vec<PointerAddress>) -> Self {
        TypeDefinition::ImplType(Box::new(ty.into()), impls)
    }

    pub fn into_type(self, prefix: TypeMetadata) -> Type {
        Type {
            type_definition: self,
            base_type: None,
            metadata: prefix,
        }
    }
}

impl From<TypeDefinition> for Type {
    fn from(type_definition: TypeDefinition) -> Self {
        Type {
            type_definition,
            base_type: None,
            metadata: TypeMetadata::default(),
        }
    }
}
