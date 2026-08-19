pub mod datex_proxy;

use core::ops::Receiver;
use core::ops::Deref;
use std::ops::DerefMut;
use crate::{
    datex_proxy::{
        DatexValueContainerProxySerialize,
    },
    shared_values::{SharedContainer, traits::SharedContainerCommon},
};
use crate::datex_proxy::{DatexValueContainerProxyDeserialize, DatexValueContainerProxyInfallibleSerialize, TryFromDatexValueError, TryToDatexValueError};
use crate::runtime::pointer_address_provider::SelfOwnedPointerAddressProvider;
use crate::shared_values::SharedContainerMutability;
use crate::types::type_definition::TypeDefinition;
use crate::values::core_value::{CoreValue, DatexNative, NativeCoreValue};
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

pub struct Shared<T: DatexNative> {
    container: SharedContainer,
    _phantom_t: core::marker::PhantomData<T>,
}

impl<T: DatexNative> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        todo!()
        // &self.container
    }
}
impl<T: DatexNative> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        todo!()
        // &mut self.container
    }
}

impl<T: DatexNative> Shared<T> {
    pub fn to_container(self) -> SharedContainer {
        self.container
    }
}

impl<T: DatexNative + 'static> Shared<T> {
    pub fn try_new(
        value: T,
        type_definition: TypeDefinition,
        address_provider: &mut SelfOwnedPointerAddressProvider,
    ) -> Result<Self, TryToDatexValueError> {
        let value_container = ValueContainer::from(Value::new(
            CoreValue::Native(NativeCoreValue::new(value)),
            Some(type_definition),
        ));
        Ok(Self {
            container: SharedContainer::new_owned_with_inferred_allowed_type(
                value_container,
                SharedContainerMutability::Mutable,
                address_provider,
            ),
            _phantom_t: core::marker::PhantomData,
        })
    }
}

impl<T: DatexNative + 'static> Shared<T> {
    pub fn new(
        value: T,
        type_definition: TypeDefinition,
        address_provider: &mut SelfOwnedPointerAddressProvider,
    ) -> Self {
        let value_container = ValueContainer::from(Value::new(
            CoreValue::Native(NativeCoreValue::new(value)),
            Some(type_definition),
        ));
        Self {
            container: SharedContainer::new_owned_with_inferred_allowed_type(
                value_container,
                SharedContainerMutability::Mutable,
                address_provider,
            ),
            _phantom_t: core::marker::PhantomData,
        }
    }
}

impl<T: DatexNative + 'static> TryFrom<SharedContainer> for Shared<T>
{
    type Error = TryFromDatexValueError;
    fn try_from(container: SharedContainer) -> Result<Self, Self::Error> {
        // TODO: check if is native
        Ok(Self {
            container,
            _phantom_t: core::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        datex_proxy::shared::Shared,
        runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
        shared_values::{SharedContainer, SharedContainerMutability},
    };

    use crate::prelude::*;
    use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

    // FIME
    // #[test]
    // fn string_shared() {
    //     let address_provider = &mut SelfOwnedPointerAddressProvider::default();
    //
    //     let shared_container =
    //         SharedContainer::new_owned_with_inferred_allowed_type(
    //             "Hello DATEX",
    //             SharedContainerMutability::Mutable,
    //             address_provider,
    //         );
    //
    //     let shared_string: Shared<String, ()> =
    //         Shared::try_from(shared_container).unwrap();
    //     assert_eq!(shared_string.value, "Hello DATEX");
    // }
}
