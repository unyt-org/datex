use crate::datex_proxy::data::{EnumVariant, Field, FieldAttributes, Fields, IndexedField, NamedField, NamedFieldAttributes, FieldMapping, Structure, StructureAttributes, StructureData, TypeKind};
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
            Structure::Struct(parse_struct(data_struct, &attributes))
        }
        Data::Enum(data_enum) => Structure::Enum(parse_enum(data_enum, &attributes)),
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
fn parse_struct(data_struct: DataStruct, structure_attributes: &StructureAttributes) -> Fields {
    parse_fields(data_struct.fields, structure_attributes)
}

/// Parses a rust enum into the internal [Vec<EnumVariant>] representation.
fn parse_enum(data_enum: DataEnum, structure_attributes: &StructureAttributes) -> Vec<EnumVariant> {
    data_enum
        .variants
        .into_iter()
        .map(|variant| {
            let name = variant.ident.to_string();
            let fields = parse_fields(variant.fields, structure_attributes);
            EnumVariant { name, fields }
        })
        .collect()
}

/// Parses the fields of a struct or enum variant into the internal [Fields] representation.
fn parse_fields(
    fields: syn::Fields,
    structure_attributes: &StructureAttributes
) -> Fields {
    if fields.is_empty() {
        return Fields::Unit;
    }

    let has_named_fields = fields.iter().any(|field| field.ident.is_some());

    if has_named_fields {
        Fields::Named(
            fields
                .into_iter()
                .filter_map(|field| parse_named_field_attributes(&field.attrs, structure_attributes.no_deserialize).map(|attributes|(attributes, field)))
                .map(|((attributes, named_attributes), field)| {
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
            .filter_map(|field| parse_field_attributes(&field.attrs, structure_attributes.no_deserialize).map(|attributes| (attributes, field)))
            .map(|(attributes, field)| Field {
                ty: field.ty,
                attributes,
            })
            .collect::<Vec<_>>();
        if fields_list.len() == 1 {
            Fields::Transparent(IndexedField {
                index: 0,
                field: fields_list.remove(0),
            })
        } else {
            Fields::Unnamed(fields_list
                .into_iter()
                .enumerate()
                .map(|(index, field)| IndexedField { index, field }).collect()
            )
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
                    type_kind = TypeKind::Structural { only_structural: false };
                }
                
                Meta::Path(path) if path.is_ident("only_structural") => {
                    type_kind = TypeKind::Structural { only_structural: true };
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
                meta => {
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
    is_no_deserialize: bool
) -> Option<(FieldAttributes, NamedFieldAttributes)> {
    parse_all_field_attributes(attrs, true, is_no_deserialize)
}

fn parse_field_attributes(
    attrs: &[Attribute],
    is_no_deserialize: bool
) -> Option<FieldAttributes> {
    parse_all_field_attributes(attrs, false, is_no_deserialize).map(|(field_attributes, _)| field_attributes)
}

fn parse_all_field_attributes(
    attrs: &[Attribute],
    parse_named_attributes: bool,
    is_no_deserialize: bool,
) -> Option<(FieldAttributes, NamedFieldAttributes)> {

    let mut field_attributes = FieldAttributes {
        field_mapping: FieldMapping::Datex,
    };
    let mut named_field_attributes = NamedFieldAttributes {
        skip_with_default: false,
        rename: None,
    };

    let check_named_attribute_allowed = |name: &str| {
        if !parse_named_attributes {
            panic!(
                "The attribute datex({name}) is only allowed on named fields"
            );
        }
    };

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
                            field_attributes.field_mapping = FieldMapping::Serde;
                        } else if path.is_ident("default") {
                            check_named_attribute_allowed("default");
                            named_field_attributes.skip_with_default = true;
                        } else if path.is_ident("skip") {
                            if named_field_attributes.skip_with_default {
                                panic!("Cannot use both datex(skip) and datex(default)");
                            }
                            // ignore this field, cannot deserialize from DATEX
                            if !is_no_deserialize {
                                panic!("Cannot use datex(skip) on a struct or enum that is not marked with datex(no_deserialize)");
                            }
                            return None;
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
    
    Some((field_attributes, named_field_attributes))
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
