use datex_core::types::type_definition::callable::{CallableKind, CallableTypeDefinition};
use datex_core::values::core_values::callable::{Callable, CallableBody};
use datex_core::values::core_values::map::Map;
use datex_code_generator::generate_glue_code_for_module;
use datex_code_generator::rust_code_generator::RustCodeGenerator;
use datex_core::libs::core::type_id::CoreLibBaseTypeId;
use datex_core::runtime::Runtime;
use datex_core::shared_wrappers::shared::to_shared::ToShared;
use datex_core::traits::convert_value_container::ConvertValueContainer;
use datex_core::types::r#type::Type;

#[cfg(feature = "rust_codegen")]
#[test]
fn test() {
    let runtime = Runtime::stub();

    let datex_module = Map::from(vec![
        (
            "example_function",
            Callable {
                name: Some("example_function".to_string()),
                signature: CallableTypeDefinition {
                    kind: CallableKind::Function,
                    requires_async: false,
                    parameters: vec![
                        (
                            Some("param1".to_string()),
                            Type::core(CoreLibBaseTypeId::Text),
                        ),
                        (
                            Some("param2".to_string()),
                            Type::core(CoreLibBaseTypeId::Boolean),
                        ),
                    ],
                    rest_parameter: None,
                    return_type: Some(Box::new(Type::core(CoreLibBaseTypeId::Text))),
                    yeet_type: None,
                },
                body: CallableBody::Hidden,
                creator: Default::default(),
                docs: None,
            }.shared(&runtime).to_value_container()
        ),
        (
            "name",
            "Example Endpoint".to_string().to_value_container()
        )
    ]);

    let mut generator = RustCodeGenerator::default();
    let code = generate_glue_code_for_module("example", datex_module, &mut generator).unwrap();

    println!("modules:\n{}", code);
}