//! This module contains the core library for DATEX, which provides the fundamental types and values that are essential for the operation of the DATEX language. The core library is loaded into memory at startup and is accessible globally throughout the system.
use crate::{
    libs::core::{
        core_lib_id::CoreLibId,
        type_id::{CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId},
        value_id::CoreLibValueId,
    },
    random::RandomState,
    types::type_definition::callable::CallableTypeDefinition,
    values::{
        core_value::CoreValue,
        core_values::{
            callable::{CallableBody, error::CallableError},
            map::Map,
        },
        value::Value,
        value_container::ValueContainer,
    },
};

pub mod core_lib_id;
pub mod type_id;
mod type_match;
pub mod value_id;

use crate::{
    prelude::*,
    types::{r#type::Type, type_definition::callable::CallableKind},
};
use indexmap::IndexMap;
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
                CallableTypeDefinition {
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
    pub fn get_by_id(&self, id: &CoreLibValueId) -> &Value {
        match id {
            CoreLibValueId::Print => &self.print,
        }
    }

    /// Iterate over all core library values, yielding their id and value.
    gen fn iterate(&self) -> (CoreLibValueId, &Value) {
        for id in CoreLibValueId::iter() {
            yield (id, self.get_by_id(&id));
        }
    }

    fn print_impl(
        mut args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, CallableError> {
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
pub struct CoreLibraryTypes(IndexMap<CoreLibTypeId, Value, RandomState>);

impl Default for CoreLibraryTypes {
    fn default() -> Self {
        Self(Self::core_lib_types().collect())
    }
}

impl CoreLibraryTypes {
    /// Resolve a core library type by its id and return it as a value.
    pub fn get_by_id(&self, id: &CoreLibTypeId) -> &Value {
        self.0.get(id).expect("Core library type not found")
    }

    /// Returns a map of all core library type values by id
    fn core_lib_types() -> impl Iterator<Item = (CoreLibTypeId, Value)> {
        gen {
            for id in CoreLibBaseTypeId::iter() {
                yield Self::create_type(CoreLibTypeId::Base(id));
                for variant_id in CoreLibVariantTypeId::variant_ids(&id) {
                    yield Self::create_type(CoreLibTypeId::Variant(variant_id));
                }
            }
        }
    }

    /// Iterate over all core library types, yielding their id and value.
    gen fn iterate(&self) -> (CoreLibTypeId, &Value) {
        for (id, value) in &self.0 {
            yield (*id, value);
        }
    }

    /// Creates a new core lib type via definition and id
    fn create_type(id: CoreLibTypeId) -> (CoreLibTypeId, Value) {
        (id, Value::from(CoreValue::Type(Type::core(id))))
    }
}

#[derive(Debug)]
pub struct CoreLibrary {
    values: CoreLibraryValues,
    types: CoreLibraryTypes,
    map: Value,
}

impl Default for CoreLibrary {
    fn default() -> Self {
        let values = CoreLibraryValues::default();
        let types = CoreLibraryTypes::default();
        let entries = values
            .iterate()
            .map(|(id, value)| (CoreLibId::Value(id), value.clone()))
            .chain(
                types
                    .iterate()
                    .map(|(id, value)| (CoreLibId::Type(id), value.clone())),
            )
            .map(|(id, value)| (id.to_string(), ValueContainer::from(value)))
            .collect::<Vec<_>>();

        CoreLibrary {
            map: Value::from(Map::from(entries)),
            types,
            values,
        }
    }
}

impl CoreLibrary {
    /// Resolves a pointer address to a core library value if it exists, otherwise returns an error.
    pub fn value_or_type_by_id(&self, id: CoreLibId) -> &Value {
        match id {
            CoreLibId::Value(id) => self.values.get_by_id(&id),
            CoreLibId::Type(id) => self.types.get_by_id(&id),
            CoreLibId::CoreMap => &self.map,
        }
    }

    /// Gets the core library map, which contains all core library values and types indexed by their id as strings.
    pub fn map(&self) -> &Map {
        self.map.try_as().unwrap()
    }

    /// Resolve a core library value by its id.
    pub fn value_by_id(&self, id: &CoreLibValueId) -> &Value {
        self.values.get_by_id(id)
    }

    /// Resolve a core library type by its id and return it as a value.
    pub fn type_by_id(&self, id: &CoreLibTypeId) -> &Value {
        self.types.get_by_id(id)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        libs::core::core_lib_id::CoreLibIdIndex, shared_values::PointerAddress,
    };

    use super::*;

    #[test]
    fn debug() {
        flexi_logger::init();
        info!("{}", CoreLibrary::default().map());
    }

    #[test]
    #[ignore]
    #[cfg(feature = "std")]
    /// Generates a TypeScript mapping of core type names to their ids
    /// Run this test and copy the output into `src/dif/definitions.ts`.
    ///
    /// `cargo test create_core_type_ts_mapping -- --show-output --ignored`
    fn create_core_type_ts_mapping() {
        println!("export const CoreLibTypeId = {{");

        for base_id in CoreLibBaseTypeId::iter() {
            println!("    {}: {},", base_id, CoreLibIdIndex::from(base_id).0);
            for variant_id in CoreLibVariantTypeId::variant_ids(&base_id) {
                println!(
                    "    {}_{}: {},",
                    base_id,
                    variant_id.variant_name(),
                    CoreLibIdIndex::from(variant_id).0
                );
            }
        }

        println!("}} as const;");
    }
}
