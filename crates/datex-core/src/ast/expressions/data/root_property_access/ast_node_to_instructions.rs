use crate::{
    ast::expressions::RootPropertyAccess,
    compiler::{
        ast_node_to_instructions::AstNodeToInstructions,
        context::CompilationContext, error::CompilerError,
        metadata::CompileContextMetadata,
        precompiler::precompiled_ast::AstMetadata,
    },
    global::root_properties::RootProperty,
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
};
use core::str::FromStr;

impl AstNodeToInstructions for RootPropertyAccess {
    fn ast_node_to_instructions<'a>(
        &'a self,
        _compilation_context: &'a mut CompilationContext,
        _cache: &'a mut SharedReferencesCache,
        _ast_metadata: &'a mut AstMetadata,
        _ctx_metadata: CompileContextMetadata,
    ) -> Box<
        dyn Iterator<
                Item = Result<crate::instruction::BytecodeItem, CompilerError>,
            > + 'a,
    > {
        Box::new(gen move {
            let root_property = RootProperty::from_str(&self.property_name);

            if let Ok(root_property) = root_property {
                yield Ok(
                    RegularInstruction::get_root_property(root_property).into()
                );
            } else {
                yield Err(CompilerError::InvalidRootPropertyName(
                    self.property_name.clone(),
                ))
            }
        })
    }
}
