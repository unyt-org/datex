use core::{
    cell::Ref,
    hash::{Hash, Hasher},
};

use crate::{
    datex_proxy::DatexValueContainerProxyInfallibleSerialize,
    shared_values::{SharedContainer, traits::SharedContainerCommon},
    value_updates::{
        errors::UpdateError,
        update_handler::{UpdateHandler, UpdateResult},
    },
    values::value_container::ValueContainer,
};
use fnv64_rs::Fnv1aHasher;

use crate::prelude::*;
use alloc::collections::BTreeSet;
mod update_id;
pub use update_id::*;

mod history_entry;
pub use history_entry::*;

mod snapshot;
pub use snapshot::*;

mod sync;
pub use sync::*;

pub struct UpdateHistory {
    snapshot: Snapshot,
    current: SharedContainer,
    entries: Vec<UpdateHistoryEntry>,
    known_updates: BTreeSet<UpdateId>,
}

impl UpdateHistory {
    pub fn new(initial: SharedContainer) -> Self {
        Self {
            snapshot: Snapshot::new(initial.clone()),
            current: initial,
            entries: Vec::new(),
            known_updates: BTreeSet::new(),
        }
    }

    /// Returns the index at which an update with the given id should be inserted
    /// If an update with the same id already exists, the index of that update is returned
    fn insertion_index(&self, id: &UpdateId) -> usize {
        match self.entries.binary_search_by(|e| e.id.cmp(id)) {
            Ok(idx) => idx,
            Err(idx) => idx,
        }
    }

    /// Checks if the update history contains an update with the given id
    pub fn contains(&self, id: UpdateId) -> bool {
        self.known_updates.contains(&id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn history_hash(&self) -> u64 {
        self.entries
            .last()
            .map(|e| e.history_hash)
            .unwrap_or(self.snapshot.history_hash)
    }

    pub fn last_update(&self) -> Option<UpdateId> {
        self.entries.last().map(|e| e.id.clone())
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    fn rebuild_hashes(&mut self) {
        let mut hasher = Fnv1aHasher::default();
        hasher.write_u64(self.snapshot.history_hash);

        for entry in &mut self.entries {
            hasher.write_u64(entry.id.timestamp);
            hasher.write(&entry.id.endpoint.to_slice());
            entry.update.hash(&mut hasher);
            entry.history_hash = hasher.clone().finish();
        }
    }
    pub fn value(&self) -> Ref<'_, ValueContainer> {
        Ref::map(self.current.base_shared_container(), |base| {
            base.value_container()
        })
    }

    pub fn insert(
        &mut self,
        entry: UpdateHistoryEntry,
    ) -> Result<bool, UpdateError> {
        self.insert_multiple(vec![entry])
    }
    fn insert_multiple(
        &mut self,
        entries: Vec<UpdateHistoryEntry>,
    ) -> Result<bool, UpdateError> {
        for mut entry in entries {
            if self.known_updates.contains(&entry.id) {
                return Ok(false);
            }
            let idx = self.insertion_index(&entry.id);
            entry.history_hash = 0;
            let id = entry.id.clone();
            self.entries.insert(idx, entry);
            self.known_updates.insert(id);
        }

        self.rebuild_hashes();
        self.replay_all()?;
        Ok(true)
    }

    fn replay_all(&mut self) -> Result<(), UpdateError> {
        self.current = self.snapshot.value.clone();
        for entry in &self.entries {
            self.current.update(entry.update.clone())?;
        }
        Ok(())
    }

    pub fn sync_state(&self) -> SyncState {
        SyncState {
            last_update: self.last_update(),
            history_hash: self.history_hash(),
        }
    }

    pub fn compact(&mut self) {
        self.snapshot = Snapshot {
            value: self.current.clone(),
            history_hash: self.history_hash(),
            last_update: self.entries.last().map(|e| e.id.clone()),
        };
        self.entries.clear();
        self.known_updates.clear();
    }

    pub fn updates_after(&self, update: Option<UpdateId>) -> UpdateBatch {
        let start = match &update {
            None => 0,
            Some(id) => match self.entries.binary_search_by(|e| e.id.cmp(id)) {
                Ok(index) => index + 1,
                Err(index) => index,
            },
        };

        let updates = self.entries[start..].to_vec();
        UpdateBatch {
            from: update,
            updates,
            final_hash: self.history_hash(),
        }
    }
    pub fn verify_hash(&self, expected: u64) -> bool {
        self.history_hash() == expected
    }

    pub fn apply_batch(
        &mut self,
        batch: UpdateBatch,
    ) -> Result<(), UpdateError> {
        self.insert_multiple(batch.updates)?;
        if self.history_hash() != batch.final_hash {
            return Err(UpdateError::InvalidUpdate);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        prelude::*,
        runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
        shared_values::{
            SharedContainerMutability,
            base_shared_value_container::observers::TransceiverId,
            traits::SharedContainerCommon,
        },
        value_updates::update_data::{ReplaceUpdateData, Update, UpdateData},
        values::{core_values::endpoint::Endpoint, value::Value},
    };
    fn shared<T>(
        value: T,
        provider: &mut SelfOwnedPointerAddressProvider,
    ) -> SharedContainer
    where
        T: Into<ValueContainer>,
    {
        SharedContainer::new_owned_with_inferred_allowed_type(
            value.into(),
            SharedContainerMutability::Mutable,
            provider,
        )
    }

    fn empty_value(
        provider: &mut SelfOwnedPointerAddressProvider,
    ) -> SharedContainer {
        shared(Value::null(), provider)
    }

    fn entry(timestamp: u64, value: ValueContainer) -> UpdateHistoryEntry {
        UpdateHistoryEntry {
            id: UpdateId {
                timestamp,
                endpoint: Endpoint::new("@jonas"),
            },
            update: Update::new(
                TransceiverId::Local,
                UpdateData::Replace(ReplaceUpdateData { value }),
            ),
            history_hash: 0,
        }
    }

    #[test]
    fn empty_history() {
        let provider = &mut SelfOwnedPointerAddressProvider::default();
        let history = UpdateHistory::new(empty_value(provider));
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
        assert!(history.last_update().is_none());
    }

    #[test]
    fn insert_order() {
        let provider = &mut SelfOwnedPointerAddressProvider::default();
        let mut history = UpdateHistory::new(empty_value(provider));

        history.insert(entry(3, 30.into())).unwrap();
        history.insert(entry(1, 10.into())).unwrap();
        history.insert(entry(2, 20.into())).unwrap();

        assert_eq!(
            history
                .entries
                .iter()
                .map(|e| e.id.timestamp)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn ignore_duplicate_updates() {
        let provider = &mut SelfOwnedPointerAddressProvider::default();
        let mut history = UpdateHistory::new(empty_value(provider));
        let update = entry(1, 100.into());
        assert!(history.insert(update.clone()).unwrap());

        // Inserting the same update again should return false and not change the length of the history
        assert!(!history.insert(update).unwrap());
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn random_order() {
        let provider = &mut SelfOwnedPointerAddressProvider::default();
        let updates = vec![
            entry(1, 10.into()),
            entry(2, 20.into()),
            entry(3, 30.into()),
            entry(4, 40.into()),
            entry(5, 50.into()),
        ];

        // Insert in order
        let mut ordered = UpdateHistory::new(empty_value(provider));
        for u in updates.clone() {
            ordered.insert(u).unwrap();
        }

        // Insert in random order
        let mut shuffled = UpdateHistory::new(empty_value(provider));
        for u in [3, 0, 4, 1, 2] {
            shuffled.insert(updates[u].clone()).unwrap();
        }

        // Check that the final value and history hash are the same
        assert_eq!(ordered.value().get_cloned(), shuffled.value().get_cloned());
        assert_eq!(ordered.history_hash(), shuffled.history_hash());
    }

    #[test]
    fn hashes_deterministic() {
        let provider = &mut SelfOwnedPointerAddressProvider::default();
        let updates = vec![
            entry(1, 10.into()),
            entry(2, 20.into()),
            entry(3, 30.into()),
        ];

        let mut a = UpdateHistory::new(empty_value(provider));
        let mut b = UpdateHistory::new(empty_value(provider));

        // Insert in order into a
        for u in &updates {
            a.insert(u.clone()).unwrap();
        }

        // Insert in reverse order into b
        for u in updates.iter().rev() {
            b.insert(u.clone()).unwrap();
        }

        assert_eq!(a.history_hash(), b.history_hash());
    }

    #[test]
    fn compact_preserves_value() {
        let provider = &mut SelfOwnedPointerAddressProvider::default();
        let mut history = UpdateHistory::new(empty_value(provider));

        history.insert(entry(1, 10.into())).unwrap();
        history.insert(entry(2, 20.into())).unwrap();

        let before = history.value().clone();

        history.compact();

        assert_eq!(history.value().clone(), before);
        assert_eq!(history.len(), 0);
        assert!(history.snapshot().last_update.is_some());
    }

    #[test]
    fn updates_after() {
        let provider = &mut SelfOwnedPointerAddressProvider::default();
        let mut history = UpdateHistory::new(empty_value(provider));

        for i in 1..=5 {
            history.insert(entry(i, i.into())).unwrap();
        }
        let batch = history.updates_after(Some(UpdateId {
            timestamp: 3,
            endpoint: Endpoint::new("@jonas"),
        }));
        assert_eq!(batch.updates.len(), 2);
        assert_eq!(batch.updates[0].id.timestamp, 4);
        assert_eq!(batch.updates[1].id.timestamp, 5);
    }

    #[test]
    fn two_nodes_sync() {
        let provider = &mut SelfOwnedPointerAddressProvider::default();
        let mut endpoint_owner = UpdateHistory::new(empty_value(provider));
        let mut endpoint_receiver = UpdateHistory::new(empty_value(provider));

        for i in 1..=10 {
            endpoint_owner.insert(entry(i, i.into())).unwrap();
        }

        let batch = endpoint_owner.updates_after(None);
        endpoint_receiver.apply_batch(batch).unwrap();
        assert_eq!(
            endpoint_owner.value().clone(),
            endpoint_receiver.value().clone()
        );
        assert_eq!(
            endpoint_owner.history_hash(),
            endpoint_receiver.history_hash()
        );
    }
}
