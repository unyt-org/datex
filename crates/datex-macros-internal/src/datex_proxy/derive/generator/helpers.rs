use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use crate::datex_proxy::data::{EnumVariant, Fields, Structure};

/// Generates a mapping for the fields of a struct or the variants of an enum, depending on the structure type.
/// The `fields_mapping` function is called with the fields of the struct or the fields of each enum variant, and should return a TokenStream representing the mapping for those fields.
pub fn generate_struct_or_enum_variants_fields_mapping(
    structure: &Structure,
    fields_mapping: impl Fn(&Fields) -> TokenStream,
) -> TokenStream {
    match structure {
        Structure::Struct(fields) => {
            let field_assignments = generate_struct_field_accessors(fields);
            let mapping = fields_mapping(fields);
            quote! {{
                #field_assignments
                #mapping
            }}
        }
        Structure::Enum(variants) => {
            map_enum_variants(
                variants,
                |variant| fields_mapping(&variant.fields)
            )
        }
    }
}

/// Generates a match expression for the given enum type, mapping each variant to a corresponding match arm body.
/// This provides the identifiers of the fields in the match arm, so that they can be used in the body of the match arm.
pub fn map_enum_variants(
    enum_ty: &[EnumVariant],
    generate_match_arm_body: impl Fn(&EnumVariant) -> TokenStream,
) -> TokenStream {
    let arms = enum_ty.iter().map(|variant| {
        let variant_ident = Ident::new(&variant.name, Span::call_site());
        let match_arm_body = generate_match_arm_body(variant);
        let field_idents = variant.fields.normalized_field_idents();
        match &variant.fields {
            Fields::Named(_fields) => {
                quote! {
                    Self::#variant_ident { #(#field_idents),* } => {
                        #match_arm_body
                    }
                }
            }
            Fields::Unnamed(_fields) => {
                quote! {
                    Self::#variant_ident(#(#field_idents),*) => {
                        #match_arm_body
                    }
                }
            }
            Fields::Transparent(_field) => {
                let first_field_ident = field_idents.first().unwrap();
                quote! {
                    Self::#variant_ident(#first_field_ident) => {
                        #match_arm_body
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#variant_ident => {
                        #match_arm_body
                    }
                }
            }
        }
    });

    quote! {
        match self {
            #(#arms),*
        }
    }
}

/// Generates the field accessors for a struct, creating let bindings for each field.
pub fn generate_struct_field_accessors(fields: &Fields) -> TokenStream {
    let field_assignments = fields
        .field_accessors()
        .iter()
        .zip(fields.normalized_field_idents().iter())
        .map(|(accessor, normalized_ident)| {
            quote! {
                let #normalized_ident = &self.#accessor;
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #(#field_assignments)*
    }
}