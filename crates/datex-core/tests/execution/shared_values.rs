use core::assert_matches;
use datex_core::{
    runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
    shared_values::{SharedContainer, SharedContainerMutability},
    values::{
        core_values::decimal::typed_decimal::TypedDecimal,
        value_container::ValueContainer,
    },
};

use crate::execution::compile_and_execute;

#[test]
fn injected_value_reference() {
    let provider = &mut SelfOwnedPointerAddressProvider::default();
    let input = ValueContainer::Shared(SharedContainer::Referenced(
        SharedContainer::new_owned_with_inferred_allowed_type(
            ValueContainer::from(TypedDecimal::F32(42f32.into())),
            SharedContainerMutability::Immutable,
            provider,
        )
        .derive_immutable_reference(),
    ));
    let referenced = input.clone();
    let result = compile_and_execute(input);
    assert_matches!(
        result,
        ValueContainer::Shared(SharedContainer::Referenced(_))
    );
    assert_eq!(result, referenced);
}

#[test]
fn injected_value_owned() {
    flexi_logger::init();
    let provider = &mut SelfOwnedPointerAddressProvider::default();
    let input = ValueContainer::Shared(
        SharedContainer::new_owned_with_inferred_allowed_type(
            ValueContainer::from(TypedDecimal::F32(42f32.into())),
            SharedContainerMutability::Immutable,
            provider,
        ),
    );
    let referenced = input.clone();
    let result = compile_and_execute(input);
    assert_matches!(result, ValueContainer::Shared(SharedContainer::Owned(_)));
    assert_eq!(result, referenced);
}
