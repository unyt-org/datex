pub mod datex_proxy;

use crate::{
    datex_proxy::{
        DatexValueContainerProxy, DatexValueContainerProxyDeserialize,
        DatexValueContainerProxySerialize,
        TryFromDatexValueError,
    },
    shared_values::{SharedContainer, traits::SharedContainerCommon},
};

pub struct Shared<T: DatexValueContainerProxySerialize<C> + DatexValueContainerProxyDeserialize, C> {
    value: T,
    container: SharedContainer,
    _phantom: core::marker::PhantomData<C>,
}

impl<T: DatexValueContainerProxySerialize<C> + DatexValueContainerProxyDeserialize, C> Shared<T, C> {
    pub fn new(value: T, container: SharedContainer) -> Self {
        Self {
            value,
            container,
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
        Ok(Shared::new(value, container))
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

    #[test]
    fn string_shared() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        let shared_container =
            SharedContainer::new_owned_with_inferred_allowed_type(
                "Hello DATEX",
                SharedContainerMutability::Mutable,
                address_provider,
            );

        let shared_string: Shared<String, ()> =
            Shared::try_from(shared_container).unwrap();
        assert_eq!(shared_string.value, "Hello DATEX");
    }
}
