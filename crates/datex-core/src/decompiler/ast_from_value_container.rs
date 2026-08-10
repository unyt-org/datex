use crate::{
    ast::{
        expressions::{DatexExpressionData, List, Map, RangeDeclaration},
        spanned::Spanned,
        type_expressions::{
            Intersection, RangeTypeExpr, TypeExpression, TypeExpressionData,
            Union,
        },
    },
    types::literal_type_definition::LiteralTypeDefinition,
    values::{
        core_value::CoreValue, value::Value, value_container::ValueContainer,
    },
};

use crate::{
    ast::{
        expressions::{
            CallableDeclaration, CreateShared, DeriveSharedRef,
            EntityDeclarationExpression, TagExpression,
        },
        type_expressions::{
            IdentifierWithPointerAddress, StructuralList, StructuralMap,
        },
    },
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    prelude::*,
    shared_values::{SharedContainer, traits::SharedContainerCommon},
    types::{
        shared_container_containing_entity_type::SharedContainerContainingEntityType,
        r#type::Type,
        type_definition::{
            TypeDefinition, range::RangeTypeDefinition,
            tagged_type::TaggedTypeDefinition,
        },
        type_definition_with_metadata::TypeDefinitionWithMetadata,
    },
};
use alloc::format;
use core::ops::Deref;

impl From<&ValueContainer> for DatexExpressionData {
    /// Converts a ValueContainer into a DatexExpression AST.
    /// This AST can then be further processed or decompiled into human-readable DATEX code.
    fn from(value: &ValueContainer) -> Self {
        match value {
            ValueContainer::Local(value) => value_to_datex_expression(value),
            ValueContainer::Shared(shared) => match shared {
                SharedContainer::Referenced(referenced_container) => {
                    DatexExpressionData::DeriveSharedRef(DeriveSharedRef {
                        mutability: referenced_container.reference_mutability(),
                        expression: (create_shared(referenced_container)
                            .with_default_span()),
                    })
                }
                SharedContainer::Owned(owned_container) => {
                    create_shared(owned_container)
                }
            },
        }
    }
}

fn create_shared(
    shared_container: &impl SharedContainerCommon,
) -> DatexExpressionData {
    if shared_container.is_borrowed() {
        DatexExpressionData::OmitRecursive
    } else {
        DatexExpressionData::CreateShared(CreateShared {
            mutability: shared_container.container_mutability(),
            expression: (DatexExpressionData::from(
                &*shared_container.value_container(),
            )
            .with_default_span()),
        })
    }
}

fn value_to_datex_expression(value: &Value) -> DatexExpressionData {
    let core_value_expression = core_value_to_datex_expression(&value.inner);
    if let Some(custom_type) = &value.custom_type {
        type_cast_expression(core_value_expression, custom_type)
    } else {
        core_value_expression
    }
}

fn core_value_to_datex_expression(
    core_value: &CoreValue,
) -> DatexExpressionData {
    match &core_value {
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
                start: (DatexExpressionData::from(&*range.start.clone())
                    .with_default_span()),
                end: (DatexExpressionData::from(&*range.end.clone())
                    .with_default_span()),
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
            DatexExpressionData::CallableDeclaration(CallableDeclaration {
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
                body: (DatexExpressionData::NativeImplementationIndicator
                    .with_default_span()),
                injected_variable_count: None,
            })
        }
        CoreValue::EntityTypeDefinition(_) => {
            todo!()
        }
    }
}

fn type_cast_expression(
    expression: DatexExpressionData,
    target_type: &TypeDefinition,
) -> DatexExpressionData {
    // special handling for some type casts
    match target_type {
        // #SomeTag (...)
        TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag,
            ty: Option::None,
        }) => DatexExpressionData::Tag(TagExpression {
            tag: tag.clone(),
            expression: Some(expression.with_default_span()),
        }),
        // #SomeTag
        TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag,
            ty:
                Some(box Type::Definition(TypeDefinitionWithMetadata {
                    definition:
                        TypeDefinition::CoreType(CoreLibTypeId::Base(
                            CoreLibBaseTypeId::Unit,
                        )),
                    ..
                })),
        }) => DatexExpressionData::Tag(TagExpression {
            tag: tag.clone(),
            expression: None,
        }),
        _ => todo!(),
    }
}

fn type_to_type_expression(ty: &Type) -> TypeExpression {
    match ty {
        Type::Entity(container) => {
            entity_type_container_to_type_expression(container)
        }
        Type::Definition(definition) => {
            type_definition_with_metadata_to_type_expression(definition)
        }
    }
}

fn entity_type_container_to_type_expression(
    container: &SharedContainerContainingEntityType,
) -> TypeExpression {
    let pointer_address = container.pointer_address();
    TypeExpressionData::IdentifierWithPointerAddress(
        IdentifierWithPointerAddress {
            name: container.entity_definition().name.clone(),
            pointer_address,
        },
    )
    .with_default_span()
}

fn type_definition_with_metadata_to_type_expression(
    type_def_with_metadata: &TypeDefinitionWithMetadata,
) -> TypeExpression {
    // TODO: handle type metadata
    type_definition_to_type_expression(&type_def_with_metadata.definition)
}
// FIXME can we make this consuming?
fn type_definition_to_type_expression(
    type_definition: &TypeDefinition,
) -> TypeExpression {
    match type_definition {
        TypeDefinition::Literal(struct_type) => match struct_type {
            LiteralTypeDefinition::Integer(integer) => {
                TypeExpressionData::Integer(integer.clone()).with_default_span()
            }
            LiteralTypeDefinition::Text(text) => {
                TypeExpressionData::Text(text.clone()).with_default_span()
            }
            LiteralTypeDefinition::Boolean(boolean) => {
                TypeExpressionData::Boolean(boolean.clone()).with_default_span()
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
        },
        TypeDefinition::Range(RangeTypeDefinition { start, end }) => {
            let x = type_to_type_expression(start);
            let y = type_to_type_expression(end);
            TypeExpressionData::Range(RangeTypeExpr {
                start: Box::new(x),
                end: Box::new(y),
            })
            .with_default_span()
        }
        TypeDefinition::Union(union_types) => TypeExpressionData::Union(Union(
            union_types
                .iter()
                .map(type_to_type_expression)
                .collect::<Vec<TypeExpression>>(),
        ))
        .with_default_span(),
        TypeDefinition::Intersection(intersection_types) => {
            TypeExpressionData::Intersection(Intersection(
                intersection_types
                    .iter()
                    .map(type_to_type_expression)
                    .collect::<Vec<TypeExpression>>(),
            ))
            .with_default_span()
        }
        TypeDefinition::Shared(_type_reference) => {
            todo!("#651 Handle type references in decompiler");
        }
        TypeDefinition::CoreType(core_type) => {
            TypeExpressionData::Identifier(core_type.to_string())
                .with_default_span()
        }
        TypeDefinition::Map(map_type) => {
            TypeExpressionData::StructuralMap(StructuralMap(
                map_type
                    .0
                    .iter()
                    .map(|(k, v)| {
                        (type_to_type_expression(k), type_to_type_expression(v))
                    })
                    .collect::<Vec<_>>(),
            ))
            .with_default_span()
        }
        TypeDefinition::List(list_type) => {
            TypeExpressionData::StructuralList(StructuralList(
                list_type
                    .0
                    .iter()
                    .map(type_to_type_expression)
                    .collect::<Vec<TypeExpression>>(),
            ))
            .with_default_span()
        }
        _ => TypeExpressionData::Text(
            format!("[[TYPE {:?}]]", type_definition).into(),
        )
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
                decimal::{Decimal, typed_decimal::TypedDecimal},
                integer::{Integer, typed_integer::TypedInteger},
                range::Range,
            },
            value::Value,
            value_container::ValueContainer,
        },
    };

    use crate::prelude::*;
    #[test]
    fn integer_to_ast() {
        let value = ValueContainer::from(Integer::from(42));
        let ast = DatexExpressionData::from(&value);
        assert_eq!(ast, DatexExpressionData::Integer(Integer::from(42)));
    }

    #[test]
    fn typed_integer_to_ast() {
        let value = ValueContainer::from(TypedInteger::from(42i8));
        let ast = DatexExpressionData::from(&value);
        assert_eq!(
            ast,
            DatexExpressionData::TypedInteger(TypedInteger::from(42i8))
        );
    }

    #[test]
    fn decimal_to_ast() {
        let value = ValueContainer::from(Decimal::from(1.23));
        let ast = DatexExpressionData::from(&value);
        assert_eq!(ast, DatexExpressionData::Decimal(Decimal::from(1.23)));
    }

    #[test]
    fn typed_decimal_to_ast() {
        let value = ValueContainer::from(TypedDecimal::from(2.71f32));
        let ast = DatexExpressionData::from(&value);
        assert_eq!(
            ast,
            DatexExpressionData::TypedDecimal(TypedDecimal::from(2.71f32))
        );
    }

    #[test]
    fn boolean_to_ast() {
        let value = ValueContainer::from(true);
        let ast = DatexExpressionData::from(&value);
        assert_eq!(ast, DatexExpressionData::Boolean(true.into()));
    }

    #[test]
    fn text_to_ast() {
        let value = ValueContainer::from("Hello, World!".to_string());
        let ast = DatexExpressionData::from(&value);
        assert_eq!(ast, DatexExpressionData::Text("Hello, World!".into()));
    }

    #[test]
    fn null_to_ast() {
        let value = ValueContainer::Local(Value::null());
        let ast = DatexExpressionData::from(&value);
        assert_eq!(ast, DatexExpressionData::Null);
    }

    #[test]
    fn list_to_ast() {
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
                start: (DatexExpressionData::Integer(Integer::from(11))
                    .with_default_span()),
                end: (DatexExpressionData::Integer(Integer::from(13))
                    .with_default_span()),
            })
        );
    }
}
