use crate::prelude::*;
use crate::ast::spanned::Spanned;
use crate::ast::type_expressions::{Intersection, RangeTypeExpr, StructuralList, StructuralMap, TypeExpression, TypeExpressionData, Union};
use crate::traits::to_type_expression_data::ToTypeExpressionData;
use crate::types::literal_type_definition::LiteralTypeDefinition;
use crate::types::type_definition::range::RangeTypeDefinition;
use crate::types::type_definition::TypeDefinition;


impl ToTypeExpressionData for TypeDefinition {
    fn to_type_expression_data(&self) -> TypeExpressionData {
        match self {
            TypeDefinition::Literal(struct_type) => match struct_type {
                LiteralTypeDefinition::Integer(integer) => {
                    TypeExpressionData::Integer(integer.clone())
                }
                LiteralTypeDefinition::Text(text) => {
                    TypeExpressionData::Text(text.clone())
                }
                LiteralTypeDefinition::Boolean(boolean) => {
                    TypeExpressionData::Boolean(boolean.clone())
                }
                LiteralTypeDefinition::Decimal(decimal) => {
                    TypeExpressionData::Decimal(decimal.clone())
                }
                LiteralTypeDefinition::TypedInteger(typed_integer) => {
                    TypeExpressionData::TypedInteger(typed_integer.clone())
                        
                }
                LiteralTypeDefinition::TypedDecimal(typed_decimal) => {
                    TypeExpressionData::TypedDecimal(typed_decimal.clone())
                        
                }
                LiteralTypeDefinition::Endpoint(endpoint) => {
                    TypeExpressionData::Endpoint(endpoint.clone())
                    
                }
            },
            TypeDefinition::Range(RangeTypeDefinition { start, end }) => {
                TypeExpressionData::Range(RangeTypeExpr {
                    start: Box::new(start.to_type_expression_data().with_default_span()),
                    end: Box::new(end.to_type_expression_data().with_default_span()),
                })
                    
            }
            TypeDefinition::Union(union_types) => TypeExpressionData::Union(Union(
                union_types
                    .iter()
                    .map(|ty| ty.to_type_expression_data().with_default_span())
                    .collect::<Vec<TypeExpression>>(),
            ))
                ,
            TypeDefinition::Intersection(intersection_types) => {
                TypeExpressionData::Intersection(Intersection(
                    intersection_types
                        .iter()
                        .map(|ty| ty.to_type_expression_data().with_default_span())
                        .collect::<Vec<TypeExpression>>(),
                ))
                    
            }
            TypeDefinition::Shared(_type_reference) => {
                todo!("#651 Handle type references in decompiler");
            }
            TypeDefinition::CoreType(core_type) => {
                TypeExpressionData::Identifier(core_type.to_string())
                    
            }
            TypeDefinition::Map(map_type) => {
                TypeExpressionData::StructuralMap(StructuralMap(
                    map_type
                        .0
                        .iter()
                        .map(|(k, v)| {
                            (
                                k.to_type_expression_data().with_default_span(),
                                v.to_type_expression_data().with_default_span()
                            )
                        })
                        .collect::<Vec<_>>(),
                ))
                    
            }
            TypeDefinition::List(list_type) => {
                TypeExpressionData::StructuralList(StructuralList(
                    list_type
                        .0
                        .iter()
                        .map(|ty| ty.to_type_expression_data().with_default_span())
                        .collect::<Vec<TypeExpression>>(),
                ))
                    
            }
            _ => TypeExpressionData::Text(
                format!("[[TYPE {:?}]]", self).into(),
            )
        }
    }
}