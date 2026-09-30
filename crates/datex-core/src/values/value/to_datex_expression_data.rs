use crate::{
    ast::{
        expressions::{
            DatexExpressionData, EntityValueExpression, TagExpression,
        },
        spanned::Spanned,
    },
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::value::{
        Value,
        value_classification::{ValueClassification, ValueTag},
    },
};

impl ToDatexExpressionData for Value {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        let core_value_expression = self.inner.to_datex_expression_data();
        classification_expression(core_value_expression, &self.classification)
    }
}

fn classification_expression(
    mut expression: DatexExpressionData,
    classification: &ValueClassification,
) -> DatexExpressionData {
    if classification.is_none() {
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
