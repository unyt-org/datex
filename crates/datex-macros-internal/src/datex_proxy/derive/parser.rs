use crate::datex_proxy::data::{
    EnumVariant, Field, FieldAttributes, Fields, NamedField,
    NamedFieldAttributes, SerdeMode, Structure, StructureAttributes,
    StructureData, TypeKind,
};
use proc_macro2::Span;
use std::{env, path::PathBuf, str::FromStr};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Meta, Token,
    punctuated::Punctuated,
};

fn get_derive_module_path() -> Vec<String> {
    let root_path = PathBuf::from_str(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()),
    )
    .unwrap();
    let call_site = PathBuf::from(Span::call_site().file());

    if call_site.is_relative() {
        return call_site
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>();
    }
    call_site.strip_prefix(&root_path).unwrap_or_else(|_x| {
        panic!(
            "Failed to get project relative file path. Call site: {:?}, Root path: {:?}",
            call_site, root_path
        )
    }).to_path_buf().iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>()
}

/// Parses the structure data from the provided [DeriveInput].
pub fn parse_structure_data(input: DeriveInput) -> StructureData {
    let ident = input.ident;
    let generics = input.generics;
    let attributes = parse_structure_attributes(&input.attrs);

    // parse the different structure types into the internal representation
    let structure = match input.data {
        Data::Struct(data_struct) => {
            Structure::Struct(parse_struct(data_struct))
        }
        Data::Enum(data_enum) => Structure::Enum(parse_enum(data_enum)),
        Data::Union(_) => {
            unimplemented!(
                "Union types are not supported for DATEX derive macros."
            )
        }
    };

    StructureData {
        namespace: get_derive_module_path(),
        ident,
        generics,
        attributes,
        structure,
    }
}

/// Parses a rust struct into the internal [Fields] representation.
fn parse_struct(data_struct: DataStruct) -> Fields {
    parse_fields(data_struct.fields)
}

/// Parses a rust enum into the internal [Vec<EnumVariant>] representation.
fn parse_enum(data_enum: DataEnum) -> Vec<EnumVariant> {
    data_enum
        .variants
        .into_iter()
        .map(|variant| {
            let name = variant.ident.to_string();
            let fields = parse_fields(variant.fields);
            EnumVariant { name, fields }
        })
        .collect()
}

/// Parses the fields of a struct or enum variant into the internal [Fields] representation.
fn parse_fields(fields: syn::Fields) -> Fields {
    if fields.is_empty() {
        return Fields::Unit;
    }

    let has_named_fields = fields.iter().any(|field| field.ident.is_some());

    if has_named_fields {
        Fields::Named(
            fields
                .into_iter()
                .map(|field| {
                    let (attributes, named_attributes) =
                        parse_named_field_attributes(&field.attrs);
                    NamedField {
                        name: field.ident.unwrap().to_string(),
                        field: Field {
                            ty: field.ty,
                            attributes,
                        },
                        attributes: named_attributes,
                    }
                })
                .collect(),
        )
    } else {
        let mut fields_list = fields
            .into_iter()
            .map(|field| Field {
                ty: field.ty,
                attributes: parse_field_attributes(&field.attrs),
            })
            .collect::<Vec<_>>();
        if fields_list.len() == 1 {
            Fields::Transparent(fields_list.remove(0))
        } else {
            Fields::Unnamed(fields_list)
        }
    }
}

/// Parses the [StructureAttributes] from the provided list of [Attribute]s.
fn parse_structure_attributes(attrs: &[Attribute]) -> StructureAttributes {
    let mut force_datex_core_namespace = false;
    let mut datex_name = None;
    let mut export = false;
    let mut no_deserialize = false;
    let mut type_kind = TypeKind::Entity;

    for attr in attrs {
        if !attr.path().is_ident("datex") {
            continue;
        }
        let Meta::List(meta_list) = &attr.meta else {
            continue;
        };
        let nested = meta_list
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .unwrap_or_else(|error| {
                panic!("Invalid #[datex(...)] attribute: {error}")
            });

        for meta in nested {
            match meta {
                Meta::Path(path)
                    if path.is_ident("_force_datex_core_namespace") =>
                {
                    force_datex_core_namespace = true;
                }
                Meta::Path(path) if path.is_ident("export") => {
                    export = true;
                }

                Meta::Path(path) if path.is_ident("structural") => {
                    type_kind = TypeKind::Structural;
                }

                Meta::Path(path) if path.is_ident("no_deserialize") => {
                    no_deserialize = true;
                }

                Meta::NameValue(name_value)
                    if name_value.path.is_ident("name") =>
                {
                    if datex_name.is_some() {
                        panic!("datex(name = ...) must only be specified once");
                    }
                    datex_name =
                        Some(parse_string_attribute(&name_value, "name"));
                }
                _ => {
                    panic!("Invalid #[datex(...)] attribute: {meta:?}")
                }
            }
        }
    }

    StructureAttributes {
        force_datex_core_namespace,
        no_deserialize,
        datex_name,
        type_kind,
        export,
        docs: parse_doc_comments(attrs),
    }
}

fn parse_named_field_attributes(
    attrs: &[Attribute],
) -> (FieldAttributes, NamedFieldAttributes) {
    parse_all_field_attributes(attrs, true)
}

fn parse_field_attributes(attrs: &[Attribute]) -> FieldAttributes {
    parse_all_field_attributes(attrs, false).0
}

fn parse_all_field_attributes(
    attrs: &[Attribute],
    parse_named_attributes: bool,
) -> (FieldAttributes, NamedFieldAttributes) {
    let mut field_attributes = FieldAttributes {
        serde_mode: SerdeMode::None,
    };
    let mut named_field_attributes = NamedFieldAttributes {
        skip: false,
        default: false,
        rename: None,
    };

    let check_named_attribute_allowed = |name: &str| {
        if !parse_named_attributes {
            panic!(
                "The attribute datex({name}) is only allowed on named fields"
            );
        }
    };

    // find datex(serde) or datex(serde_infallible) attribute
    for attr in attrs {
        if attr.path().is_ident("datex")
            && let Meta::List(meta_list) = &attr.meta
        {
            for nested in meta_list
                .parse_args_with(
                    Punctuated::<Meta, Token![,]>::parse_terminated,
                )
                .unwrap()
            {
                match nested {
                    Meta::Path(path) => {
                        if path.is_ident("serde") {
                            if matches!(
                                field_attributes.serde_mode,
                                SerdeMode::Infallible
                            ) {
                                panic!(
                                    "Cannot use both datex(serde) and datex(serde_infallible)"
                                );
                            }
                            field_attributes.serde_mode = SerdeMode::Fallible;
                        } else if path.is_ident("serde_infallible") {
                            if matches!(
                                field_attributes.serde_mode,
                                SerdeMode::Fallible
                            ) {
                                panic!(
                                    "Cannot use both datex(serde) and datex(serde_infallible)"
                                );
                            }
                            field_attributes.serde_mode = SerdeMode::Infallible;
                        } else if path.is_ident("default") {
                            check_named_attribute_allowed("default");
                            named_field_attributes.default = true;
                        } else if path.is_ident("skip") {
                            check_named_attribute_allowed("skip");
                            named_field_attributes.skip = true;
                        } else {
                            panic!(
                                "Unknown datex field attribute: {}",
                                path.get_ident().unwrap()
                            );
                        }
                    }

                    Meta::NameValue(nv) if nv.path.is_ident("rename") => {
                        let value = match &nv.value {
                            syn::Expr::Lit(expr_lit) => {
                                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                    lit_str.value()
                                } else {
                                    panic!(
                                        "datex(rename = ...) must be a string"
                                    )
                                }
                            }
                            _ => panic!(
                                "datex(rename = ...) must be a string literal"
                            ),
                        };
                        check_named_attribute_allowed("rename");
                        named_field_attributes.rename = Some(value);
                    }

                    _ => {}
                }
            }
        }
    }

    if named_field_attributes.skip && named_field_attributes.default {
        panic!("Cannot use both datex(skip) and datex(default)");
    }

    (field_attributes, named_field_attributes)
}

fn parse_string_attribute(
    name_value: &syn::MetaNameValue,
    attribute_name: &str,
) -> String {
    match &name_value.value {
        syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
            syn::Lit::Str(lit_str) => lit_str.value(),
            _ => {
                panic!("datex({attribute_name} = ...) must be a string literal")
            }
        },
        _ => panic!("datex({attribute_name} = ...) must be a string literal"),
    }
}

fn parse_doc_comments(attrs: &[Attribute]) -> Option<String> {
    let docs = attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path().is_ident("doc") {
                return None;
            }
            let Meta::NameValue(name_value) = &attr.meta else {
                return None;
            };
            let syn::Expr::Lit(expr_lit) = &name_value.value else {
                return None;
            };
            let syn::Lit::Str(lit_str) = &expr_lit.lit else {
                return None;
            };
            Some(lit_str.value().trim_start().to_string())
        })
        .collect::<Vec<_>>();
    if docs.is_empty() {
        None
    } else {
        Some(docs.join("\n"))
    }
}
