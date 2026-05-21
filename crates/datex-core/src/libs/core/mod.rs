//! This module contains the core library for DATEX, which provides the fundamental types and values that are essential for the operation of the DATEX language. The core library is loaded into memory at startup and is accessible globally throughout the system.
use crate::{
    libs::core::{
        core_lib_id::{CoreLibId, CoreLibIdTrait},
        type_id::{CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId},
        value_id::CoreLibValueId,
    },
    runtime::{execution::ExecutionError, memory::Memory},
    shared_values::{
        BuiltinPointerAddress, ExternalPointerAddress,
        ReferencedSharedContainer,
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

pub mod core_lib_id;
pub mod type_id;
pub mod value_id;

use crate::{
    libs::library::Library,
    prelude::*,
    shared_values::{PointerAddress, SharedContainer},
    types::r#type::Type,
};
use log::info;
use strum::IntoEnumIterator;

#[derive(Debug)]
pub struct CoreLibraryValues {
    print: Value,
}

impl Default for CoreLibraryValues {
    fn default() -> Self {
        CoreLibraryValues {
            print: Value::callable(
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
                CallableBody::Native(Self::print_impl),
            ),
        }
    }
}

impl CoreLibraryValues {
    /// Resolve a core library value by its id.
    pub fn get_value_by_id(&self, id: &CoreLibValueId) -> &Value {
        match id {
            CoreLibValueId::Print => &self.print,
        }
    }

    gen fn iterate(&self) -> (CoreLibValueId, &Value) {
        for id in CoreLibValueId::iter() {
            yield (id, self.get_value_by_id(&id));
        }
    }

    fn print_impl(
        mut args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
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
                    crate::decompiler::DecompileOptions::colorized(),
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
    }
}

#[derive(Debug)]
pub struct CoreLibrary {
    values: CoreLibraryValues,
    types: HashMap<CoreLibTypeId, Value>,
    map: Value,
}

impl Default for CoreLibrary {
    fn default() -> Self {
        let values = CoreLibraryValues::default();
        let types = Self::core_lib_types().collect::<HashMap<_, _>>();
        let entries = values
            .iterate()
            .map(|(id, value)| (CoreLibId::Value(id), value.clone()))
            .chain(
                types
                    .iter()
                    .map(|(id, value)| (CoreLibId::Type(*id), value.clone())),
            )
            .map(|(id, value)| (id.name(), ValueContainer::from(value)))
            .collect::<Vec<_>>();

        CoreLibrary {
            map: Value::from(Map::from(entries)),
            types,
            values,
        }
    }
}

type CoreLibTypeDefinition = (CoreLibTypeId, Value);

impl CoreLibrary {
    pub fn by_buitin_pointer_address(
        &self,
        address: &ExternalPointerAddress,
    ) -> Result<&Value, ()> {
        match CoreLibId::try_from(address)? {
            CoreLibId::Value(id) => Ok(self.values.get_value_by_id(&id)),
            CoreLibId::Type(id) => self.types.get(&id).ok_or(()),
            CoreLibId::Map => Ok(&self.map),
        }
    }

    pub fn map(&self) -> &Map {
        self.map.try_as().unwrap()
    }

    pub fn value_by_id(&self, id: &CoreLibValueId) -> &Value {
        self.values.get_value_by_id(id)
    }

    pub fn type_by_id(&self, id: &CoreLibTypeId) -> Option<&Value> {
        self.types.get(id)
    }

    /// Returns a map of all core library type values by id
    fn core_lib_types() -> impl Iterator<Item = CoreLibTypeDefinition> {
        gen {
            for id in CoreLibBaseTypeId::iter() {
                yield Self::create_type(CoreLibTypeId::Base(id));
                for variant_id in CoreLibVariantTypeId::variant_ids(&id) {
                    yield Self::create_type(CoreLibTypeId::Variant(variant_id));
                }
            }
        }
    }

    /// Creates a new core lib type via definition and id
    fn create_type(id: CoreLibTypeId) -> CoreLibTypeDefinition {
        (id, Value::from(CoreValue::Type(Type::core(id))))
    }
}

#[cfg(test)]
mod tests {
    use crate::shared_values::PointerAddress;

    use super::*;

    #[test]
    fn debug() {
        info!("{}", CoreLibrary::default().map());
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
