use crate::ast::expressions::CallableDeclaration;
use crate::types::type_definition::map::MapTypeDefinition;
use crate::values::core_values::map::Map;

pub trait CodeGenerator {
    /// Folds a static map definition, a.k.a. "module"
    fn fold_static_map(map: MapTypeDefinition) {

    }

    fn fold_callable(callable: CallableDeclaration, access_path: Vec<String>) {

    }
}