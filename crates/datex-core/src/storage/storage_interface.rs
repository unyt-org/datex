use crate::{
    shared_values::pointer_address::PointerAddress,
    values::value_container::ValueContainer,
};

/// Unique identifier for an entry within a storage backend.
///
/// Optionally scoped to a specific collection, enabling partitioned storage.
#[derive(Debug, Clone, Copy)]
pub struct StorageEntryId {
    /// The unique identifier of the entry within its collection or global scope.
    id: u32,
    /// The collection this entry belongs to, or `None` for unscoped entries.
    collection_id: Option<u32>,
}

impl StorageEntryId {
    /// Creates a new `StorageEntryId` with the given entry id and optional collection scope.
    pub fn new(id: u32, collection_id: Option<u32>) -> Self {
        StorageEntryId { id, collection_id }
    }

    /// Returns the unique identifier of this entry.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the collection this entry belongs to, or `None` if unscoped.
    pub fn collection_id(&self) -> Option<u32> {
        self.collection_id
    }
}

/// Defines the contract for a persistent storage backend.
///
/// Implementations provide CRUD operations over [`ValueContainer`] entries
/// identified by [`StorageEntryId`], as well as pointer resolution for
/// indirect addressing.
pub trait StorageInterface {
    /// Persists a new value and returns its assigned [`StorageEntryId`].
    async fn create(&mut self, value: &ValueContainer) -> StorageEntryId;

    /// Removes the entry identified by `id` from storage.
    ///
    /// Returns `true` if the entry existed and was deleted, `false` otherwise.
    async fn delete(&mut self, id: StorageEntryId) -> bool;

    /// Checks whether an entry with the given `id` exists in storage.
    async fn has(&mut self, id: StorageEntryId) -> bool;

    /// Retrieves the value associated with `id`, or `None` if it does not exist.
    async fn get(&mut self, id: StorageEntryId) -> Option<ValueContainer>;

    /// Resolves a [`PointerAddress`] to the concrete [`StorageEntryId`] it references.
    async fn resolve_pointer_address(
        &mut self,
        address: PointerAddress,
    ) -> StorageEntryId;

    /// Overwrites the value of an existing entry identified by `id`.
    async fn update(&mut self, id: StorageEntryId, value: &ValueContainer);
}
