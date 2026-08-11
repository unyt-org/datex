use crate::{
    prelude::*, runtime::cache::shared_references_cache::SharedReferencesCache,
    types::entities::entity_impls::EntityImpl,
};

#[derive(Debug)]
pub struct DatexImplRegistration {
    pub name: &'static str,
    pub namespace: &'static str,
    pub create_impl: fn(memory: &mut SharedReferencesCache) -> EntityImpl,
}

inventory::collect!(DatexImplRegistration);

/// Returns an iterator over all registered Datex types.
pub fn all_datex_impl_registrations()
-> impl Iterator<Item = &'static DatexImplRegistration> {
    inventory::iter::<DatexImplRegistration>.into_iter()
}

pub fn get_impls(
    name: &str,
    namespace: &str,
    memory: &mut SharedReferencesCache,
) -> Vec<EntityImpl> {
    all_datex_impl_registrations()
        .filter(|registration| {
            registration.namespace == namespace && registration.name == name
        })
        .map(|registration| (registration.create_impl)(memory))
        .collect()
}
