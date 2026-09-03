use crate::{
    ast::{
        expressions::{
            CallableDeclaration, CallableSignature, DatexExpressionData,
        },
        spanned::Spanned,
    },
    prelude::*,
    traits::{
        to_datex_expression_data::ToDatexExpressionData,
        to_type_expression_data::ToTypeExpressionData,
    },
    values::core_values::callable::{
        Callable, CallableBody, DatexBytecodeCallable,
    },
};

impl ToDatexExpressionData for Callable {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::CallableDeclaration(CallableDeclaration {
            signature: CallableSignature {
                name: self.name.clone(),
                kind: self.signature.kind,
                requires_async: self.signature.requires_async,
                parameters: self
                    .signature
                    .parameters
                    .iter()
                    .map(|(maybe_name, ty)| {
                        (
                            maybe_name.clone().unwrap_or("_".to_string()),
                            ty.to_type_expression_data().with_default_span(),
                        )
                    })
                    .collect(),
                rest_parameter: self.signature.rest_parameter.as_ref().map(
                    |(maybe_name, ty)| {
                        (
                            maybe_name.clone().unwrap_or("_".to_string()),
                            ty.to_type_expression_data().with_default_span(),
                        )
                    },
                ),
                return_type: self
                    .signature
                    .return_type
                    .as_ref()
                    .map(|ty| ty.to_type_expression_data().with_default_span()),
                yeet_type: self
                    .signature
                    .yeet_type
                    .as_ref()
                    .map(|ty| ty.to_type_expression_data().with_default_span()),
            },
            body: match &self.body {
                CallableBody::CoreStub(_) => {
                    DatexExpressionData::NativeImplementationIndicator
                        .with_default_span()
                }
                CallableBody::Native(_) => {
                    DatexExpressionData::NativeImplementationIndicator
                        .with_default_span()
                }
                CallableBody::Hidden => {
                    DatexExpressionData::NativeImplementationIndicator
                        .with_default_span()
                }
                #[cfg(feature = "decompiler")]
                CallableBody::DatexBytecode(DatexBytecodeCallable {
                    body,
                    ..
                }) => {
                    use crate::decompiler::dxb_to_source_code::ast_from_bytecode::ast_from_bytecode;
                    
                    ast_from_bytecode(body).unwrap_or_else(|_| {
                        DatexExpressionData::Noop.with_default_span()
                    })
                },
                #[cfg(not(feature = "decompiler"))]
                CallableBody::DatexBytecode(_) => {
                    DatexExpressionData::NativeImplementationIndicator
                        .with_default_span()
                }
            },
            injected_variable_count: None,
        })
    }
}
