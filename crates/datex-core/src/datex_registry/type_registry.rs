//! This module acts as the central type registry, to collect structs and enums annotated with `#[derive(datex)]` to make them available for external projects.
use crate::{
    datex_proxy::DatexProxyTypes, prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::r#type::Type,
};
use core::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub struct DatexTypeMetadata {
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

pub struct DatexTypeRegistration {
    pub metadata: DatexTypeMetadata,
    resolve_type: fn(&mut SharedReferencesCache) -> Type,
}

impl Debug for DatexTypeRegistration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DatexTypeRegistration")
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl DatexTypeRegistration {
    /// Creates a new [DatexTypeRegistration] for a type T that implements [DatexProxyTypes<SharedReferencesCache>].
    pub const fn new_with_cache<T: DatexProxyTypes>(
        metadata: DatexTypeMetadata,
    ) -> Self {
        Self {
            metadata,
            resolve_type: T::datex_type
                as fn(&mut SharedReferencesCache) -> Type,
        }
    }

    /// Creates a new [DatexTypeRegistration] for a type T that implements [DatexProxyTypes].
    pub const fn new_without_cache<T: DatexProxyTypes>(
        metadata: DatexTypeMetadata,
    ) -> Self {
        Self {
            metadata,
            resolve_type: |_| T::datex_type(&mut ()),
        }
    }

    /// Resolves the Datex type using the provided memory.
    pub fn resolve(&self, memory: &mut SharedReferencesCache) -> Type {
        (self.resolve_type)(memory)
    }
}

inventory::collect!(DatexTypeRegistration);

/// Returns an iterator over all registered Datex types.
pub fn all_datex_type_registrations()
-> impl Iterator<Item = &'static DatexTypeRegistration> {
    inventory::iter::<DatexTypeRegistration>.into_iter()
}

/// Returns a vector of all Datex types resolved using the provided memory.
pub fn all_datex_types(memory: &mut SharedReferencesCache) -> Vec<Type> {
    all_datex_type_registrations()
        .map(|registration| registration.resolve(memory))
        .collect()
}
