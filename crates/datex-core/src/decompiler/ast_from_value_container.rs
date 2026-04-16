use crate::{
    ast::{
        expressions::{DatexExpressionData, List, Map, RangeDeclaration},
        spanned::Spanned,
        type_expressions::{
            Intersection, RangeTypeExpr, TypeExpression, TypeExpressionData,
            Union,
        },
    },
    types::{
        literal_type_definition::LiteralTypeDefinition,
        structural_type_definition::StructuralTypeDefinition,
    },
    values::{
        core_value::CoreValue, value::Value,
        value_container::ValueContainer,
    },
};

use crate::{
    ast::expressions::{CallableDeclaration, CreateShared, GetSharedRef},
    prelude::*,
};
use alloc::format;
use crate::libs::core::core_lib_id::CoreLibId;
use crate::shared_values::shared_containers::SharedContainer;
use crate::types::r#type::Type;
use crate::types::type_definition::TypeDefinition;

impl From<&ValueContainer> for DatexExpressionData {
    /// Converts a ValueContainer into a DatexExpression AST.
    /// This AST can then be further processed or decompiled into human-readable DATEX code.
    fn from(value: &ValueContainer) -> Self {
        match value {
            ValueContainer::Local(value) => value_to_datex_expression(value),
            ValueContainer::Shared(shared) => {
                match shared {
                    SharedContainer::Referenced(referenced_container) => {
                        DatexExpressionData::GetSharedRef(GetSharedRef {
                            mutability: referenced_container.reference_mutability(),
                            expression: Box::new(
                                DatexExpressionData::CreateShared(
                                    CreateShared {
                                        mutability: referenced_container.container_mutability(),
                                        expression: Box::new(
                                            DatexExpressionData::from(
                                                &*shared.value_container()
                                            )
                                            .with_default_span(),
                                        ),
                                    },
                                )
                                .with_default_span(),
                            ),
                        })
                    }
                    SharedContainer::Owned(owned_container) => DatexExpressionData::CreateShared(CreateShared {
                        mutability: owned_container.container_mutability(),
                        expression: Box::new(
                            DatexExpressionData::from(
                                &*owned_container.value_container(),
                            )
                            .with_default_span(),
                        ),
                    }),
                }
            }
        }
    }
}

fn value_to_datex_expression(value: &Value) -> DatexExpressionData {
    match &value.inner {
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
        CoreValue::Boolean(boolean) => DatexExpressionData::Boolean(boolean.0),
        CoreValue::Text(text) => DatexExpressionData::Text(text.0.clone()),

        CoreValue::Range(range) => {
            DatexExpressionData::Range(RangeDeclaration {
                start: Box::new(
                    DatexExpressionData::from(&*range.start.clone())
                        .with_default_span(),
                ),
                end: Box::new(
                    DatexExpressionData::from(&*range.end.clone())
                        .with_default_span(),
                ),
            })
        }

        CoreValue::Endpoint(endpoint) => {
            DatexExpressionData::Endpoint(endpoint.clone())
        }
        CoreValue::Null => DatexExpressionData::Null,
        CoreValue::List(list) => DatexExpressionData::List(List::new(
            list.into_iter()
                .map(DatexExpressionData::from)
                .map(|data| data.with_default_span())
                .collect(),
        )),
        CoreValue::Map(map) => DatexExpressionData::Map(Map::new(
            map.iter()
                .map(|(key, value)| {
                    (
                        DatexExpressionData::from(&ValueContainer::from(key))
                            .with_default_span(),
                        DatexExpressionData::from(value).with_default_span(),
                    )
                })
                .collect(),
        )),
        CoreValue::Type(type_value) => DatexExpressionData::TypeExpression(
            type_to_type_expression(type_value),
        ),
        CoreValue::Callable(callable) => {
            DatexExpressionData::CallableDeclaration(Box::new(
                CallableDeclaration {
                    name: callable.name.clone(),
                    kind: callable.signature.kind.clone(),
                    parameters: callable
                        .signature
                        .parameter_types
                        .iter()
                        .map(|(maybe_name, ty)| {
                            (
                                maybe_name.clone().unwrap_or("_".to_string()),
                                type_to_type_expression(ty),
                            )
                        })
                        .collect(),
                    rest_parameter: callable
                        .signature
                        .rest_parameter_type
                        .as_ref()
                        .map(|(maybe_name, ty)| {
                            (
                                maybe_name.clone().unwrap_or("_".to_string()),
                                type_to_type_expression(ty),
                            )
                        }),
                    return_type: callable
                        .signature
                        .return_type
                        .as_ref()
                        .map(|ty| type_to_type_expression(ty)),
                    yeet_type: callable
                        .signature
                        .yeet_type
                        .as_ref()
                        .map(|ty| type_to_type_expression(ty)),
                    body: Box::new(
                        DatexExpressionData::NativeImplementationIndicator
                            .with_default_span(),
                    ),
                    injected_variable_count: None,
                },
            ))
        }
    }
}

fn type_to_type_expression(type_value: &Type) -> TypeExpression {
    // TODO: handle nominal types
    type_definition_to_type_expression(&type_value.definition())
}

fn type_definition_to_type_expression(type_value: &TypeDefinition) -> TypeExpression {
    // TODO: handle type metadata
    structural_type_definition_to_type_expression(&type_value.structural_definition)
}


fn structural_type_definition_to_type_expression(type_definition: &StructuralTypeDefinition) -> TypeExpression {
    match type_definition {
        StructuralTypeDefinition::Literal(struct_type) => match struct_type {
            LiteralTypeDefinition::Integer(integer) => {
                TypeExpressionData::Integer(integer.clone()).with_default_span()
            }
            LiteralTypeDefinition::Text(text) => {
                TypeExpressionData::Text(text.clone()).with_default_span()
            }
            LiteralTypeDefinition::Boolean(boolean) => {
                TypeExpressionData::Boolean(*boolean).with_default_span()
            }
            LiteralTypeDefinition::Decimal(decimal) => {
                TypeExpressionData::Decimal(decimal.clone()).with_default_span()
            }
            LiteralTypeDefinition::TypedInteger(typed_integer) => {
                TypeExpressionData::TypedInteger(typed_integer.clone())
                    .with_default_span()
            }
            LiteralTypeDefinition::TypedDecimal(typed_decimal) => {
                TypeExpressionData::TypedDecimal(typed_decimal.clone())
                    .with_default_span()
            }
            LiteralTypeDefinition::Endpoint(endpoint) => {
                TypeExpressionData::Endpoint(endpoint.clone())
                    .with_default_span()
            }
            LiteralTypeDefinition::Null => {
                TypeExpressionData::Null.with_default_span()
            }
            _ => TypeExpressionData::Text(format!(
                "[[STRUCTURAL TYPE {:?}]]",
                struct_type
            ))
                .with_default_span(),
        },
        StructuralTypeDefinition::Range((start_type, end_type)) => {
            let x = type_to_type_expression(start_type);
            let y = type_to_type_expression(end_type);
            TypeExpressionData::Range(RangeTypeExpr {
                start: Box::new(x),
                end: Box::new(y),
            })
                .with_default_span()
        }
        StructuralTypeDefinition::Union(union_types) => TypeExpressionData::Union(Union(
            union_types
                .iter()
                .map(type_to_type_expression)
                .collect::<Vec<TypeExpression>>(),
        ))
        .with_default_span(),
        StructuralTypeDefinition::Intersection(intersection_types) => {
            TypeExpressionData::Intersection(Intersection(
                intersection_types
                    .iter()
                    .map(type_to_type_expression)
                    .collect::<Vec<TypeExpression>>(),
            ))
            .with_default_span()
        }
        StructuralTypeDefinition::Unit => TypeExpressionData::Unit.with_default_span(),
        StructuralTypeDefinition::Shared(type_reference) => {
            // try to resolve to core lib value
            if let Ok(core_lib_type) = CoreLibId::try_from(
                type_reference.pointer_address(),
            ) {
                TypeExpressionData::Identifier(core_lib_type.to_string())
                    .with_default_span()
            } else {
                todo!("#651 Handle non-core-lib type references in decompiler");
            }
        }
        _ => TypeExpressionData::Text(format!(
            "[[TYPE {:?}]]",
            type_definition
        ))
        .with_default_span(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        alloc::boxed::Box,
        ast::{
            expressions::{DatexExpressionData, List, RangeDeclaration},
            spanned::Spanned,
        },
        values::{
            core_values::{
                decimal::{typed_decimal::TypedDecimal, Decimal},
                integer::{typed_integer::TypedInteger, Integer},
                range::Range,
            },
            value::Value,
            value_container::ValueContainer,
        },
    };

    use crate::prelude::*;
    #[test]
    fn test_integer_to_ast() {
        let value = ValueContainer::from(Integer::from(42));
        let ast = DatexExpressionData::from(&value);
        assert_eq!(ast, DatexExpressionData::Integer(Integer::from(42)));
    }

    #[test]
    fn test_typed_integer_to_ast() {
        let value = ValueContainer::from(TypedInteger::from(42i8));
        let ast = DatexExpressionData::from(&value);
        assert_eq!(
            ast,
            DatexExpressionData::TypedInteger(TypedInteger::from(42i8))
        );
    }

    #[test]
    fn test_decimal_to_ast() {
        let value = ValueContainer::from(Decimal::from(1.23));
        let ast = DatexExpressionData::from(&value);
        assert_eq!(ast, DatexExpressionData::Decimal(Decimal::from(1.23)));
    }

    #[test]
    fn test_typed_decimal_to_ast() {
        let value = ValueContainer::from(TypedDecimal::from(2.71f32));
        let ast = DatexExpressionData::from(&value);
        assert_eq!(
            ast,
            DatexExpressionData::TypedDecimal(TypedDecimal::from(2.71f32))
        );
    }

    #[test]
    fn test_boolean_to_ast() {
        let value = ValueContainer::from(true);
        let ast = DatexExpressionData::from(&value);
        assert_eq!(ast, DatexExpressionData::Boolean(true));
    }

    #[test]
    fn test_text_to_ast() {
        let value = ValueContainer::from("Hello, World!".to_string());
        let ast = DatexExpressionData::from(&value);
        assert_eq!(ast, DatexExpressionData::Text("Hello, World!".to_string()));
    }

    #[test]
    fn test_null_to_ast() {
        let value = ValueContainer::Local(Value::null());
        let ast = DatexExpressionData::from(&value);
        assert_eq!(ast, DatexExpressionData::Null);
    }

    #[test]
    fn test_list_to_ast() {
        let value = ValueContainer::from(vec![
            Integer::from(1),
            Integer::from(2),
            Integer::from(3),
        ]);
        let ast = DatexExpressionData::from(&value);
        assert_eq!(
            ast,
            DatexExpressionData::List(List::new(vec![
                DatexExpressionData::Integer(Integer::from(1))
                    .with_default_span(),
                DatexExpressionData::Integer(Integer::from(2))
                    .with_default_span(),
                DatexExpressionData::Integer(Integer::from(3))
                    .with_default_span(),
            ]))
        );
    }

    #[test]
    fn range_to_ast() {
        let range = ValueContainer::from(Range {
            start: Box::new(Integer::from(11).into()),
            end: Box::new(Integer::from(13).into()),
        });
        let ast = DatexExpressionData::from(&range);
        assert_eq!(
            ast,
            DatexExpressionData::Range(RangeDeclaration {
                start: Box::new(
                    DatexExpressionData::Integer(Integer::from(11))
                        .with_default_span()
                ),
                end: Box::new(
                    DatexExpressionData::Integer(Integer::from(13))
                        .with_default_span()
                ),
            })
        );
    }
}
