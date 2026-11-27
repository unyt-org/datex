use crate::ModuleGeneratorContext;
use proc_macro2::TokenStream;
use quote::quote;
use datex_core::libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId};
use datex_core::shared_wrappers::shared::shared_ref::SharedRef;
use datex_core::types::type_definition::TypeDefinition;
use datex_core::values::core_values::callable::Callable;
use super::CodeGenerator;

#[derive(Debug, Clone, Default)]
pub struct RustCodeGenerator;

impl CodeGenerator for RustCodeGenerator {
    type Code = TokenStream;

    fn code_to_string(&self, code: &Self::Code) -> String {
        let file: syn::File = syn::parse2(code.clone()).expect("invalid Rust");
        prettyplease::unparse(&file)
    }

    fn join_code_statements(&self, statements: &[Self::Code]) -> Self::Code {
        let mut combined = TokenStream::new();
        for statement in statements {
            combined.extend(statement.clone());
        }
        combined
    }

    fn fold_doc_comment(&mut self, comment: &str) -> Self::Code {
        let doc_comment = format!("/// {}", comment.replace("\n", "\n/// "));
        let doc_comment_token: TokenStream = syn::parse_str(&doc_comment)
            .expect("Failed to parse doc comment");
        doc_comment_token
    }

    fn fold_callable(&mut self, module_context: &mut ModuleGeneratorContext<Self::Code>, name: &str, callable: SharedRef<Callable>) -> Self::Code {
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());

        let borrowed_callable = callable.borrow();
        let signature = &borrowed_callable.signature;
        let return_type = if let Some(return_type) = &signature.return_type {
            let ty = self.fold_type_definition(return_type.clone().convert_to_definition(), false);
            quote! { -> #ty }
        } else {
            quote! {}
        };

        let parameters = signature.parameters.iter().enumerate().map(|(index, param)| {
            let name = param.0.clone().unwrap_or_else(|| format!("param_{}", index));
            let param_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
            let param_type = self.fold_type_definition(param.1.clone().convert_to_definition(), false);
            quote! { #param_name: #param_type }
        }).collect::<Vec<_>>();
        
        let doc_comment = if let Some(docs) = &borrowed_callable.docs {
            self.fold_doc_comment(docs)
        } else {
            TokenStream::new()
        };

        quote! {
            #doc_comment
            pub fn #ident(#(#parameters),*) #return_type {
                todo!()
            }
        }
    }

    fn fold_constant_declaration(&mut self, name: &str, type_declaration: Self::Code, value: Self::Code) -> Self::Code {
        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        quote! {
            pub const #ident: #type_declaration = #value;
        }
    }

    fn fold_text(&mut self, text: &str) -> Self::Code {
        let text_literal = syn::LitStr::new(text, proc_macro2::Span::call_site());
        quote! {
            #text_literal
        }
    }

    fn fold_type_definition(&mut self, ty: TypeDefinition, is_const: bool,) -> Self::Code {
        match ty {
            TypeDefinition::CoreType(core_type) => {
                match core_type {
                    CoreLibTypeId::Base(base_type) => match base_type {
                        CoreLibBaseTypeId::Text => if is_const {
                            quote! { &'static str }
                        } else {
                            quote! { String }
                        },
                        CoreLibBaseTypeId::Boolean => quote! { bool },
                        _ => todo!()
                    }
                    CoreLibTypeId::Variant(variant_id) => {
                        todo!()
                    }
                }
            }
            _ => todo!()
        }
    }
}