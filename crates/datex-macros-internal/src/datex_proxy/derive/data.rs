use proc_macro2::{Ident, Span};
use syn::{Generics, Type};

#[derive(Debug, PartialEq, Eq)]
pub enum TypeKind {
    Entity,
    Structural,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Namespace {
    None,
    Module,
    Named(String),
}

#[derive(Debug, PartialEq)]
pub enum SerdeMode {
    /// Serde serializable/deserializable fields are not allowed inside the datex proxy value.
    /// Since the generated code will not attempt to serialize any fields with serde,
    /// it will only provide an infallible into method to convert to ValueContainer
    None,
    /// Serde serializable/deserializable fields are allowed inside the datex proxy value.
    /// It is assumed that the serialization might fail, so the generated code will only provide a
    /// try_into method to convert to ValueContainer
    Fallible,
    /// Serde serializable/deserializable fields are allowed inside the datex proxy value.
    /// The user explicitly guarantees that the serialization will not fail, so the generated code will
    /// provide an infallible into method to convert to ValueContainer
    Infallible,
}

/// Top-level attributes for the Datex derive macro
#[derive(Debug, PartialEq)]
pub struct StructureAttributes {
    /// Internally used attribute to indicate that the macro should use the `datex_core` namespace
    /// instead of inferring it. This is required for doctests to work.
    pub force_datex_core_namespace: bool,

    /// Optional override for the exported name of the type. Defaults to the Rust struct or enum name.
    pub datex_name: Option<String>,

    /// If the decorated struct or enum should not be deserializable from a Datex value.
    pub no_deserialize: bool,

    /// When set to true, the struct/enum will map to a DATEX structural type instead of a nominal entity type.
    pub type_kind: TypeKind,

    /// If the decorated struct or enum should be exported to the Datex registry.
    /// `#[datex(export)]`
    pub export: bool,
    pub docs: Option<String>,
}

#[derive(Debug, PartialEq)]
/// Represents a field in a struct or enum variant, along with its type and attributes.
pub struct Field {
    pub ty: Type,
    pub attributes: FieldAttributes,
}

#[derive(Debug, PartialEq)]
/// General attributes that can be applied to any field.
pub struct FieldAttributes {
    pub serde_mode: SerdeMode,
}

#[derive(Debug, PartialEq)]
/// Attributes specific to named fields in structs or enum variants.
pub struct NamedFieldAttributes {
    /// If true, this field will be invisible to DATEX.
    pub skip: bool,
    /// TODO:
    pub default: bool,
    /// An optional rename for the field used for the DATEX representation. If not provided, the rust field name will be used.
    pub rename: Option<String>,
}

#[derive(Debug, PartialEq)]
/// Represents a named field in a struct or enum variant.
pub struct NamedField {
    pub name: String,
    pub field: Field,
    pub attributes: NamedFieldAttributes,
}

#[derive(Debug, PartialEq)]
/// Represents the different kinds of fields a struct or enum variant can have.
pub enum Fields {
    Named(Vec<NamedField>),
    Unnamed(Vec<Field>),
    Transparent(Field),
    Unit,
}

impl Fields {
    pub fn field_idents(&self) -> Vec<Ident> {
        match self {
            Fields::Named(fields) => fields
                .iter()
                .map(|f| Ident::new(&f.name, Span::call_site()))
                .collect(),
            Fields::Unnamed(fields) => fields
                .iter()
                .enumerate()
                .map(|(i, _)| Ident::new(&format!("_{}", i), Span::call_site()))
                .collect(),
            Fields::Transparent(_) => {
                vec![Ident::new("_0", Span::call_site())]
            }
            Fields::Unit => vec![],
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Fields,
    // TODO: enum variant attributes?
}

#[derive(Debug, PartialEq)]
pub enum Structure {
    Enum(Vec<EnumVariant>),
    Struct(Fields),
}

#[derive(Debug, PartialEq)]
pub struct StructureData {
    pub namespace: Vec<String>,
    pub ident: Ident,
    pub generics: Generics,
    pub attributes: StructureAttributes,
    pub structure: Structure,
}

// TODO: derive is_fallible_serialization from fields
