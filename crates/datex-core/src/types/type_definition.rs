use crate::{
    prelude::*,
    shared_values::pointer_address::PointerAddress,
    traits::structural_eq::StructuralEq,
    types::{
        collection_type_definition::CollectionTypeDefinition,
        literal_type_definition::LiteralTypeDefinition,
        shared_container_containing_type::SharedContainerContainingType,
        r#type::Type,
        type_definition_with_metadata::{TypeDefinitionWithMetadata, TypeMetadata},
    },
    values::core_values::callable::CallableSignature,
};
use core::{fmt::Display, hash::Hash, ops::Deref, prelude::rust_2024::*};
use crate::runtime::memory::Memory;
use crate::types::shared_container_containing_nominal_type::SharedContainerContainingNominalType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDefinition {
    /// e.g. 1, "example"
    Literal(LiteralTypeDefinition),

    List(Vec<Type>), // e.g. [&mut integer, text, boolean]
    Map(Vec<(Type, Type)>),
    Range((Box<Type>, Box<Type>)),

    // TODO #371: Rename to generic?
    /// e.g. [integer], [integer; 5], Map<string, integer>
    Collection(CollectionTypeDefinition),

    /// typealias A = B
    Shared(SharedContainerContainingType), // integer

    /// needed for nested types with multiple reference layers (e.g. 'mut 'mut shared X)
    Nested(Box<Type>),

    // FIXME DO we still need Type(Type) here?
    // Hopefully no, as nominal type definitions referring other types need
    // shared container containing types in the chain
    //
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

    /// meta type for a type
    Type,
}

impl Hash for TypeDefinition {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            TypeDefinition::Collection(value) => {
                value.hash(state);
            }
            TypeDefinition::Literal(value) => {
                value.hash(state);
            }
            TypeDefinition::Map(entries) => {
                for (key, value) in entries {
                    key.hash(state);
                    value.hash(state);
                }
            }
            TypeDefinition::List(elements) => {
                for element in elements {
                    element.hash(state);
                }
            }
            TypeDefinition::Range((start, end)) => {
                start.hash(state);
                end.hash(state);
            }
            TypeDefinition::Shared(reference) => {
                reference.hash(state);
            }

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
            TypeDefinition::Nested(ty) => {
                ty.hash(state);
            }
            TypeDefinition::Type => {
                // no fields to hash
                // TODO: can we do this?
                0.hash(state);
            }
        }
    }
}

impl Display for TypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TypeDefinition::Collection(value) => {
                core::write!(f, "{}", value)
            }
            TypeDefinition::Map(entries) => {
                let entries_str: Vec<String> = entries
                    .iter()
                    .map(|(key, value)| format!("{}: {}", key, value))
                    .collect();
                core::write!(f, "{{{}}}", entries_str.join(", "))
            }
            TypeDefinition::List(elements) => {
                let elements_str: Vec<String> =
                    elements.iter().map(|e| e.to_string()).collect();
                core::write!(f, "[{}]", elements_str.join(", "))
            }
            TypeDefinition::Range((start, end)) => {
                core::write!(f, "{}..{}", start, end)
            }

            TypeDefinition::Literal(value) => {
                core::write!(f, "{}", value)
            }
            TypeDefinition::Shared(reference) => {
                core::write!(f, "{}", reference.deref())
            }
            TypeDefinition::ImplType(ty, impls) => {
                core::write!(f, "{}", ty)?;
                for marker in impls {
                    core::write!(f, " + {}", marker)?;
                }
                Ok(())
            }

            TypeDefinition::Union(types) => {
                let types_str: Vec<String> =
                    types.iter().map(|t| t.to_string()).collect();
                core::write!(f, "({})", types_str.join(" | "))
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
            TypeDefinition::Nested(ty) => {
                core::write!(f, "{}", ty)
            }
            TypeDefinition::Type => {
                core::write!(f, "Type")
            }
        }
    }
}

impl StructuralEq for TypeDefinition {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypeDefinition::Literal(a), TypeDefinition::Literal(b)) => {
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
    /// Calls the provided callback with a reference to the recursively collapsed inner [TypeDefinition] value
    pub fn with_collapsed<R>(&self, f: impl FnOnce(&TypeDefinition) -> R) -> R {
        match self {
            TypeDefinition::Shared(reference) =>
            // collapse shared container to inner Type
            {
                reference.with_collapsed_type_value(|ty| {
                    // collapse Type definition to inner StructuralTypeDefinition
                    ty.with_collapsed_type_definition(f)
                })
            }
            _ => f(self),
        }
    }

    /// Creates a new literal type.
    pub fn literal(literal_type: impl Into<LiteralTypeDefinition>) -> Self {
        TypeDefinition::Literal(literal_type.into())
    }

    /// Creates a new list type.
    pub fn list(element_types: Vec<Type>) -> Self {
        TypeDefinition::List(element_types)
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

    /// Creates a new shared type.
    pub fn shared(reference: SharedContainerContainingType) -> Self {
        TypeDefinition::Shared(reference)
    }

    /// Creates a new callable type.
    pub fn callable(signature: CallableSignature) -> Self {
        TypeDefinition::Callable(signature)
    }

    /// Creates a new type with impls.
    pub fn impl_type(ty: impl Into<Type>, impls: Vec<PointerAddress>) -> Self {
        TypeDefinition::ImplType(Box::new(ty.into()), impls)
    }
}

impl TypeDefinition {
    /// Map a type definition (e.g. 42u8) to it's upper level base type (e.g. integer)
    /// integer/u8 -> integer
    /// integer -> integer
    /// 42u8 -> integer
    /// 42 -> integer
    /// User/variant -> User
    pub fn base_core_lib_type(&self, memory: &Memory) -> SharedContainerContainingNominalType {
        match &self {
            TypeDefinition::Literal(value) => {
                memory.get_core_type_reference(value.get_core_lib_type_pointer_id())
            }
            TypeDefinition::Union(_) => {
                core::todo!("#322 handle union base type"); // generic type base type / type
            }
            TypeDefinition::Shared(reference) => reference
                .with_collapsed_type_value(|ty| ty.base_core_lib_type(memory)),
            _ => core::panic!("Unhandled type definition for base type"),
        }
    }
}

impl From<TypeDefinition> for TypeDefinitionWithMetadata {
    fn from(structural_definition: TypeDefinition) -> Self {
        TypeDefinitionWithMetadata {
            definition: structural_definition,
            metadata: TypeMetadata::default(),
        }
    }
}

impl From<LiteralTypeDefinition> for TypeDefinitionWithMetadata {
    fn from(literal_definition: LiteralTypeDefinition) -> Self {
        TypeDefinitionWithMetadata {
            definition: literal_definition.into(),
            metadata: TypeMetadata::default(),
        }
    }
}
