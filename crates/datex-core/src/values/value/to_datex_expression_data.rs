use crate::ast::expressions::{DatexExpressionData, EntityValueExpression, TagExpression};
use crate::ast::spanned::Spanned;
use crate::libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId};
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::types::r#type::Type;
use crate::types::type_definition::tagged_type::TaggedTypeDefinition;
use crate::types::type_definition::TypeDefinition;
use crate::types::type_definition_with_metadata::TypeDefinitionWithMetadata;
use crate::values::value::Value;

impl ToDatexExpressionData for Value {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        let core_value_expression = self.inner.to_datex_expression_data();
        // only entity types with a non default type need to be casted
        if self.needs_type_cast()
            && let Some(custom_type) = &self.custom_type
        {
            type_cast_expression(core_value_expression, custom_type)
        } else {
            core_value_expression
        }
    }
}


/// Helper function to handle type casting of expressions based on the target type.
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
        // Entity {...}
        TypeDefinition::Box(box Type::Entity(entity_container)) => {
            DatexExpressionData::EntityValue(EntityValueExpression {
                entity_name: entity_container.entity_definition().name.clone(),
                entity_address: Some(entity_container.pointer_address()),
                value: expression.with_default_span(),
            })
        }
        e => {
            todo!("Handle type cast to {:?} in decompiler", e)
        }
    }
}