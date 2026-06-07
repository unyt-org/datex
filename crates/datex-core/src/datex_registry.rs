//! This module acts as the central type registry, to collect structs and enums annotated with `#[derive(datex)]` to make them available for external projects.
use crate::{
    datex_proxy::DatexProxyTypes, runtime::memory::Memory, types::r#type::Type,
};

#[derive(Debug, Clone, Copy)]
pub struct DatexMetadata {
    /// The Datex name.
    ///
    /// Defaults to the Rust struct or enum name. Can be overridden using:
    /// `#[datex(name = "...")]`
    pub name: &'static str,
    pub rust_type_name: &'static str,
    pub rust_crate_name: &'static str,
    pub rust_package_name: &'static str,
    pub rust_module_path: &'static str,
    pub rust_path: &'static str,

    /// Doc comments from the struct or enum if available.
    pub docs: Option<&'static str>,

    /// Set to true if this type should be exported to the registry.
    pub export: bool,

    /// Optional TS export path.
    ///
    /// Set using:
    /// `#[datex(export_ts = "./network/config.d.ts")]`
    /// FIXME: Shall we rename to namespace and use something like network::iterface::config and let the export logic decide how to map
    /// the namespace to a file path based on the language / project structure?
    pub export_ts: Option<&'static str>,
}

pub struct DatexRegistration {
    pub metadata: DatexMetadata,
    resolve_type: fn(&mut Memory) -> Type,
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
    pub fn resolve(&self, memory: &mut Memory) -> Type {
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
pub fn all_datex_types(memory: &mut Memory) -> Vec<Type> {
    all_datex_registrations()
        .map(|registration| registration.resolve(memory))
        .collect()
}
