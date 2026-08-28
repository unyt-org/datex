use crate::datex_proxy::data::{
    EnumVariant, Field, Fields, NamedField, Structure, StructureData,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

/// Creates the implementation of the [ToDatexExpressionData] trait for the given structure data.
/// Returns a TokenStream of the implementation.
pub fn generate_datex_expression_data(
    structure_data: &StructureData,
) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    let datex_expression_data = match &structure_data.structure {
        Structure::Enum(variants) => generate_datex_enum_fields(&variants),
        Structure::Struct(fields) => {
            generate_datex_expression_data_for_struct(&fields)
        }
    };

    quote! {
        impl #generics ToDatexExpressionData for #ident #generics {
            fn to_datex_expression_data(&self) -> DatexExpressionData {
                #datex_expression_data
            }
        }
    }
}

fn generate_datex_expression_data_for_struct(fields: &Fields) -> TokenStream {
    let field_assignments = fields
        .field_idents()
        .iter()
        .map(|ident| {
            quote! {
                let #ident = self.#ident;
            }
        })
        .collect::<Vec<_>>();
    let fields = generate_datex_expression_data_fields(fields);
    quote! {{
        #(#field_assignments)*
        #fields
    }}
}

/// Generates the datex expression data for fields. Returns a TokenStream of [DatexExpressionData].
fn generate_datex_expression_data_fields(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Unit => quote! {
            DatexExpressionData::Statements(Statements::empty())
        },
        Fields::Named(fields) => {
            let field_expressions = fields
                .iter()
                .map(|f| named_field_to_expression_data(f))
                .collect::<Vec<_>>();
            quote! {
                DatexExpressionData::Map(ast::expressions::Map::new(
                    vec![
                        #(#field_expressions),*
                    ]
                ))
            }
        }
        Fields::Unnamed(field) => {
            let field_expressions = field
                .iter()
                .enumerate()
                .map(|(i, f)| field_to_expression_data(f, quote! { #i }))
                .collect::<Vec<_>>();

            quote! {
                DatexExpressionData::List(List::new(
                    vec![
                        #(#field_expressions),*
                    ]
                ))
            }
        }
        Fields::Transparent(field) => {
            let first_field = field_to_expression_data(field, quote! { 0 });
            quote! {
                {
                    *(#first_field.data)
                }
            }
        }
    }
}

/// Generates a type definition for a single field. Returns a TokenStream of [TypeDefinition].
fn field_to_expression_data(
    field: &Field,
    accessor: TokenStream,
) -> TokenStream {
    let field_type = &field.ty;
    quote! {
        #accessor.to_datex_expression_data().with_default_span()
    }
}

/// Generates a type definition for a named field. Returns a TokenStream with a tuple of name and [TypeDefinition].
fn named_field_to_expression_data(field: &NamedField) -> TokenStream {
    let id = Ident::new(&field.name, Span::call_site());
    let expression_data =
        field_to_expression_data(&field.field, quote! { #id });
    let name = field.name.clone();
    quote! {
        (
            DatexExpressionData::Text(Text(#name.to_string())).with_default_span(),
            #expression_data,
        )
    }
}

/// Generates a type definition for an enum. Returns a TokenStream of [TypeDefinition].
fn generate_datex_enum_fields(enum_ty: &[EnumVariant]) -> TokenStream {
    let arms = enum_ty.iter().map(|variant| {
        let variant_ident = Ident::new(&variant.name, Span::call_site());
        let variant_name = &variant.name;
        let variant_fields =
            generate_datex_expression_data_fields(&variant.fields);
        let field_idents = variant.fields.field_idents();
        match &variant.fields {
            Fields::Named(fields) => {
                quote! {
                    #variant_ident { #(#field_idents),* } => {
                        #variant_fields
                    }
                }
            }
            Fields::Unnamed(fields) => {
                quote! {
                    #variant_ident(#(#field_idents),*) => {
                        #variant_fields
                    }
                }
            }
            Fields::Transparent(field) => {
                let first_field_ident = field_idents.first().unwrap();
                quote! {
                    #variant_ident(#first_field_ident) => {
                        #variant_fields
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    #variant_ident => {
                        #variant_fields
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
