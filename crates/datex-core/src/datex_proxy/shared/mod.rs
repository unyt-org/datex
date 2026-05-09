pub mod datex_proxy;

use crate::datex_proxy::DatexProxy;
use crate::shared_values::SharedContainer;

pub struct Shared<T: DatexProxy> {
    value: T,
    container: SharedContainer,
}

impl<T: DatexProxy> Shared<T> {
}

impl<T: DatexProxy> TryFrom<SharedContainer> for Shared<T> {
    type Error = ();
    fn try_from(container: SharedContainer) -> Result<Self, Self::Error> {
        let value = T::try_from_value_container(container.value_container().clone()).map_err(|_| ())?;
        Ok(Shared { value, container })
    }
}


#[cfg(test)]
mod test {
    use crate::runtime::pointer_address_provider::SelfOwnedPointerAddressProvider;
    use crate::shared_values::SharedContainerMutability;
    use super::*;

    #[test]
    fn string_shared() {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        let shared_container = SharedContainer::new_owned_with_inferred_allowed_type(
            "Hello DATEX",
            SharedContainerMutability::Mutable,
            address_provider
        );

        let shared_string: Shared<String> = Shared::try_from(shared_container).unwrap();
        assert_eq!(shared_string.value, "Hello DATEX");

    }
}