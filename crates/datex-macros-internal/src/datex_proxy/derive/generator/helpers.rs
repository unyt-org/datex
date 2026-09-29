use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use crate::datex_proxy::data::{EnumVariant, Fields, Structure};

/// Represents whether the self value is borrowed or moved in the context of generating code for struct or enum variants.
pub enum SelfAccess {
    /// Indicates that the self value is borrowed, and field access should use references.
    Borrowed,
    /// Indicates that the self value is moved, and field access should take ownership of the fields.
    Moved,
}

/// Generates a mapping for the fields of a struct or the variants of an enum, depending on the structure type.
/// The `fields_mapping` function is called with the fields of the struct or the fields of each enum variant, and should return a TokenStream representing the mapping for those fields.
pub fn generate_struct_or_enum_variants_fields_mapping(
    structure: &Structure,
    self_access: SelfAccess,
    fields_mapping: impl Fn(&Fields, Option<&String>) -> TokenStream,
) -> TokenStream {
    match structure {
        Structure::Struct(fields) => {
            let field_assignments = generate_struct_field_accessors(fields, self_access);
            let mapping = fields_mapping(fields, None);
            quote! {{
                #field_assignments
                #mapping
            }}
        }
        Structure::Enum(variants) => {
            map_enum_variants(
                variants,
                |variant| fields_mapping(&variant.fields, Some(&variant.name))
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
pub fn generate_struct_field_accessors(fields: &Fields, self_access: SelfAccess) -> TokenStream {
    let field_assignments = fields
        .field_accessors()
        .iter()
        .zip(fields.normalized_field_idents().iter())
        .map(|(accessor, normalized_ident)| {
            match self_access {
                SelfAccess::Borrowed => {
                    quote! {
                        let #normalized_ident = &self.#accessor;
                    }
                }
                SelfAccess::Moved => {
                    quote! {
                        let #normalized_ident = self.#accessor;
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #(#field_assignments)*
    }
}