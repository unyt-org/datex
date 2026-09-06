use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use crate::datex_proxy::data::{
    FieldMapping, Fields, IndexedField, NamedField, Structure, StructureData,
};

/// Generates the implementation of the [ToInstructions] trait for the given structure data.
/// Returns a [TokenStream] containing the generated implementation.
pub fn generate_to_instructions(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    // TODO add for the other derives to fix generics!
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let body = match &structure_data.structure {
        Structure::Struct(fields) => {
            generate_to_instructions_for_struct(fields)
        }
        Structure::Enum(_variants) => {
            quote! {
                todo!("Implement ToInstructions for enums")
            }
        }
    };

    quote! {
        impl #impl_generics ToInstructions for #ident #ty_generics #where_clause {
            fn to_instructions<'ctx, 'a>(
                &'a self,
                ctx: &'a mut dyn ValueVisitor<'ctx>,
            ) -> Box<dyn Iterator<Item = Instruction> + 'a>
            where
                'ctx: 'a,
            {
                Box::new(gen move {
                    #body
                })
            }
        }
    }
}

/// Generates the instructions for a struct based on its fields.
fn generate_to_instructions_for_struct(fields: &Fields) -> TokenStream {
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

    let fields = match fields {
        Fields::Unit => {
            quote! {}
        }
        Fields::Named(fields) => generate_named_fields_to_instructions(fields),
        Fields::Unnamed(fields) => {
            generate_unnamed_fields_to_instructions(fields)
        }
        Fields::Transparent(field) => {
            generate_transparent_field_to_instructions(field)
        }
    };

    quote! {{
        #(#field_assignments)*
        #fields
    }}
}

/// Generates the instructions for named fields within a struct.
fn generate_named_fields_to_instructions(fields: &[NamedField]) -> TokenStream {
    let field_instructions = fields.iter().map(|field| {
        let accessor = field.ident_accessor().to_token_stream();
        let name = field.name.to_string();
        let mapping = &field.field.attributes.field_mapping;
        let value_instructions = field_to_instructions(accessor, mapping);

        quote! {
            yield RegularInstruction::text(
                #name.to_string()
            ).into();

            #value_instructions
        }
    });

    let field_count = fields.len() as u32;
    quote! {
        yield RegularInstruction::map(#field_count).into();
        #(
            #field_instructions
        )*
    }
}

/// Generates the instructions for unnamed fields within a struct.
fn generate_unnamed_fields_to_instructions(
    fields: &[IndexedField],
) -> TokenStream {
    let field_instructions = fields.iter().map(|field| {
        let accessor = field.index_accessor().to_token_stream();
        let mapping = &field.field.attributes.field_mapping;
        field_to_instructions(accessor, mapping)
    });

    let field_count = fields.len() as u32;
    quote! {
        yield RegularInstruction::list(#field_count).into();
        #(
            #field_instructions
        )*
    }
}

/// Generates the instructions for a transparent field within a struct.
fn generate_transparent_field_to_instructions(
    field: &IndexedField,
) -> TokenStream {
    let accessor = field.index_accessor().to_token_stream();
    let mapping = &field.field.attributes.field_mapping;
    field_to_instructions(accessor, mapping)
}

/// Generates the instructions for a single field based on its mapping.
fn field_to_instructions(
    accessor: TokenStream,
    field_mapping: &FieldMapping,
) -> TokenStream {
    match field_mapping {
        FieldMapping::Datex => {
            quote! {
                for i in (#accessor).to_instructions(ctx) {
                    yield i;
                }
            }
        }
        FieldMapping::Serde => {
            // we have to collect, as serde_to_value_container gives an owned container,
            // and this leads to lifetime issues if we don't collect the instructions first
            // but try to yield on the fly
            quote! {
                let value = serde_to_value_container(&(#accessor)).to_instructions(ctx).collect::<Vec<_>>();
                for i in value {
                    yield i;
                }
            }
        }
    }
}
