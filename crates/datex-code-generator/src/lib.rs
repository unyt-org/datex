pub mod rust_code_generator;

use core::fmt::Display;
use indexmap::IndexMap;
use datex_core::shared_wrappers::shared::shared_ref::SharedRef;
use datex_core::types::type_definition::TypeDefinition;
use datex_core::values::core_values::callable::Callable;
use datex_core::values::core_values::endpoint::Endpoint;
use datex_core::values::core_values::map::Map;
use datex_core::values::value_container::ValueContainer;

#[derive(Debug, Clone)]
pub struct ModuleGeneratorContext<Code> {
    imports: IndexMap<String, Code>,
}
impl<Code> ModuleGeneratorContext<Code> {
    pub fn new() -> Self {
        Self {
            imports: IndexMap::new(),
        }
    }

    pub fn add_import(&mut self, uuid: String, code: Code) {
        self.imports.insert(uuid, code);
    }

    pub fn imports(&self) -> &IndexMap<String, Code> {
        &self.imports
    }
}

#[derive(Default, Debug, Clone)]
pub struct GeneratedModules {
    pub modules: IndexMap<String, String>,
}

impl Display for GeneratedModules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, code) in &self.modules {
            writeln!(f, "/** module {} */\n{}\n\n", name, code)?;
        }
        Ok(())
    }
}


#[derive(Default, Debug, Clone)]
pub struct CodeGeneratorContext {
    generated_modules: GeneratedModules,
}

impl CodeGeneratorContext {
    pub fn generated_modules(&self) -> &IndexMap<String, String> {
        &self.generated_modules.modules
    }

    pub fn add_module(&mut self, name: String, code: String) {
        self.generated_modules.modules.insert(name, code);
    }

    pub fn into_generated_code(self) -> GeneratedModules {
        self.generated_modules
    }
}

pub trait CodeGenerator {
    type Code;

    fn code_to_string(&self, code: &Self::Code) -> String;
    fn join_code_statements(&self, statements: &[Self::Code]) -> Self::Code;

    /// Folds a static map definition, a.k.a. "module"
    fn visit_module(&mut self, ctx: &mut CodeGeneratorContext, name: &str, map: Map) {
        let mut module_context = ModuleGeneratorContext::<Self::Code>::new();
        let mut module_items = Vec::<Self::Code>::new();

        for (key, value) in map.iter() {
            if let Some(key) = key.try_as_str() {
                match value {
                    // fold different shared values
                    ValueContainer::Shared(shared) => {
                        if let Ok(callable) = SharedRef::<Callable>::try_from(shared.derive_reference_with_max_mutability()) {
                            module_items.push(self.fold_callable(&mut module_context, key, callable));
                        } else {
                           todo!()
                        }
                    }
                    // local constant values
                    ValueContainer::Local(value) => {
                        let ty = value.actual_type();
                        let type_code = self.fold_type_definition(ty, true);
                        let value_code = if let Some(string) = value.try_as_str() {
                            self.fold_text(string)
                        }
                        else {
                            todo!()
                        };
                        let constant_code = self.fold_constant_declaration(key, type_code, value_code);
                        module_items.push(constant_code);
                    }
                    _ => todo!()
                }
            }
            else {
                unimplemented!("Non-string keys in module maps are not supported yet");
            }
        }
        
        let imports_code = self.join_code_statements(&module_context.imports.into_values().collect::<Vec<_>>());
        let module_code = self.join_code_statements(&module_items);
        let code = self.join_code_statements(&[imports_code, module_code]);
        
        let code_str = self.code_to_string(&code);
        ctx.add_module(name.to_string(), code_str);
    }

    fn fold_callable(&mut self, module_context: &mut ModuleGeneratorContext<Self::Code>, name: &str, callable: SharedRef<Callable>) -> Self::Code;
    fn fold_constant_declaration(&mut self, name: &str, type_declaration: Self::Code, value: Self::Code) -> Self::Code;
    fn fold_text(&mut self, text: &str) -> Self::Code;
    fn fold_type_definition(&mut self, ty: TypeDefinition, is_const: bool) -> Self::Code;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CodeGenError {
    Other(String),
}


pub fn generate_glue_code_for_endpoint(
    endpoint: Endpoint, 
    generator: &mut impl CodeGenerator
) -> Result<IndexMap<String, String>, CodeGenError> {
    todo!("get type definition for endpoint and generate glue code")
}

pub fn generate_glue_code_for_module<G: CodeGenerator>(
    name: &str,
    map: Map,
    generator: &mut G
) -> Result<GeneratedModules, CodeGenError> {
    let mut context = CodeGeneratorContext::default();
    generator.visit_module(&mut context, name, map);
    Ok(context.into_generated_code())
}