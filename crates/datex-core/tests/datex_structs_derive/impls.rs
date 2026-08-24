use core::{cell::Ref, ops::DerefMut};
use datex_core::{
    datex_proxy::{
        DatexProxyType, DatexValueContainerProxyDeserialize,
        DatexValueContainerProxySerialize,
    },
    datex_registry::{
        all_datex_module_registrations, all_datex_type_registrations, get_all_modules,
    },
    libs::core::type_id::{
        CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId,
    },
    runtime::Runtime,
    shared_values::{
        SelfOwnedPointerAddress, SharedContainer, SharedContainerMutability,
    },
    traits::apply::Apply,
    types::{
        entities::{
            entity_impls::{EntityImpl, EntityImplMethod},
            entity_type_definition::EntityTypeDefinition,
        },
        literal_type_definition::LiteralTypeDefinition,
        shared_container_containing_entity_type::SharedContainerContainingEntityType,
        r#type::Type,
        type_definition::{
            TypeDefinition,
            callable::{CallableKind, CallableTypeDefinition},
        },
    },
    values::{
        core_value::CoreValue,
        core_values::{
            integer::typed_integer::{IntegerTypeVariant, TypedInteger},
            native::DatexNative,
        },
        value::Value,
        value_container::ValueContainer,
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

    pub fn set_a(&mut self, a: u8) -> u8 {
        self.a = a;
        self.a
    }
    pub fn set_b(&mut self, b: u8) {
        self.b = b;
    }

    /// An example of a static method that adds two u8s together.
    pub fn add(a: u8, b: u8) -> u8 {
        a + b
    }
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
    let mut memory = runtime.shared_references_cache_refcell().borrow_mut();
    let example_type = Example::datex_type(memory.deref_mut());

    // when calling the datex_type function multiple times, it should return the same type definition from cache
    assert_eq!(example_type, Example::datex_type(memory.deref_mut()));
}

#[test]
fn signatures() {
    let runtime = Runtime::stub();
    let mut memory = runtime.shared_references_cache_refcell().borrow_mut();
    let example_type = Example::datex_type(memory.deref_mut());
    let type_definition = entity_type_definition_from_type(&example_type);

    // call static method by manually extracting it from the type definition
    match &example_type {
        Type::Entity(entity) => {
            let definition = entity.entity_definition();
            let static_add_method = &definition
                .try_get_property("add")
                .expect("Static add method not found");

            // call the static add method
            let result = static_add_method
                .try_apply_sync(
                    &Runtime::stub(),
                    vec![
                        TypedInteger::U8(1).into(),
                        TypedInteger::U8(2).into(),
                    ],
                )
                .unwrap()
                .0
                .unwrap();
            assert_eq!(
                result,
                ValueContainer::Local(Value::new(
                    CoreValue::native(3u8),
                    Some(TypeDefinition::core(CoreLibVariantTypeId::Integer(
                        IntegerTypeVariant::U8
                    )))
                ))
            );
        }
        _ => {
            panic!("Expected entity type, got {:?}", example_type);
        }
    }

    // call method directly on an Example instance via the ValueContainer
    let example_instance = Box::new(Example { a: 1, b: 2 });
    let example_instance_vc = example_instance
        .try_boxed_to_value_container(&mut memory)
        .unwrap();
    // TODO: also store type in value container (this will require passing cache to the ValueContainer::from function somehow)
    // Then we can access methods on the type definition here

    {
        // set_a
        let set_a_sig = &type_definition
            .try_get_property("set_a")
            .expect("set_a method not found")
            .signature;
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
                return_type: Some(Box::new(Type::core(
                    CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                        IntegerTypeVariant::U8
                    ))
                ))),
                yeet_type: None,
            }
        );
    }
    {
        // set_b
        let set_b_sig = &type_definition
            .try_get_property("set_b")
            .expect("set_b method not found")
            .signature;
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
        let new_sig = &type_definition
            .try_get_property("new")
            .expect("new method not found")
            .signature;
        assert_eq!(
            new_sig,
            &CallableTypeDefinition {
                kind: CallableKind::Function,
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
