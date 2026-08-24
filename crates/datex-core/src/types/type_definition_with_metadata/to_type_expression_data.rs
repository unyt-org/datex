use crate::ast::spanned::Spanned;
use crate::ast::type_expressions::TypeExpressionData;
use crate::shared_values::SharedContainerMutability;
use crate::traits::to_type_expression_data::ToTypeExpressionData;
use crate::types::type_definition_with_metadata::{LocalMutability, LocalOwnership, LocalReferenceMutability, TypeDefinitionWithMetadata, TypeMetadata};

impl ToTypeExpressionData for TypeDefinitionWithMetadata {
    fn to_type_expression_data(&self) -> TypeExpressionData {
        let mut type_expr = self.definition.to_type_expression_data();

        // add inner "mut"
        if matches!(self.metadata, TypeMetadata::Local { mutability: LocalMutability::Mutable, .. })
            || matches!(self.metadata, TypeMetadata::Shared { mutability: SharedContainerMutability::Mutable, .. })
        {
            type_expr = TypeExpressionData::Mut(Box::new(type_expr.with_default_span()));
        }

        // add "shared" prefix
        if matches!(self.metadata, TypeMetadata::Shared { .. }) {
            type_expr = TypeExpressionData::Shared(Box::new(type_expr.with_default_span()));
        }
        
        // add & or &mut
        if matches!(self.metadata, TypeMetadata::Local { ownership: LocalOwnership::Referenced(LocalReferenceMutability::Mutable), .. })
        {
            type_expr = TypeExpressionData::RefMut(Box::new(type_expr.with_default_span()));
        } else if matches!(self.metadata, TypeMetadata::Local { ownership: LocalOwnership::Referenced(LocalReferenceMutability::Immutable), .. }) {
            type_expr = TypeExpressionData::Ref(Box::new(type_expr.with_default_span()));
        }
        
        // TODO: ' and 'mut references

        type_expr
    }
}