use proc_macro2::TokenStream;
use quote::quote;

use crate::datex_proxy::data::{StructureData, TypeKind};

/// Generates the [ToInstructions] implementation for the given structure data.
/// Returns a TokenStream of the implementations.
pub fn generate_to_instructions(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    let native_only_structural_impl =
        generate_datex_native_only_structural(structure_data);

    quote! {
        impl #generics ToInstructions for #ident #generics {
            fn to_instructions<'ctx, 'a>(
                &'a self,
                ctx: &'a mut dyn ValueVisitor<'ctx>,
            ) -> Box<dyn Iterator<Item = Instruction> + 'a>
            where
                'ctx: 'a,
            {
                Box::new(gen move {
                    todo!()
                })
            }
        }
    }
}

pub fn generate_datex_native_only_structural(
    structure_data: &StructureData,
) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;
    // TODO: validate that all children also implement DatexNativeOnlyStructural
    match structure_data.attributes.type_kind {
        TypeKind::Structural {
            only_structural: false,
        } => quote! {
            impl #generics DatexNativeStructural for #ident #generics {}
        },
        TypeKind::Structural {
            only_structural: true,
        } => quote! {
            impl #generics DatexNativeStructural for #ident #generics {}
            impl #generics DatexNativeOnlyStructural for #ident #generics {}
        },
        _ => quote! {},
    }
}
