use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    datex_proxy::data::{FieldIdent, Fields, Structure, StructureData},
    generator::helpers::{
        SelfAccess, generate_struct_or_enum_variants_fields_mapping,
    },
};

/// Generates the [ChildIterator] implementations
pub fn generate_child_iterator(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let iterator_methods =
        generate_child_property_methods(&structure_data.structure);

    quote! {
        #[automatically_derived]
        impl #impl_generics ChildIterator for #ident #ty_generics #where_clause {
            #iterator_methods
        }
    }
}

fn generate_child_property_methods(structure: &Structure) -> TokenStream {
    let child_iterator = generate_struct_or_enum_variants_fields_mapping(
        structure,
        SelfAccess::Borrowed,
        |fields, _| generate_iterator(fields, false),
    );
    let child_iterator_mut = generate_struct_or_enum_variants_fields_mapping(
        structure,
        SelfAccess::Borrowed,
        |fields, _| generate_iterator(fields, true),
    );
    quote! {
        fn iter_children<'a>(&'a self) -> Option<Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>> {
            #child_iterator
        }
        fn iter_children_mut<'a>(&'a mut self) -> Option<Box<dyn Iterator<Item = BorrowedValueContainerMut<'a>> + 'a>> {
            todo!() // We can not provide the keys of the struct as mutable ref, what shall we do instead?
            // ignore the keys? Then it would be inconsistent to the immutable iterator. Give back Options for all items?
            // or just return None
        }
    }
}

fn generate_iterator(fields: &Fields, is_mut: bool) -> TokenStream {
    let as_borrowed_value_container_with_mutability = if is_mut {
        quote! {
            as_borrowed_value_container_mut
        }
    } else {
        quote! {
            as_borrowed_value_container
        }
    };
    let borrowed_value_container_with_mutability = if is_mut {
        quote! {
            BorrowedValueContainerMut
        }
    } else {
        quote! {
            BorrowedValueContainer
        }
    };
    match fields {
        Fields::Unnamed(unnnamed) => {
            let fields = unnnamed
                .iter()
                .map(|field| {
                    let ident = field.normalized_ident();
                    quote! {
                        yield #ident.#as_borrowed_value_container_with_mutability();
                    }
                })
                .collect::<Vec<_>>();
            quote! {
                Some(Box::new(gen move {
                    #(#fields)*
                }))
            }
        }
        Fields::Named(named) => {
            let fields_with_names = named
                .iter()
                .map(|(field)| {
                    let name = field.datex_field_name();
                    let ident = field.normalized_ident();
                    quote! {
                        yield #borrowed_value_container_with_mutability::from(#name);
                        yield #ident.#as_borrowed_value_container_with_mutability();
                    }
                })
                .collect::<Vec<_>>();
            quote! {
                Some(Box::new(gen move {
                    #(#fields_with_names)*
                }))
            }
        }
        Fields::Transparent(inner) => {
            let inner = inner.normalized_ident();
            quote! {
                #inner.iter_children()
            }
        }
        Fields::Unit => {
            quote! {
                None
            }
        }
    }
}
