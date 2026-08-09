use core::ops::DerefMut;
use datex_core::{
    datex_proxy::DatexProxyTypes,
    datex_registry::all_datex_registrations,
    decompiler::{DecompileOptions, decompile_value},
    runtime::Runtime,
};
use datex_core::libs::core::type_id::CoreLibBaseTypeId;
use datex_core::shared_values::{SelfOwnedPointerAddress, SharedContainer, SharedContainerMutability};
use datex_core::types::entity_type_definition::EntityTypeDefinition;
use datex_core::types::r#type::Type;
use datex_core::types::shared_container_containing_entity_type::SharedContainerContainingEntityType;
use datex_core::values::core_value::CoreValue;
use datex_macros_internal::{Datex, datex};

#[derive(Datex, Debug, Clone, PartialEq)]
// TODO: #[structural], nominal default
struct Example {
    a: u8,
    b: u8,
}

#[datex]
impl Example {
    pub fn set_a(&mut self, a: u8) {
        self.a = a;
    }
}

#[test]
fn impl_functions() {
    let runtime = Runtime::stub();
    let mut memory = runtime.memory().borrow_mut();

    let example_type = Example::datex_type(memory.deref_mut());
    println!(
        "{}",
        decompile_value(
            &example_type.into(),
            DecompileOptions::colorized_pretty()
        )
    );

    // Type::Entity(unsafe {
    //     SharedContainerContainingEntityType::new_base_with_address(
    //         name,
    //         SelfOwnedPointerAddress::new_static_from_name(namespace + name),
    //         Type::core(CoreLibBaseTypeId::Unknown)
    //     )
    // })

    println!("reg {:#?}", all_datex_registrations().collect::<Vec<_>>());

    let mut example = Example { a: 1, b: 2 };
    example.set_a(2);
}
