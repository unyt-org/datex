mod impl_registry;
mod module_registry;
mod type_registry;

pub use impl_registry::*;
pub use module_registry::*;
pub use type_registry::*;


#[cfg(target_arch = "wasm32")]
pub macro registry_collect(
    $ty:ty,
    $registry_ident:ident,
    $function:ident
) {
    use $crate::prelude::*;

    $crate::inventory::collect!($ty);
    
    /// Returns an iterator over all registered items.
    pub fn $function()
    -> impl Iterator<Item = &'static $ty> {
        $crate::inventory::iter::<$ty>.into_iter()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub macro registry_collect(
    $ty:ty,
    $registry_ident:ident,
    $function:ident
) {
    use $crate::prelude::*;

    #[$crate::linkme::distributed_slice]
    pub static $registry_ident: [$ty];

    /// Returns an iterator over all registered items.
    pub fn $function()
    -> impl Iterator<Item = &'static $ty> {
        $registry_ident.into_iter()
    }
}

/// Registers an item in the registry. This macro should be used in the crate that defines the item to be registered.
pub macro registry_register(
    $ty:ty,
    $registry_ident:ident,
    $item:expr
) {
    #[cfg(target_arch = "wasm32")]
    $crate::inventory::submit!($item);
    #[cfg(not(target_arch = "wasm32"))]
    #[$crate::linkme::distributed_slice($registry_ident)]
    static REGISTRATION: $ty = $item;
}