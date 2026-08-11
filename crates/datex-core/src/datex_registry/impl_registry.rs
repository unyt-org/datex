use crate::types::entities::entity_impls::EntityImpl;
use crate::prelude::*;

#[derive(Debug)]
pub struct DatexImplRegistration {
    pub name: &'static str,
    pub namespace: &'static str,
    pub create_impl: fn () -> EntityImpl
}

inventory::collect!(DatexImplRegistration);


/// Returns an iterator over all registered Datex types.
pub fn all_datex_impl_registrations() -> impl Iterator<Item = &'static DatexImplRegistration> {
    inventory::iter::<DatexImplRegistration>.into_iter()
}

pub fn get_impls(name: &str, namespace: &str) -> Vec<EntityImpl> {
    all_datex_impl_registrations()
        .filter(|registration| registration.namespace == namespace && registration.name == name)
        .map(|registration| (registration.create_impl)())
        .collect()
}