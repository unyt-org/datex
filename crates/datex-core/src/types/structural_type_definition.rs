use crate::{
    traits::structural_eq::StructuralEq,
    types::{
        collection_type_definition::CollectionTypeDefinition,
        literal_type_definition::LiteralTypeDefinition,
    },
    values::core_values::callable::CallableSignature,
};
use core::{fmt::Display, hash::Hash, prelude::rust_2024::*};
use std::ops::Deref;
use crate::{
    prelude::*, shared_values::pointer_address::PointerAddress,
};
use crate::types::r#type::Type;
use crate::types::shared_container_containing_type::SharedContainerContainingType;
use crate::types::type_definition::{TypeDefinition, TypeMetadata};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuralTypeDefinition {
    /// e.g. 1, "example"
    Literal(LiteralTypeDefinition),

    List(Vec<Type>), // e.g. [&mut integer, text, boolean]
    Map(Vec<(Type, Type)>),
    Range((Box<Type>, Box<Type>)),

    // TODO #371: Rename to generic?
    /// e.g. [integer], [integer; 5], Map<string, integer>
    Collection(CollectionTypeDefinition),

    /// type A = B
    Shared(SharedContainerContainingType), // integer

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

impl Hash for StructuralTypeDefinition {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            StructuralTypeDefinition::Collection(value) => {
                value.hash(state);
            }
            StructuralTypeDefinition::Literal(value) => {
                value.hash(state);
            }
            StructuralTypeDefinition::Shared(reference) => {
                reference.hash(state);
            }
            StructuralTypeDefinition::Type(value) => {
                value.hash(state);
            }

            StructuralTypeDefinition::Unit => 0_u8.hash(state),
            StructuralTypeDefinition::Unknown => 1_u8.hash(state),
            StructuralTypeDefinition::Never => 2_u8.hash(state),

            StructuralTypeDefinition::Union(types) => {
                for ty in types {
                    ty.hash(state);
                }
            }
            StructuralTypeDefinition::Intersection(types) => {
                for ty in types {
                    ty.hash(state);
                }
            }
            StructuralTypeDefinition::Callable(callable) => {
                callable.kind.hash(state);
                for (name, ty) in callable.parameter_types.iter() {
                    name.hash(state);
                    ty.hash(state);
                }
                callable.rest_parameter_type.hash(state);
                callable.return_type.hash(state);
                callable.yeet_type.hash(state);
            }
            StructuralTypeDefinition::ImplType(ty, impls) => {
                ty.hash(state);
                for marker in impls {
                    marker.hash(state);
                }
            }
        }
    }
}

impl Display for StructuralTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StructuralTypeDefinition::Collection(value) => core::write!(f, "{}", value),
            StructuralTypeDefinition::Literal(value) => core::write!(f, "{}", value),
            StructuralTypeDefinition::Shared(reference) => {
                core::write!(f, "{}", reference.deref())
            }
            StructuralTypeDefinition::Type(ty) => core::write!(f, "{}", ty),
            StructuralTypeDefinition::Unit => core::write!(f, "()"),
            StructuralTypeDefinition::Unknown => core::write!(f, "unknown"),
            StructuralTypeDefinition::Never => core::write!(f, "never"),
            StructuralTypeDefinition::ImplType(ty, impls) => {
                core::write!(f, "{}", ty)?;
                for marker in impls {
                    core::write!(f, " + {}", marker)?;
                }
                Ok(())
            }

            StructuralTypeDefinition::Union(types) => {
                let is_level_zero = types.iter().all(|t| {
                    core::matches!(
                        t.definition().structural_definition,
                        StructuralTypeDefinition::Literal(_)
                            | StructuralTypeDefinition::Shared(_)
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
            StructuralTypeDefinition::Intersection(types) => {
                let types_str: Vec<String> =
                    types.iter().map(|t| t.to_string()).collect();
                core::write!(f, "({})", types_str.join(" & "))
            }
            StructuralTypeDefinition::Callable(callable) => {
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

impl StructuralEq for StructuralTypeDefinition {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StructuralTypeDefinition::Literal(a), StructuralTypeDefinition::Literal(b)) => {
                a.structural_eq(b)
            }
            (StructuralTypeDefinition::Union(a), StructuralTypeDefinition::Union(b)) => {
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

impl StructuralTypeDefinition {

    /// Calls the provided callback with a reference to the recursively collapsed inner [StructuralTypeDefinition] value
    pub fn with_collapsed_structural_type_definition<R>(&self, f: impl FnOnce(&StructuralTypeDefinition) -> R) -> R {
        match self {
            StructuralTypeDefinition::Shared(reference) =>
                // collapse shared container to inner Type
                reference.with_collapsed_type_value(|ty| {
                    // collapse Type definition to inner StructuralTypeDefinition
                    ty.definition().structural_definition.with_collapsed_structural_type_definition(f)
                }),
            _ => f(self)
        }
    }

    /// Creates a new literal type.
    pub fn literal(
        literal_type: impl Into<LiteralTypeDefinition>,
    ) -> Self {
        StructuralTypeDefinition::Literal(literal_type.into())
    }

    /// Creates a new list type.
    pub fn list(element_types: Vec<Type>) -> Self {
        StructuralTypeDefinition::List(
            element_types,
        )
    }

    /// Creates a new union type.
    pub fn union<T>(types: Vec<T>) -> Self
    where
        T: Into<Type>,
    {
        let types = types.into_iter().map(|t| t.into()).collect();
        StructuralTypeDefinition::Union(types)
    }

    /// Creates a new intersection type.
    pub fn intersection<T>(types: Vec<T>) -> Self
    where
        T: Into<Type>,
    {
        let types = types.into_iter().map(|t| t.into()).collect();
        StructuralTypeDefinition::Intersection(types)
    }

    /// Creates a new shared type.
    pub fn shared(
        reference: SharedContainerContainingType,
    ) -> Self {
        StructuralTypeDefinition::Shared(reference)
    }

    /// Creates a new callable type.
    pub fn callable(signature: CallableSignature) -> Self {
        StructuralTypeDefinition::Callable(signature)
    }

    /// Creates a new type with impls.
    pub fn impl_type(ty: impl Into<Type>, impls: Vec<PointerAddress>) -> Self {
        StructuralTypeDefinition::ImplType(Box::new(ty.into()), impls)
    }
}

impl From<StructuralTypeDefinition> for TypeDefinition {
    fn from(structural_definition: StructuralTypeDefinition) -> Self {
        TypeDefinition {
            structural_definition,
            metadata: TypeMetadata::default(),
        }
    }
}

impl From<LiteralTypeDefinition> for TypeDefinition {
    fn from(literal_definition: LiteralTypeDefinition) -> Self {
        TypeDefinition {
            structural_definition: literal_definition.into(),
            metadata: TypeMetadata::default(),
        }
    }
}