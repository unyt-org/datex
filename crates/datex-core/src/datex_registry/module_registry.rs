use crate::{
    prelude::*, runtime::cache::shared_references_cache::SharedReferencesCache,
    values::core_values::map::Map,
};

#[derive(Debug)]
pub struct DatexModuleRegistration {
    pub name: &'static str,
    pub create_module: fn(memory: &mut SharedReferencesCache) -> Map,
}

inventory::collect!(DatexModuleRegistration);

/// Returns an iterator over all registered Datex modules.
pub fn all_datex_module_registrations()
-> impl Iterator<Item = &'static DatexModuleRegistration> {
    inventory::iter::<DatexModuleRegistration>.into_iter()
}

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
