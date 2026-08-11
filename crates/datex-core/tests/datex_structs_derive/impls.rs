use core::ops::DerefMut;
use datex_core::{
    datex_proxy::{DatexProxyTypes, DatexValueContainerProxyDeserialize},
    datex_registry::{
        all_datex_impl_registrations, all_datex_type_registrations, get_impls,
    },
    runtime::Runtime,
    shared_values::{
        SelfOwnedPointerAddress, SharedContainer, SharedContainerMutability,
    },
    types::{
        entities::entity_type_definition::EntityTypeDefinition,
        shared_container_containing_entity_type::SharedContainerContainingEntityType,
        r#type::Type,
    },
    values::core_value::CoreValue,
};
use datex_macros_internal::{Datex, datex};

#[derive(Datex, Debug, Clone, PartialEq)]
struct Example {
    a: u8,
    b: u8,
}

#[datex]
impl Example {
    pub fn set_a(&mut self, a: u8, string: String) {
        self.a = a;
    }
    pub fn set_b(&mut self, b: u8) {
        self.b = b;
    }
}

#[cfg(feature = "decompiler")]
#[test]
fn impl_functions() {
    use datex_core::decompiler::{DecompileOptions, decompile_value};

    let runtime = Runtime::stub();
    let mut memory = runtime.memory().borrow_mut();

    let example_type = Example::datex_type(memory.deref_mut());
    println!(
        "{}",
        decompile_value(
            &example_type.clone().into(),
            DecompileOptions::colorized_pretty()
        )
    );

    // when calling the datex_type function multiple times, it should return the same type definition from cache
    assert_eq!(example_type, Example::datex_type(memory.deref_mut()));

    match example_type {
        Type::Entity(entity) => {
            println!(
                "regs: {:#?}",
                all_datex_impl_registrations().collect::<Vec<_>>()
            );
            let definition = entity.entity_definition();
            println!("impls: {:#?}", definition.impls());
        }
        _ => {
            panic!("Expected entity type, got {:?}", example_type);
        }
    }

    let mut example = Example { a: 1, b: 2 };
    example.set_a(2, "test".to_string());
    example.set_b(1);
}
