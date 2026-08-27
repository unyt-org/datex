use proc_macro2::Ident;
use syn::{Generics, Type};

#[derive(Debug, PartialEq, Eq)]
enum TypeKind {
    Entity,
    Structural,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Namespace {
    None,
    Module,
    Named(String),
}

#[derive(Debug, PartialEq)]
enum SerdeMode {
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
    force_datex_core_namespace: bool,

    /// Optional override for the exported name of the type. Defaults to the Rust struct or enum name.
    datex_name: Option<String>,

    /// If the decorated struct or enum should not be deserializable from a Datex value.
    no_deserialize: bool,

    /// When set to true, the struct/enum will map to a DATEX structural type instead of a nominal entity type.
    type_kind: TypeKind,

    export_namespace: Namespace,

    /// If the decorated struct or enum should be exported to the Datex registry.
    /// `#[datex(export)]`
    export: bool,
    namespace: Option<String>,
    docs: Option<String>,
}

#[derive(Debug, PartialEq)]
/// Represents a field in a struct or enum variant, along with its type and attributes.
pub struct Field {
    ty: Type,
    attributes: FieldAttributes,
}

#[derive(Debug, PartialEq)]
/// General attributes that can be applied to any field.
pub struct FieldAttributes {
    serde_mode: SerdeMode,
}

#[derive(Debug, PartialEq)]
/// Attributes specific to named fields in structs or enum variants.
pub struct NamedFieldAttributes {
    /// If true, this field will be invisible to DATEX.
    skip: bool,
    /// TODO:
    default: bool,
    /// An optional rename for the field used for the DATEX representation. If not provided, the rust field name will be used.
    rename: Option<String>,
}

#[derive(Debug, PartialEq)]
/// Represents a named field in a struct or enum variant.
pub struct NamedField {
    name: String,
    field: Field,
    attributes: NamedFieldAttributes,
}

#[derive(Debug, PartialEq)]
/// Represents the different kinds of fields a struct or enum variant can have.
pub enum Fields {
    Named(Vec<NamedField>),
    Unnamed(Vec<Field>),
    Transparent(Field),
    Unit,
}

#[derive(Debug, PartialEq)]
pub struct EnumVariant {
    name: String,
    fields: Fields,
}

#[derive(Debug, PartialEq)]
pub enum Structure {
    Enum(Vec<EnumVariant>),
    Struct(Fields),
}


#[derive(Debug, PartialEq)]
pub struct StructureData {
    ident: Ident,
    generics: Generics,
    attributes: StructureAttributes,
    structure: Structure,
}

// TODO: derive is_fallible_serialization from fields