//! This module contains the core library for DATEX, which provides the fundamental types and values that are essential for the operation of the DATEX language. The core library is loaded into memory at startup and is accessible globally throughout the system.
use crate::{
    libs::core::{
        core_lib_id::{CoreLibId, CoreLibIdTrait},
        type_id::{CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId},
        value_id::CoreLibValueId,
    },
    runtime::memory::Memory,
    shared_values::ReferencedSharedContainer,
    values::{
        core_value::CoreValue,
        core_values::{
            callable::{CallableBody, CallableKind, CallableSignature},
            map::Map,
        },
        value::Value,
        value_container::ValueContainer,
    },
};
use itertools::Itertools;

pub mod core_lib_id;
pub mod type_id;
pub mod value_id;

use crate::{
    libs::library::Library,
    prelude::*,
    shared_values::{ExternalPointerAddress, PointerAddress, SharedContainer},
    types::{
        shared_container_containing_nominal_type::SharedContainerContainingNominalType,
        r#type::Type,
    },
};
use log::info;
use strum::IntoEnumIterator;

pub struct CoreLibrary;

type CoreLibTypeDefinition = (CoreLibId, ValueContainer);

impl CoreLibrary {
    /// Loads the core library into the provided [Memory] instance.
    /// # Safety
    /// Caller must guarantee that the core library was not already loaded into the [Memory] instance
    pub unsafe fn load_core_lib(memory: &mut Memory) {
        unsafe {
            let entries = Self::generate_core_lib_vals(memory)
                .collect::<Vec<_>>()
                .into_iter()
                .map(|(id, reference)| {
                    memory.register_referenced_shared_container(&reference);
                    (
                        id,
                        ValueContainer::Shared(SharedContainer::Referenced(
                            reference.clone(),
                        )),
                    )
                })
                .chain(Self::generate_core_lib_types())
                .map(|(id, entry)| (id.name(), entry))
                .collect::<Vec<_>>();

            let core_struct = SharedContainer::Referenced(unsafe {
                ReferencedSharedContainer::new_immutable_external_with_inferred_allowed_type(
                    Map::from(entries).into(),
                    CoreLibValueId::Core.into(),
                    memory
                )
            });
            memory.register_referenced_shared_container(
                &core_struct.derive_immutable_reference(),
            );
        }
    }

    /// Returns a map of all core library type values by id
    fn generate_core_lib_types() -> impl Iterator<Item = CoreLibTypeDefinition>
    {
        gen {
            for id in CoreLibBaseTypeId::iter() {
                yield Self::create_type(CoreLibTypeId::Base(id));
                for variant_id in CoreLibVariantTypeId::variant_ids(&id) {
                    yield Self::create_type(CoreLibTypeId::Variant(variant_id));
                }
            }
        }
    }

    /// Returns a map of all core library values (excluding type values) by id
    unsafe fn generate_core_lib_vals(
        memory: &Memory,
    ) -> impl Iterator<Item = (CoreLibId, ReferencedSharedContainer)> {
        unsafe {
            gen {
                yield Self::print(memory);
            }
        }
    }

    /// Creates a new core lib type via definition and id
    fn create_type(id: CoreLibTypeId) -> CoreLibTypeDefinition {
        (
            CoreLibId::Type(id),
            ValueContainer::from(CoreValue::Type(Type::core(id))),
        )
    }

    unsafe fn print(memory: &Memory) -> (CoreLibId, ReferencedSharedContainer) {
        unsafe {
            (
            CoreLibId::Value(CoreLibValueId::Print),
            ReferencedSharedContainer::new_immutable_external_with_inferred_allowed_type(
                Value::callable(
                    Some("print".to_string()),
                    CallableSignature {
                        kind: CallableKind::Function,
                        parameter_types: vec![],
                        rest_parameter_type: Some((
                            Some("values".to_string()),
                            Box::new(Type::core(CoreLibBaseTypeId::Unknown)),
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
                memory
            ),
        )
        }
    }
}

impl Library for CoreLibrary {
    unsafe fn load(memory: &mut Memory) {
        unsafe { Self::load_core_lib(memory) }
    }
}

impl Memory {
    /// Helper function to get a core value directly from memory
    pub fn get_core_value_reference(
        &self,
        core_lib_value_id: CoreLibValueId,
    ) -> &ReferencedSharedContainer {
        let pointer_address = PointerAddress::from(core_lib_value_id);
        self.get_reference(&pointer_address).unwrap_or_else(|| {
            panic!("core reference not found in memory: {}", pointer_address)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        shared_values::PointerAddress, values::core_values::endpoint::Endpoint,
    };
    use core::str::FromStr;
    use itertools::Itertools;

    use super::*;

    #[test]
    fn debug() {
        let memory = Memory::new();
        info!(
            "{}",
            memory
                .get_core_value_reference(CoreLibValueId::Core)
                .value_container()
        );
    }

    #[ignore]
    #[test]
    #[cfg(feature = "std")]
    fn print_core_lib_addresses_as_hex() {
        for base_id in CoreLibBaseTypeId::iter() {
            println!("{:?}: {}", base_id, PointerAddress::from(base_id));
            for variant_id in CoreLibVariantTypeId::variant_ids(&base_id) {
                println!(
                    "{:?}: {}",
                    variant_id,
                    PointerAddress::from(variant_id)
                );
            }
        }
        for base_id in CoreLibValueId::iter() {
            println!("{:?}: {}", base_id, PointerAddress::from(base_id));
        }
    }

    #[test]
    #[ignore]
    #[cfg(feature = "std")]
    /// Generates a TypeScript mapping of core type addresses to their names.
    /// Run this test and copy the output into `src/dif/definitions.ts`.
    ///
    /// `cargo test create_core_type_ts_mapping -- --show-output --ignored`
    fn create_core_type_ts_mapping() {
        println!("export const CoreTypeAddress = {{");

        for base_id in CoreLibBaseTypeId::iter() {
            println!(
                "{}: \"{}\",",
                base_id,
                PointerAddress::from(base_id)
                    .to_string()
                    .strip_prefix("$")
                    .unwrap()
            );
            for variant_id in CoreLibVariantTypeId::variant_ids(&base_id) {
                println!(
                    "{}_{}: \"{}\",",
                    base_id,
                    variant_id.variant_name(),
                    PointerAddress::from(variant_id)
                        .to_string()
                        .strip_prefix("$")
                        .unwrap()
                );
            }
        }

        println!("}} as const;");
    }
}
