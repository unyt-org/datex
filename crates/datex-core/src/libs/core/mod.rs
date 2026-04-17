use crate::{
    collections::HashMap,
    libs::core::{
        core_lib_id::{CoreLibId, CoreLibIdTrait},
        type_id::{CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId},
        value_id::CoreLibValueId,
    },
    runtime::memory::Memory,
    shared_values::shared_containers::{
        OwnedSharedContainer, ReferencedSharedContainer,
        SharedContainerMutability,
    },
    types::{
        nominal_type_definition::NominalTypeDefinition,
        structural_type_definition::TypeDefinition,
    },
    values::{
        core_value::CoreValue,
        core_values::{
            callable::{CallableBody, CallableKind, CallableSignature},
            decimal::typed_decimal::DecimalTypeVariant,
            integer::typed_integer::IntegerTypeVariant,
            map::Map,
        },
        value::Value,
        value_container::ValueContainer,
    },
};

pub mod core_lib_id;
pub mod type_id;
pub mod value_id;

use crate::{
    prelude::*,
    shared_values::{
        pointer_address::ExternalPointerAddress,
        shared_containers::SharedContainer,
    },
    types::{
        shared_container_containing_type::SharedContainerContainingType,
        r#type::Type,
    },
};
use core::iter::once;
use log::info;
use strum::IntoEnumIterator;

type CoreLib = HashMap<CoreLibId, ReferencedSharedContainer>;

#[cfg_attr(not(feature = "embassy_runtime"), thread_local)]
pub static mut CORE_LIB: Option<CoreLib> = None;

fn with_core_lib<R>(handler: impl FnOnce(&CoreLib) -> R) -> R {
    unsafe {
        if CORE_LIB.is_none() {
            CORE_LIB.replace(create_core_lib());
        }
        handler(CORE_LIB.as_ref().unwrap_unchecked())
    }
}

pub fn core_lib_type(id: CoreLibTypeId) -> SharedContainerContainingType {
    with_core_lib(|entries| unsafe {
        SharedContainerContainingType::new_unchecked(
            SharedContainer::Referenced(
                entries.get(&CoreLibId::Type(id)).unwrap().clone(),
            ),
        )
    })
}

/// Retrieves either a core library type or value by its CoreLibPointerId.
pub fn core_lib_entry(id: CoreLibId) -> ReferencedSharedContainer {
    with_core_lib(|entries| {
        entries.get(&id).expect("Core lib value not found").clone()
    })
}

/// Loads the core library into the provided memory instance.
pub fn load_core_lib(memory: &mut Memory) {
    with_core_lib(|entries| {
        let mut mapping = entries
            .iter()
            .map(|(id, entry)| {
                memory.register_referenced_shared_container(
                    &entry.clone().into(),
                );
                (
                    id.name(),
                    ValueContainer::Shared(SharedContainer::Referenced(
                        entry.clone(),
                    )),
                )
            })
            .collect::<Vec<(String, ValueContainer)>>();
        let core_struct = SharedContainer::Referenced(
            ReferencedSharedContainer::new_immutable_external(
                Map::from(mapping).into(),
                CoreLibValueId::Core.into(),
            ),
        );
        memory.register_referenced_shared_container(
            &core_struct.derive_immutable_reference(),
        );
    });
}

pub fn create_core_lib() -> CoreLib {
    create_core_lib_types()
        .into_iter()
        .chain(create_core_lib_vals())
        .collect()
}

/// Returns a map of all core library type values by id
pub fn create_core_lib_types() -> CoreLib {
    CoreLibBaseTypeId::iter()
        .flat_map(|id| {
            let base_type_def = create_core_type(id);
            let base_type_def_container =
                SharedContainer::Referenced(base_type_def.1.clone());
            CoreLibVariantTypeId::variant_ids(&id)
                .into_iter()
                .map(move |variant_id| {
                    create_type(
                        NominalTypeDefinition::Variant {
                            definition: TypeDefinition::Unit.into(),
                            // Note: This is safe because we know that the base is a type
                            base: unsafe {
                                SharedContainerContainingType::new_unchecked(
                                    base_type_def_container.clone(),
                                )
                            },
                            variant_name: variant_id.variant_name(),
                        },
                        CoreLibTypeId::Variant(variant_id),
                    )
                })
                .chain(once(base_type_def))
        })
        .collect::<HashMap<CoreLibId, ReferencedSharedContainer>>()
}

/// Returns a map of all core library values (excluding type values) by id
pub fn create_core_lib_vals() -> HashMap<CoreLibId, ReferencedSharedContainer> {
    vec![print()]
        .into_iter()
        .collect::<HashMap<CoreLibId, ReferencedSharedContainer>>()
}

type CoreLibTypeDefinition = (CoreLibId, ReferencedSharedContainer);

/// Creates a new core lib type via definition and id
pub fn create_type(
    definition: NominalTypeDefinition,
    id: CoreLibTypeId,
) -> CoreLibTypeDefinition {
    let core_lib_id = CoreLibId::Type(id);
    (
        core_lib_id,
        ReferencedSharedContainer::new_immutable_external(
            Type::Nominal(definition).into(),
            id.into(),
        ),
    )
}

pub fn print() -> (CoreLibId, ReferencedSharedContainer) {
    (
        CoreLibId::Value(CoreLibValueId::Print),
        ReferencedSharedContainer::new_immutable_external(
            Value::callable(
                Some("print".to_string()),
                CallableSignature {
                    kind: CallableKind::Function,
                    parameter_types: vec![],
                    rest_parameter_type: Some((
                        Some("values".to_string()),
                        Box::new(Type::Alias(TypeDefinition::Unknown.into())),
                    )),
                    return_type: None,
                    yeet_type: None,
                },
                CallableBody::Native(|mut args: &[ValueContainer]| {
                    // TODO #680: add I/O abstraction layer / interface

                    let mut output = String::new();

                    // if first argument is a string value, print it directly
                    if let Some(ValueContainer::Local(Value {
                        inner: CoreValue::Text(text),
                        ..
                    })) = args.first()
                    {
                        output.push_str(&text.0);
                        // remove first argument from args
                        args = &args[1..];
                        // if there are still arguments, add a space
                        if !args.is_empty() {
                            output.push(' ');
                        }
                    }

                    #[cfg(feature = "decompiler")]
                    let args_string = args
                        .iter()
                        .map(|v| {
                            crate::decompiler::decompile_value(
                                v,
                                crate::decompiler::DecompileOptions::colorized(
                                ),
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    #[cfg(not(feature = "decompiler"))]
                    let args_string = args
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(" ");
                    output.push_str(&args_string);

                    #[cfg(feature = "std")]
                    println!("[PRINT] {}", output);
                    info!("[PRINT] {}", output);
                    Ok(None)
                }),
            )
            .into(),
            ExternalPointerAddress::from(CoreLibValueId::Print),
        ),
    )
}

/// Creates a core type with a given pointer id, the nominal name is derived from the id's to_string() method.
fn create_core_type(pointer_id: CoreLibBaseTypeId) -> CoreLibTypeDefinition {
    create_type(
        NominalTypeDefinition::Base {
            definition: TypeDefinition::Unit.into(),
            name: pointer_id.to_string(),
        },
        CoreLibTypeId::Base(pointer_id),
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        shared_values::pointer_address::PointerAddress,
        values::core_values::endpoint::Endpoint,
    };
    use core::str::FromStr;
    use itertools::Itertools;

    use super::*;

    #[test]
    fn debug() {
        let mut memory = Memory::new(Endpoint::LOCAL);
        load_core_lib(&mut memory);
        info!(
            "{}",
            memory
                .get_core_reference(CoreLibValueId::Core.into())
                .value_container()
        );
    }

    #[ignore]
    #[test]
    #[cfg(feature = "std")]
    fn print_core_lib_addresses_as_hex() {
        with_core_lib(|core_lib_types| {
            let sorted_entries = core_lib_types
                .keys()
                .map(|k| (k.clone(), PointerAddress::from(k.clone())))
                .sorted_by_key(|(_, address)| address.bytes().to_vec())
                .collect::<Vec<_>>();
            for (core_lib_id, address) in sorted_entries {
                println!("{:?}: {}", core_lib_id, address);
            }
        });
    }

    #[test]
    #[ignore]
    #[cfg(feature = "std")]
    /// Generates a TypeScript mapping of core type addresses to their names.
    /// Run this test and copy the output into `src/dif/definitions.ts`.
    ///
    /// `cargo test create_core_type_ts_mapping -- --show-output --ignored`
    fn create_core_type_ts_mapping() {
        let core_lib = create_core_lib_types();
        let mut core_lib: Vec<(CoreLibId, PointerAddress)> = core_lib
            .keys()
            .map(|key| (key.clone(), PointerAddress::from(key.clone())))
            .collect();
        core_lib.sort_by_key(|(key, _)| {
            PointerAddress::from(key.clone()).bytes().to_vec()
        });

        println!("export const CoreTypeAddress = {{");
        for (core_lib_id, address) in core_lib {
            println!(
                "    {}: \"{}\",",
                core_lib_id.to_string().replace("/", "_"),
                address.to_string().strip_prefix("$").unwrap()
            );
        }
        println!("}} as const;");
    }
}
