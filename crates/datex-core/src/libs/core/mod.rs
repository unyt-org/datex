use crate::{
    collections::HashMap,
    libs::core::{
        core_lib_id::{CoreLibId, CoreLibIdTrait},
        type_id::{CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId},
        value_id::CoreLibValueId,
    },
    runtime::memory::Memory,
    shared_values::shared_containers::{
        ReferencedSharedContainer, SharedContainerMutability,
    },
    types::{
        nominal_type_definition::NominalTypeDefinition,
        type_definition::TypeDefinition,
    },
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
    shared_values::{
        pointer_address::{ExternalPointerAddress, PointerAddress},
        shared_containers::SharedContainer,
    },
    types::{
        shared_container_containing_nominal_type::SharedContainerContainingNominalType,
        r#type::Type,
    },
};
use log::info;
use strum::IntoEnumIterator;

pub struct CoreLibrary;

type CoreLibTypeDefinition = (CoreLibId, ReferencedSharedContainer);
type CoreLibMap = HashMap<CoreLibId, ReferencedSharedContainer>;

impl CoreLibrary {
    /// Loads the core library into the provided [Memory] instance.
    /// Caller must guarantee that the core library was not already loaded into the [Memory] instance
    pub unsafe fn load_core_lib(memory: &mut Memory) {
        unsafe {
            let type_entries =
                Self::generate_core_lib_types(memory).collect::<Vec<_>>();
            for (_, type_entry) in type_entries.iter() {
                memory.register_referenced_shared_container(type_entry);
            }

            let value_entries =
                Self::generate_core_lib_vals(memory).collect::<Vec<_>>();
            for (_, value_entry) in value_entries.iter() {
                memory.register_referenced_shared_container(value_entry);
            }

            let entries = value_entries
                .iter()
                .chain(type_entries.iter())
                .map(|(id, entry)| {
                    (
                        id.name(),
                        ValueContainer::Shared(SharedContainer::Referenced(
                            entry.clone(),
                        )),
                    )
                })
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
    fn generate_core_lib_types(
        memory: &mut Memory,
    ) -> impl Iterator<Item = (CoreLibId, ReferencedSharedContainer)> {
        gen {
            for id in CoreLibBaseTypeId::iter() {
                let base_type_def =
                    unsafe { Self::create_core_type(id, memory) };
                let base_type_def_container =
                    SharedContainer::Referenced(base_type_def.1.clone());
                for variant_id in CoreLibVariantTypeId::variant_ids(&id) {
                    let (variant_id, variant_type) = unsafe {
                        Self::create_type(
                            NominalTypeDefinition::Variant {
                                definition_type: TypeDefinition::Internal
                                    .into(),
                                // Note: This is safe because we know that the base is a nominal type
                                base: unsafe {
                                    SharedContainerContainingNominalType::new_unchecked(
                                    base_type_def_container.clone(),
                                )
                                },
                                variant_name: variant_id.variant_name(),
                            },
                            CoreLibTypeId::Variant(variant_id),
                            memory,
                        )
                    };
                    yield (variant_id, variant_type);
                }

                yield (base_type_def.0, base_type_def.1);
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
    unsafe fn create_type(
        definition: NominalTypeDefinition,
        id: CoreLibTypeId,
        memory: &Memory,
    ) -> CoreLibTypeDefinition {
        let core_lib_id = CoreLibId::Type(id);
        (core_lib_id, unsafe {
            ReferencedSharedContainer::try_new_external(
                ValueContainer::from(CoreValue::NominalTypeDefinition(
                    definition,
                )),
                id.into(),
                SharedContainerMutability::Immutable,
                Type::from(TypeDefinition::Type),
                memory,
            )
            .unwrap()
        })
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
                            Box::new(memory.get_core_type(CoreLibBaseTypeId::Unknown)),
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

    /// Creates a core type with a given pointer id, the nominal name is derived from the id's to_string() method.
    unsafe fn create_core_type(
        pointer_id: CoreLibBaseTypeId,
        memory: &mut Memory,
    ) -> CoreLibTypeDefinition {
        unsafe {
            Self::create_type(
                NominalTypeDefinition::Base {
                    definition_type: TypeDefinition::Internal.into(),
                    name: pointer_id.to_string(),
                },
                CoreLibTypeId::Base(pointer_id),
                memory,
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
    pub fn get_core_reference(
        &self,
        core_lib_id: impl Into<CoreLibId>,
    ) -> &ReferencedSharedContainer {
        let pointer_address = PointerAddress::from(core_lib_id.into());
        self.get_reference(&pointer_address).unwrap_or_else(|| {
            panic!("core reference not found in memory: {}", pointer_address)
        })
    }

    /// Helper function to get a [SharedContainerContainingNominalType] directly from memory
    /// by [CoreLibTypeId]
    pub fn get_core_type_reference(
        &self,
        id: impl Into<CoreLibTypeId>,
    ) -> SharedContainerContainingNominalType {
        unsafe {
            SharedContainerContainingNominalType::new_unchecked(
                SharedContainer::Referenced(
                    self.get_core_reference(CoreLibId::Type(id.into())).clone(),
                ),
            )
        }
    }

    /// Helper function to get a [Type::Nominal] directly from memory by [CoreLibTypeId]
    pub fn get_core_type(&self, id: impl Into<CoreLibTypeId>) -> Type {
        Type::Nominal(self.get_core_type_reference(id.into()))
    }
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
        let memory = Memory::new();
        info!(
            "{}",
            memory
                .get_core_reference(CoreLibValueId::Core)
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
