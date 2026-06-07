//! This module acts as the central type registry, to collect structs and enums annotated with `#[derive(datex)]` to make them available for external projects.
use crate::{
    datex_proxy::DatexProxyTypes, runtime::memory::Memory, types::r#type::Type,
};
/// TODO add metadata, e.g. export opt-in marker, or file export like export_ts="./network/config.d.ts"
/// Add doc comment of the struct/enum as metadata
pub struct DatexRegistration {
    resolve_type: fn(&mut Memory) -> Type,
}

impl DatexRegistration {
    pub const fn new<T: DatexProxyTypes>() -> Self {
        Self {
            resolve_type: <T as DatexProxyTypes>::datex_type,
        }
    }

    pub fn resolve(&self, memory: &mut Memory) -> Type {
        (self.resolve_type)(memory)
    }
}

inventory::collect!(DatexRegistration);

pub fn all_datex_registrations()
-> impl Iterator<Item = &'static DatexRegistration> {
    inventory::iter::<DatexRegistration>.into_iter()
}

pub fn all_datex_types(memory: &mut Memory) -> Vec<Type> {
    all_datex_registrations()
        .map(|registration| registration.resolve(memory))
        .collect()
}
