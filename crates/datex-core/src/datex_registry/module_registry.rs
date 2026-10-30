use crate::{
    prelude::*, runtime::cache::shared_references_cache::SharedReferencesCache,
    values::core_values::map::Map,
};
use crate::datex_registry::registry_collect;

#[derive(Debug)]
pub struct DatexModuleRegistration {
    pub name: &'static str,
    pub create_module: fn(memory: &mut SharedReferencesCache) -> Map,
}


registry_collect!(
    DatexModuleRegistration,
    DATEX_MODULE_REGISTRATIONS,
    all_datex_module_registrations
);


pub fn get_all_modules(
    memory: &mut SharedReferencesCache,
) -> Vec<(String, Map)> {
    all_datex_module_registrations()
        .map(|registration| {
            (
                registration.name.to_string(),
                (registration.create_module)(memory),
            )
        })
        .collect()
}
