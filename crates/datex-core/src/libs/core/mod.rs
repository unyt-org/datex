use crate::{
    collections::HashMap,
    libs::core::{
        core_lib_id::{CoreLibId, CoreLibIdTrait},
        type_id::CoreLibBaseTypeId,
        value_id::CoreLibValueId,
    },
    runtime::memory::Memory,
    shared_values::{
        pointer_address::SelfOwnedPointerAddress,
        shared_containers::{
            OwnedSharedContainer, ReferencedSharedContainer,
            SharedContainerMutability,
        },
    },
    types::{
        nominal_type_definition::NominalTypeDefinition,
        structural_type_definition::StructuralTypeDefinition,
        type_definition::TypeDefinition,
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

pub fn core_lib_type(id: CoreLibBaseTypeId) -> SharedContainerContainingType {
    with_core_lib(|entries| unsafe {
        SharedContainerContainingType::new_unchecked(
            SharedContainer::Referenced(
                entries.get(&CoreLibId::Type(id)).unwrap().clone(),
            ),
        )
    })
}

/// Retrieves either a core library type or value by its CoreLibPointerId.
pub fn core_lib_value(id: CoreLibValueId) -> ReferencedSharedContainer {
    let id = id.into();
    with_core_lib(|entries| {
        entries
            .get(&CoreLibId::Value(id))
            .expect("Core lib value not found")
            .clone()
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
                (id.name(), entry)
            })
            .collect::<Vec<(String, ValueContainer)>>();
        let core_struct = SharedContainer::new_owned_with_inferred_allowed_type(
            ValueContainer::Local(Value::from(mapping)),
            SharedContainerMutability::Immutable,
            CoreLibValueId::Core.into(),
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

/// Creates a new instance of the core library as a ValueContainer
/// including all core types as properties.
pub fn create_core_lib_types() -> CoreLib {
    let integer = create_core_type(CoreLibBaseTypeId::Integer(None));
    let decimal = create_core_type(CoreLibBaseTypeId::Decimal(None));
    vec![
        create_core_type(CoreLibBaseTypeId::Endpoint),
        create_core_type(CoreLibBaseTypeId::Null),
        create_core_type(CoreLibBaseTypeId::Boolean),
        integer.clone(),
        decimal.clone(),
        create_core_type(CoreLibBaseTypeId::Type),
        create_core_type(CoreLibBaseTypeId::Text),
        create_core_type(CoreLibBaseTypeId::List),
        create_core_type(CoreLibBaseTypeId::Map),
        create_core_type(CoreLibBaseTypeId::Range),
        create_core_type(CoreLibBaseTypeId::Callable),
        create_core_type(CoreLibBaseTypeId::Unit),
        create_core_type(CoreLibBaseTypeId::Never),
        create_core_type(CoreLibBaseTypeId::Unknown),
    ]
    .into_iter()
    .chain(once(integer.clone()))
    .chain(
        IntegerTypeVariant::iter()
            .map(|variant| integer_variant(integer.1.clone(), variant)),
    )
    .chain(once(decimal.clone()))
    .chain(
        DecimalTypeVariant::iter()
            .map(|variant| decimal_variant(decimal.1.clone(), variant)),
    )
    .collect::<HashMap<CoreLibId, ReferencedSharedContainer>>()
}

pub fn create_core_lib_vals() -> HashMap<CoreLibId, ReferencedSharedContainer> {
    vec![print()]
        .into_iter()
        .collect::<HashMap<CoreLibId, ReferencedSharedContainer>>()
}

type CoreLibTypeDefinition = (CoreLibId, ReferencedSharedContainer);

pub fn decimal_variant(
    base: SharedContainerContainingType,
    variant: DecimalTypeVariant,
) -> CoreLibTypeDefinition {
    create_type(
        NominalTypeDefinition::Variant {
            definition: StructuralTypeDefinition::Unit.into(),
            base,
            variant_name: variant.to_string(),
        },
        CoreLibBaseTypeId::Decimal(Some(variant)),
    )
}

/// Creates a new core lib type via definition and id
pub fn create_type(
    definition: NominalTypeDefinition,
    id: CoreLibBaseTypeId,
) -> CoreLibTypeDefinition {
    let core_lib_id = CoreLibId::Type(id);
    (
        core_lib_id,
        ReferencedSharedContainer::new_immutable_external(
            Type::Nominal(definition).into(),
            ExternalPointerAddress::Builtin(id.to_bytes()),
        ),
    )
}

pub fn integer_variant(
    base: SharedContainerContainingType,
    variant: IntegerTypeVariant,
) -> CoreLibTypeDefinition {
    create_type(
        NominalTypeDefinition::Variant {
            definition: StructuralTypeDefinition::Unit.into(),
            base,
            variant_name: variant.to_string(),
        },
        CoreLibBaseTypeId::Integer(Some(variant)),
    )
}

pub fn print() -> (CoreLibValueId, ReferencedSharedContainer) {
    (
        CoreLibValueId::Print,
        ReferencedSharedContainer::new_immutable_external(
            Value::callable(
                Some("print".to_string()),
                CallableSignature {
                    kind: CallableKind::Function,
                    parameter_types: vec![],
                    rest_parameter_type: Some((
                        Some("values".to_string()),
                        Box::new(Type::unknown()),
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
            ExternalPointerAddress::from(&CoreLibValueId::Print),
        ),
    )
}

/// Creates a core type with a given pointer id, the nominal name is derived from the id's to_string() method.
fn create_core_type(pointer_id: CoreLibBaseTypeId) -> CoreLibTypeDefinition {
    create_type(
        NominalTypeDefinition::Base {
            definition: StructuralTypeDefinition::Unit.into(),
            name: pointer_id.to_string(),
        },
        pointer_id,
    )
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::values::core_values::endpoint::Endpoint;

    use super::*;

    use itertools::Itertools;

    #[test]
    fn core_lib() {
        assert!(has_core_lib_type(CoreLibBaseTypeId::Endpoint));
        assert!(has_core_lib_type(CoreLibBaseTypeId::Null));
        assert!(has_core_lib_type(CoreLibBaseTypeId::Boolean));
        assert!(has_core_lib_type(CoreLibBaseTypeId::Integer(None)));
        assert!(has_core_lib_type(CoreLibBaseTypeId::Decimal(None)));
        assert!(has_core_lib_type(CoreLibBaseTypeId::Type));
        assert!(has_core_lib_type(CoreLibBaseTypeId::Text));
        assert!(has_core_lib_type(CoreLibBaseTypeId::List));
        assert!(has_core_lib_type(CoreLibBaseTypeId::Map));
        assert!(has_core_lib_type(CoreLibBaseTypeId::Range));
        assert!(has_core_lib_type(CoreLibBaseTypeId::Callable));
        assert!(has_core_lib_type(CoreLibBaseTypeId::Unit));
        assert!(has_core_lib_type(CoreLibBaseTypeId::Never));
        assert!(has_core_lib_type(CoreLibBaseTypeId::Unknown));
        for variant in IntegerTypeVariant::iter() {
            assert!(has_core_lib_type(CoreLibBaseTypeId::Integer(Some(
                variant
            ))));
        }
        for variant in DecimalTypeVariant::iter() {
            assert!(has_core_lib_type(CoreLibBaseTypeId::Decimal(Some(
                variant
            ))));
        }
    }

    #[test]
    fn debug() {
        let mut memory = Memory::new(Endpoint::LOCAL);
        load_core_lib(&mut memory);
        info!(
            "{}",
            memory
                .get_value_reference(&CoreLibBaseTypeId::Core.into())
                .unwrap()
                .value_container
        );
    }

    #[test]
    fn core_lib_type_addresses() {
        let integer_base = "integer";
        let integer_u8 = "integer/u8";
        let integer_i32 = "integer/i32";
        let decimal_base = "decimal";
        let decimal_f64 = "decimal/f64";

        assert_eq!(
            CoreLibBaseTypeId::from_str(integer_base),
            Ok(CoreLibBaseTypeId::Integer(None))
        );
        assert_eq!(
            CoreLibBaseTypeId::from_str(integer_u8),
            Ok(CoreLibBaseTypeId::Integer(Some(IntegerTypeVariant::U8)))
        );
        assert_eq!(
            CoreLibBaseTypeId::from_str(integer_i32),
            Ok(CoreLibBaseTypeId::Integer(Some(IntegerTypeVariant::I32)))
        );
        assert_eq!(
            CoreLibBaseTypeId::from_str(decimal_base),
            Ok(CoreLibBaseTypeId::Decimal(None))
        );
        assert_eq!(
            CoreLibBaseTypeId::from_str(decimal_f64),
            Ok(CoreLibBaseTypeId::Decimal(Some(DecimalTypeVariant::F64)))
        );

        assert_eq!(CoreLibBaseTypeId::Integer(None).to_string(), integer_base);
        assert_eq!(
            CoreLibBaseTypeId::Integer(Some(IntegerTypeVariant::U8))
                .to_string(),
            integer_u8
        );
        assert_eq!(
            CoreLibBaseTypeId::Integer(Some(IntegerTypeVariant::I32))
                .to_string(),
            integer_i32
        );
        assert_eq!(CoreLibBaseTypeId::Decimal(None).to_string(), decimal_base);
        assert_eq!(
            CoreLibBaseTypeId::Decimal(Some(DecimalTypeVariant::F64))
                .to_string(),
            decimal_f64
        );
    }

    #[test]
    fn core_lib_pointer_id_conversion() {
        let core_id = CoreLibBaseTypeId::Core;
        let pointer_address: PointerAddress = core_id.clone().into();
        let converted_id: CoreLibBaseTypeId =
            (&pointer_address).try_into().unwrap();
        assert_eq!(core_id, converted_id);

        let boolean_id = CoreLibBaseTypeId::Boolean;
        let pointer_address: PointerAddress = boolean_id.clone().into();
        let converted_id: CoreLibBaseTypeId =
            (&pointer_address).try_into().unwrap();
        assert_eq!(boolean_id, converted_id);

        let integer_id =
            CoreLibBaseTypeId::Integer(Some(IntegerTypeVariant::I32));
        let pointer_address: PointerAddress = integer_id.clone().into();
        let converted_id: CoreLibBaseTypeId =
            (&pointer_address).try_into().unwrap();
        assert_eq!(integer_id, converted_id);

        let decimal_id =
            CoreLibBaseTypeId::Decimal(Some(DecimalTypeVariant::F64));
        let pointer_address: PointerAddress = decimal_id.clone().into();
        let converted_id: CoreLibBaseTypeId =
            (&pointer_address).try_into().unwrap();
        assert_eq!(decimal_id, converted_id);

        let type_id = CoreLibBaseTypeId::Type;
        let pointer_address: PointerAddress = type_id.clone().into();
        let converted_id: CoreLibBaseTypeId =
            (&pointer_address).try_into().unwrap();
        assert_eq!(type_id, converted_id);
    }

    #[test]
    fn base_type_simple() {
        // integer -> integer -> integer ...
        let integer_type = core_lib_type(CoreLibBaseTypeId::Integer(None));
        let integer_base = integer_type.base_type_reference();
        assert_eq!(integer_base.unwrap().borrow().to_string(), "integer");
    }

    #[test]
    fn base_type_complex() {
        // integer/u8 -> integer -> integer -> integer ...
        let integer_u8_type = core_lib_type(CoreLibBaseTypeId::Integer(Some(
            IntegerTypeVariant::U8,
        )));
        assert_eq!(integer_u8_type.to_string(), "integer/u8");

        let integer = integer_u8_type.base_type_reference();
        assert_eq!(integer.unwrap().borrow().to_string(), "integer");
    }

    #[ignore]
    #[test]
    #[cfg(feature = "std")]
    fn print_core_lib_addresses_as_hex() {
        with_core_lib(|core_lib_types, _| {
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
        let mut core_lib: Vec<(CoreLibBaseTypeId, PointerAddress)> = core_lib
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
