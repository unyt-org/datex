use crate::datex_proxy::data::{StructureData, TypeKind};
use proc_macro2::TokenStream;
use quote::quote;

/// Generates the [Classification] and [StaticClassification] implementations
pub fn generate_classification(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let has_classification = !structure_data
        .attributes.type_kind.is_structural();
    
    let classification_methods = generate_classification_methods(structure_data);
    
    quote! {
        #[automatically_derived]
        impl #impl_generics Classification for #ident #ty_generics #where_clause {
            #classification_methods
        }

        #[automatically_derived]
        impl #impl_generics StaticClassification for #ident #ty_generics #where_clause {
            fn has_classification() -> bool {
                #has_classification            
            }
        }
    }
}


fn generate_classification_methods(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    if structure_data.attributes.type_kind.is_entity() {
        quote! {
            fn entity_type(
                &self,
                _cache: &mut SharedReferencesCache,
            ) -> Option<EntityType> {
                todo!()
            }
            
            fn tag(&self) -> Option<ValueTag> {
                None
            }
        }
    } else {
        quote! {}
    }
}