use crate::{
    ast::{
        expressions::{
            DatexExpressionData, EntityValueExpression, TagExpression,
        },
        spanned::Spanned,
    },
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::value::{
        value_classification::{ValueClassification, ValueTag},
    },
};
use crate::values::core_value_with_classification::borrowed_core_value_with_classification::BorrowedCoreValueWithClassification;

impl<'a> ToDatexExpressionData for BorrowedCoreValueWithClassification<'a> {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        let core_value_expression = self.inner.to_datex_expression_data();
        classification_expression(core_value_expression, &self.classification)
    }
}

fn classification_expression(
    mut expression: DatexExpressionData,
    classification: &ValueClassification,
) -> DatexExpressionData {
    if classification.is_unclassified() {
        return expression;
    }

    if !classification.impls.is_empty() {
        todo!()
    }

    if let Some(ValueTag { tag, is_empty }) = &classification.tag {
        expression = DatexExpressionData::Tag(TagExpression {
            tag: tag.clone(),
            expression: if !is_empty {
                Some(expression.with_default_span())
            } else {
                None
            },
        });
    }

    if let Some(entity_type) = &classification.entity_type {
        let name = entity_type.entity_definition().name.clone();
        expression = DatexExpressionData::EntityValue(EntityValueExpression {
            entity_name: name,
            entity_address: Some(entity_type.pointer_address()),
            value: expression.with_default_span(),
        })
    }

    expression
}
