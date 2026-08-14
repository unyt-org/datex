use core::{cell::Ref, ops::DerefMut};
use datex_core::{
    datex_proxy::{DatexProxyTypes, DatexValueContainerProxyDeserialize},
    datex_registry::{
        all_datex_impl_registrations, all_datex_type_registrations, get_impls,
    },
    libs::core::type_id::{
        CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId,
    },
    runtime::Runtime,
    shared_values::{
        SelfOwnedPointerAddress, SharedContainer, SharedContainerMutability,
    },
    types::{
        entities::{
            entity_impls::{EntityImpl, EntityImplMethod},
            entity_type_definition::EntityTypeDefinition,
        },
        shared_container_containing_entity_type::SharedContainerContainingEntityType,
        r#type::Type,
        type_definition::callable::{CallableKind, CallableTypeDefinition},
    },
    values::{
        core_value::CoreValue,
        core_values::integer::typed_integer::IntegerTypeVariant,
    },
};
use datex_macros_internal::{Datex, datex};

#[derive(Datex, Debug, Clone, PartialEq)]
struct Example {
    a: u8,
    b: u8,
}

#[datex]
impl Example {
    pub fn new(a: u8, b: u8) -> Self {
        Example { a, b }
    }
    pub fn set_a(&mut self, a: u8) {
        self.a = a;
    }
    pub fn set_b(&mut self, b: u8) {
        self.b = b;
    }
}

/// Helper function to get a reference to the [EntityImplMethod] for a given type name from a list of [EntityImpl]s.
fn get_impls_for_type<'a>(
    impls: &'a [EntityImpl],
    type_name: &str,
) -> &'a EntityImplMethod {
    impls
        .iter()
        .flat_map(|entity_impl| entity_impl.methods.iter())
        .filter(|method| {
            method.name().map(|name| name == type_name).unwrap_or(false)
        })
        .next()
        .expect("Method not found")
}

/// Helper function to extract the [EntityTypeDefinition] from a given [Type].
fn entity_type_definition_from_type(
    ty: &Type,
) -> Ref<'_, EntityTypeDefinition> {
    match ty {
        Type::Entity(entity) => entity.entity_definition(),
        _ => panic!("Expected entity type, got {:?}", ty),
    }
}

#[test]
fn take_from_cache() {
    let runtime = Runtime::stub();
    let mut memory = runtime.memory().borrow_mut();
    let example_type = Example::datex_type(memory.deref_mut());

    // when calling the datex_type function multiple times, it should return the same type definition from cache
    assert_eq!(example_type, Example::datex_type(memory.deref_mut()));
}

#[test]
fn signatures() {
    let runtime = Runtime::stub();
    let mut memory = runtime.memory().borrow_mut();
    let example_type = Example::datex_type(memory.deref_mut());
    let type_definition = entity_type_definition_from_type(&example_type);

    {
        // set_a
        let set_a_sig =
            get_impls_for_type(type_definition.impls(), "set_a").signature();
        assert_eq!(
            set_a_sig,
            &CallableTypeDefinition {
                kind: CallableKind::Procedure,
                requires_async: false,
                parameters: vec![(
                    Some("a".to_string()),
                    Type::core(CoreLibTypeId::Variant(
                        CoreLibVariantTypeId::Integer(IntegerTypeVariant::U8)
                    ))
                ),],
                rest_parameter: None,
                return_type: None,
                yeet_type: None,
            }
        );
    }
    {
        // set_b
        let set_b_sig =
            get_impls_for_type(type_definition.impls(), "set_b").signature();
        assert_eq!(
            set_b_sig,
            &CallableTypeDefinition {
                kind: CallableKind::Procedure,
                requires_async: false,
                parameters: vec![(
                    Some("b".to_string()),
                    Type::core(CoreLibTypeId::Variant(
                        CoreLibVariantTypeId::Integer(IntegerTypeVariant::U8)
                    ))
                )],
                rest_parameter: None,
                return_type: None,
                yeet_type: None,
            }
        );
    }
    {
        // new
        let new_sig =
            get_impls_for_type(type_definition.impls(), "new").signature();
        assert_eq!(
            new_sig,
            &CallableTypeDefinition {
                kind: CallableKind::Procedure,
                requires_async: false,
                parameters: vec![
                    (
                        Some("a".to_string()),
                        Type::core(CoreLibTypeId::Variant(
                            CoreLibVariantTypeId::Integer(
                                IntegerTypeVariant::U8
                            )
                        ))
                    ),
                    (
                        Some("b".to_string()),
                        Type::core(CoreLibTypeId::Variant(
                            CoreLibVariantTypeId::Integer(
                                IntegerTypeVariant::U8
                            )
                        ))
                    )
                ],
                rest_parameter: None,
                return_type: Some(Box::new(example_type.clone())),
                yeet_type: None,
            }
        );
    }
}
