//! This module acts as the central type registry, to collect structs and enums annotated with `#[derive(datex)]` to make them available for external projects.
use crate::{
    datex_proxy::DatexProxyTypes, prelude::*, runtime::cache::shared_references_cache::SharedReferencesCache,
    types::r#type::Type,
};

#[derive(Debug, Clone, Copy)]
pub struct DatexMetadata {
    /// The Datex name.
    ///
    /// Defaults to the Rust struct or enum name. Can be overridden using:
    /// `#[datex(name = "...")]`
    pub name: &'static str,

    /// Doc comments from the struct or enum if available.
    pub docs: Option<&'static str>,

    /// Set to true if this type should be exported to the registry.
    pub export: bool,

    /// The original identifier of the struct or enum used in Rust.
    pub rust_ident: &'static str,

    /// The namespace path. Default is set to the Rust call site, of where the struct or enum is decorated at.
    ///
    /// Set using:
    /// `#[datex(namespace = "./network/config.d.ts")]`
    pub namespace: &'static str,
}

pub struct DatexRegistration {
    pub metadata: DatexMetadata,
    resolve_type: fn(&mut SharedReferencesCache) -> Type,
}

impl DatexRegistration {
    /// Creates a new DatexRegistration for a type T that implements DatexProxyTypes.
    pub const fn new<T: DatexProxyTypes>(metadata: DatexMetadata) -> Self {
        Self {
            metadata,
            resolve_type: <T as DatexProxyTypes>::datex_type,
        }
    }

    /// Resolves the Datex type using the provided memory.
    pub fn resolve(&self, memory: &mut SharedReferencesCache) -> Type {
        (self.resolve_type)(memory)
    }
}

inventory::collect!(DatexRegistration);

/// Returns an iterator over all registered Datex types.
pub fn all_datex_registrations()
-> impl Iterator<Item = &'static DatexRegistration> {
    inventory::iter::<DatexRegistration>.into_iter()
}

/// Returns a vector of all Datex types resolved using the provided memory.
pub fn all_datex_types(memory: &mut SharedReferencesCache) -> Vec<Type> {
    all_datex_registrations()
        .map(|registration| registration.resolve(memory))
        .collect()
}
