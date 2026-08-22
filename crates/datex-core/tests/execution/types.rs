use datex_core::{
    compiler::{CompileOptions, compile_template, scope::CompilationScope},
    disassembler::{
        options::DisassemblerOptions, print_disassembled_with_options,
    },
    libs::core::type_id::CoreLibTypeId,
    runtime::{
        Runtime,
        execution::{
            ExecutionInput, ExecutionOptions, execute_dxb_sync,
            execution_input::ExecutionCallerMetadata,
        },
        pointer_address_provider::SelfOwnedPointerAddressProvider,
    },
    shared_values::{
        PointerAddress, SharedContainer, SharedContainerMutability,
    },
    types::{
        literal_type_definition::LiteralTypeDefinition,
        shared_container_containing_type::SharedContainerContainingType,
        r#type::Type,
        type_definition::{
            TypeDefinition,
            callable::{CallableKind, CallableTypeDefinition},
            collection::{
                CollectionTypeDefinition,
                type_definition::{
                    list::ListCollectionTypeDefinition,
                    map::MapCollectionTypeDefinition,
                },
            },
            impl_type::ImplTypeDefinition,
            intersection::IntersectionTypeDefinition,
            map::MapTypeDefinition,
            range::RangeTypeDefinition,
            tagged_type::TaggedTypeDefinition,
            union::UnionTypeDefinition,
        },
    },
    values::{
        core_values::{endpoint::Endpoint, integer::Integer},
        value_container::ValueContainer,
    },
};
use test_case::test_case;

#[test_case(LiteralTypeDefinition::Integer(Integer::from(42)))]
#[test_case(LiteralTypeDefinition::TypedInteger(3u8.into()))]
#[test_case(LiteralTypeDefinition::Decimal(1.14.into()))]
#[test_case(LiteralTypeDefinition::TypedDecimal(5.14f64.into()))]
#[test_case(LiteralTypeDefinition::TypedDecimal(6.14f32.into()))]
#[test_case(LiteralTypeDefinition::Text("Hello".to_string().into()))]
#[test_case(LiteralTypeDefinition::Boolean(true.into()))]
#[test_case(LiteralTypeDefinition::Endpoint(Endpoint::new("@+unyt")))]
fn literal(literal: LiteralTypeDefinition) {
    round_trip_type_definition(literal.into());
}

#[test]
fn list() {
    let ty = Type::Definition(
        TypeDefinition::Collection(CollectionTypeDefinition::List(
            ListCollectionTypeDefinition::new(Type::Definition(
                LiteralTypeDefinition::Integer(42.into()).into(),
            )),
        ))
        .into(),
    );
    round_trip_type(ty);
}

#[test]
fn core_type() {
    for id in CoreLibTypeId::iter() {
        round_trip_type_definition(TypeDefinition::CoreType(id));
    }
}

#[test]
fn tagged() {
    let ty = Type::Definition(
        TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "Example".to_string(),
            ty: None,
        })
        .into(),
    );
    round_trip_type(ty);

    let ty = Type::Definition(
        TypeDefinition::TaggedType(TaggedTypeDefinition {
            tag: "Example".to_string(),
            ty: Some(Box::new(Type::Definition(
                LiteralTypeDefinition::Integer(42.into()).into(),
            ))),
        })
        .into(),
    );
    round_trip_type(ty);
}

#[test]
fn union() {
    let ty = Type::Definition(
        TypeDefinition::Union(UnionTypeDefinition::new(vec![
            Type::Definition(LiteralTypeDefinition::Integer(42.into()).into()),
            Type::Definition(
                LiteralTypeDefinition::Text("Hello".to_string().into()).into(),
            ),
        ]))
        .into(),
    );
    round_trip_type(ty);
}

#[test]
fn impl_type() {
    let ty = Type::Definition(
        TypeDefinition::ImplType(ImplTypeDefinition::new(
            Type::Definition(LiteralTypeDefinition::Integer(42.into()).into()),
            vec![PointerAddress::self_owned([1u8, 2u8, 3u8, 4u8, 5u8])],
        ))
        .into(),
    );
    round_trip_type(ty);
}

#[test]
fn callable() {
    let ty = Type::Definition(
        TypeDefinition::Callable(CallableTypeDefinition {
            kind: CallableKind::Function,
            parameters: vec![(
                Some("test".to_owned()),
                Type::Definition(
                    LiteralTypeDefinition::Integer(Integer::from(42)).into(),
                ),
            )],
            return_type: Some(Box::new(Type::Definition(
                LiteralTypeDefinition::Text("Hello".to_string().into()).into(),
            ))),
            requires_async: false,
            rest_parameter: None,
            yeet_type: None,
        })
        .into(),
    );
    round_trip_type(ty);
}

#[test]
fn boxed() {
    let ty = Type::Definition(
        TypeDefinition::Box(Box::new(Type::Definition(
            LiteralTypeDefinition::Integer(42.into()).into(),
        )))
        .into(),
    );
    round_trip_type(ty);
}

#[test]
fn shared_reference() {
    let address_provider = &mut SelfOwnedPointerAddressProvider::default();
    let ty = Type::Definition(
        TypeDefinition::Shared(unsafe {
            SharedContainerContainingType::new_unchecked(
                SharedContainer::Referenced(
                    SharedContainer::new_owned_with_inferred_allowed_type(
                        ValueContainer::Local(
                            Type::Definition(
                                LiteralTypeDefinition::Integer(42.into())
                                    .into(),
                            )
                            .into(),
                        ),
                        SharedContainerMutability::Immutable,
                        address_provider,
                    )
                    .derive_immutable_reference(),
                ),
            )
        })
        .into(),
    );
    round_trip_type(ty);
}

#[test_case(CollectionTypeDefinition::List(ListCollectionTypeDefinition::new(
	Type::Definition(LiteralTypeDefinition::Integer(42.into()).into()),
)))]
#[test_case(CollectionTypeDefinition::Map(MapCollectionTypeDefinition::new(
	Type::Definition(LiteralTypeDefinition::Integer(42.into()).into()),
	Type::Definition(LiteralTypeDefinition::Text("Hello".to_string().into()).into()),
)))]
fn collection(collection: CollectionTypeDefinition) {
    round_trip_type_definition(TypeDefinition::Collection(collection));
}

#[test]
fn intersection() {
    let ty = Type::Definition(
        TypeDefinition::Intersection(IntersectionTypeDefinition::new(vec![
            Type::Definition(LiteralTypeDefinition::Integer(42.into()).into()),
            Type::Definition(
                LiteralTypeDefinition::Text("Hello".to_string().into()).into(),
            ),
        ]))
        .into(),
    );
    round_trip_type(ty);
}

#[test]
fn range() {
    let ty = Type::Definition(
        TypeDefinition::Range(RangeTypeDefinition::new(
            Type::Definition(LiteralTypeDefinition::Integer(42.into()).into()),
            Type::Definition(LiteralTypeDefinition::Integer(100.into()).into()),
        ))
        .into(),
    );
    round_trip_type(ty);
}

#[test]
fn map() {
    let ty = Type::Definition(
        TypeDefinition::Map(MapTypeDefinition::new(vec![
            (
                Type::Definition(
                    LiteralTypeDefinition::Integer(42.into()).into(),
                ),
                Type::Definition(
                    LiteralTypeDefinition::Text("Hello".to_string().into())
                        .into(),
                ),
            ),
            (
                Type::Definition(
                    LiteralTypeDefinition::Boolean(true.into()).into(),
                ),
                Type::Definition(
                    LiteralTypeDefinition::Decimal(1.14.into()).into(),
                ),
            ),
        ]))
        .into(),
    );
    round_trip_type(ty);
}

fn round_trip_type_definition(ty: TypeDefinition) {
    round_trip_type(Type::Definition(ty.into()))
}

fn round_trip_type(ty: Type) {
    let runtime = Runtime::stub();
    let (dxb, _) = compile_template(
        "?",
        vec![Some(ValueContainer::local(ty.clone()))],
        CompileOptions::new(CompilationScope::default(), vec![Endpoint::LOCAL]),
        runtime.clone(),
    )
    .unwrap();
    print_disassembled_with_options(&dxb.dxb, DisassemblerOptions::default());
    let result = execute_dxb_sync(ExecutionInput::new(
        dxb,
        ExecutionCallerMetadata::local_default(),
        ExecutionOptions { verbose: true },
        runtime,
    ))
    .unwrap()
    .unwrap();
    let out: &Type = result.try_as().expect("Result should be a Type");
    assert_eq!(&ty, out);
}
