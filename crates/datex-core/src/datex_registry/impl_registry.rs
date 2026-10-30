use core::any::TypeId;
use crate::{
    prelude::*, runtime::cache::shared_references_cache::SharedReferencesCache,
    types::entities::entity_impls::EntityImpl,
};
use crate::datex_registry::registry_collect;

#[derive(Debug)]
pub struct DatexImplRegistration {
    pub name: &'static str,
    pub namespace: &'static str,
    pub create_impl: fn(memory: &mut SharedReferencesCache) -> EntityImpl,
    pub owner_type_id: fn() -> TypeId,
}

registry_collect!(
    DatexImplRegistration,
    DATEX_IMPL_REGISTRATIONS,
    all_datex_impl_registrations
);

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
pub fn get_impls_for<T>(memory: &mut SharedReferencesCache) -> Vec<EntityImpl>
where
    T: 'static,
{
    let owner_type_id = TypeId::of::<T>();
    all_datex_impl_registrations()
        .filter(|registration| (registration.owner_type_id)() == owner_type_id)
        .map(|registration| (registration.create_impl)(memory))
        .collect()
}
