pub mod rust_code_generator;

use datex_core::ast::expressions::CallableDeclaration;
use datex_core::types::type_definition::map::MapTypeDefinition;
use datex_core::values::core_values::endpoint::Endpoint;

pub trait CodeGenerator {
    /// Folds a static map definition, a.k.a. "module"
    fn fold_static_map(map: MapTypeDefinition) {

    }

    fn fold_callable(callable: CallableDeclaration, access_path: Vec<String>) {

    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CodeGenError {
    Other(String),
}


pub fn generate_glue_code_for_endpoint(
    endpoint: Endpoint, 
    generator: &mut impl CodeGenerator
) -> Result<Vec<SourceFile>, CodeGenError> {
    todo!("get type definition for endpoint and generate glue code")
}

pub fn generate_glue_code_for_module(
    map: MapTypeDefinition, 
    generator: &mut impl CodeGenerator
) -> Result<Vec<SourceFile>, CodeGenError> {
    todo!()
}