use crate::ast::expressions::{CallableDeclaration, CallableSignature, DatexExpressionData, List, Map, RangeDeclaration, Statements};
use crate::ast::spanned::Spanned;
use crate::decompiler::ast_from_bytecode::ast_from_bytecode;
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::traits::to_type_expression_data::ToTypeExpressionData;
use crate::values::core_value::CoreValue;
use crate::values::core_values::callable::{CallableBody, DatexBytecodeCallable};
use crate::values::value_container::ValueContainer;

impl ToDatexExpressionData for CoreValue {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        match &self {
            CoreValue::Integer(integer) => {
                DatexExpressionData::Integer(integer.clone())
            }
            CoreValue::TypedInteger(typed_integer) => {
                DatexExpressionData::TypedInteger(typed_integer.clone())
            }
            CoreValue::Decimal(decimal) => {
                DatexExpressionData::Decimal(decimal.clone())
            }
            CoreValue::TypedDecimal(typed_decimal) => {
                DatexExpressionData::TypedDecimal(typed_decimal.clone())
            }
            CoreValue::Boolean(boolean) => {
                DatexExpressionData::Boolean(boolean.clone())
            }
            CoreValue::Text(text) => DatexExpressionData::Text(text.clone()),

            CoreValue::Range(range) => {
                DatexExpressionData::Range(RangeDeclaration {
                    start: (range.start.to_datex_expression_data()
                        .with_default_span()),
                    end: (range.end.to_datex_expression_data()
                        .with_default_span()),
                })
            }

            CoreValue::Endpoint(endpoint) => {
                DatexExpressionData::Endpoint(endpoint.clone())
            }
            CoreValue::Null => DatexExpressionData::Null,
            CoreValue::List(list) => DatexExpressionData::List(List::new(
                list.into_iter()
                    .map(|item| item.to_datex_expression_data().with_default_span())
                    .collect(),
            )),
            CoreValue::Map(map) => DatexExpressionData::Map(Map::new(
                map.iter()
                    .map(|(key, value)| {
                        (
                            ValueContainer::from(key).to_datex_expression_data()
                                .with_default_span(),
                            value.to_datex_expression_data().with_default_span(),
                        )
                    })
                    .collect(),
            )),
            CoreValue::Type(type_value) => DatexExpressionData::TypeExpression(
                type_value.to_type_expression_data().with_default_span(),
            ),
            CoreValue::Callable(callable) => {
                DatexExpressionData::CallableDeclaration(CallableDeclaration {
                    signature: CallableSignature {
                        name: callable.name.clone(),
                        kind: callable.signature.kind,
                        requires_async: callable.signature.requires_async,
                        parameters: callable
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
                        rest_parameter: callable
                            .signature
                            .rest_parameter
                            .as_ref()
                            .map(|(maybe_name, ty)| {
                                (
                                    maybe_name.clone().unwrap_or("_".to_string()),
                                    ty.to_type_expression_data().with_default_span(),
                                )
                            }),
                        return_type: callable
                            .signature
                            .return_type
                            .as_ref()
                            .map(|ty| ty.to_type_expression_data().with_default_span()),
                        yeet_type: callable
                            .signature
                            .yeet_type
                            .as_ref()
                            .map(|ty| ty.to_type_expression_data().with_default_span()),
                    },
                    body: match &callable.body {
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
                        CallableBody::DatexBytecode(DatexBytecodeCallable {
                            body,
                            ..
                        }) => ast_from_bytecode(body).unwrap_or_else(|_| {
                            DatexExpressionData::Noop.with_default_span()
                        }), // TODO: handle error?
                    },
                    injected_variable_count: None,
                })
            }
            CoreValue::EntityTypeDefinition(_) => {
                todo!()
            }
            CoreValue::Uninitialized => {
                todo!()
            }
            CoreValue::Box(inner) => DatexExpressionData::Statements(Statements {
                statements: vec![
                    inner.to_datex_expression_data().with_default_span(),
                ],
                is_terminated: false,
                unbounded: None,
            }),
            CoreValue::Native(_) => {
                DatexExpressionData::NativeImplementationIndicator
            }
        }
    }
}