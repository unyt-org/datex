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
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::runtime::pointer_address_provider::SelfOwnedPointerAddressProvider;
use crate::shared_values::SharedContainerMutability;

pub struct Shared<T: DatexValueContainerProxySerialize<C>, C = SharedReferencesCache> {
    value: T, // TODO: store actual value inside core value
    container: SharedContainer,
    _phantom: core::marker::PhantomData<C>,
}

impl<T: DatexValueContainerProxySerialize<C>, C> Deref for Shared<T, C> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T: DatexValueContainerProxySerialize<C>, C> DerefMut for Shared<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: DatexValueContainerProxySerialize<C>, C> Shared<T, C> {
    pub fn to_container(self) -> SharedContainer {
        self.container
    }
}

// FIXME: no clone constraint. just move T inside value container
impl<T: DatexValueContainerProxySerialize<C> + Clone, C> Shared<T, C> {
    pub fn try_new(
        value: T,
        address_provider: &mut SelfOwnedPointerAddressProvider,
        context: &mut C,
    ) -> Result<Self, TryToDatexValueError> {
        let value_container = value.clone().try_to_value_container(context)?;
        Ok(Self {
            value,
            container: SharedContainer::new_owned_with_inferred_allowed_type(
                value_container,
                SharedContainerMutability::Mutable,
                address_provider,
            ),
            _phantom: core::marker::PhantomData,
        })
    }
}

// FIXME: no clone constraint. just move T inside value container
impl<T: DatexValueContainerProxyInfallibleSerialize<C> + DatexValueContainerProxySerialize<C> + Clone, C> Shared<T, C> {
    pub fn new(
        value: T,
        address_provider: &mut SelfOwnedPointerAddressProvider,
        context: &mut C,
    ) -> Self {
        let value_container = value.clone().to_value_container(context);
        Self {
            value,
            container: SharedContainer::new_owned_with_inferred_allowed_type(
                value_container,
                SharedContainerMutability::Mutable,
                address_provider,
            ),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<T: DatexValueContainerProxySerialize<C> + DatexValueContainerProxyDeserialize, C> TryFrom<SharedContainer>
for Shared<T, C>
{
    type Error = TryFromDatexValueError;
    fn try_from(container: SharedContainer) -> Result<Self, Self::Error> {
        let value =
            T::try_from_value_container(container.value_container().clone())?;
        Ok(Self {
            value,
            container,
            _phantom: core::marker::PhantomData,
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
