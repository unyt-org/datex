use alloc::collections::LinkedList;
use crate::time::Instant;
use crate::value_updates::update_data::Update;

#[derive(Debug)]
pub struct UpdateHistoryEntry {
    time: Instant,
    update: Update,
}

#[derive(Default, Debug)]
pub struct UpdateHistory {
    pub updates: LinkedList<UpdateHistoryEntry>,
}


impl UpdateHistory {
    /// Inserts a new update entry into the history
    pub fn insert_entry(&mut self, entry: UpdateHistoryEntry) {
        self.updates.push_back(entry);
    }
    
    /// Gets an iterator over the updates that occurred after the specified [Instant]
    pub fn iter_updates_after_time(&self, time: Instant) -> impl Iterator<Item = &UpdateHistoryEntry> {
        self.updates.iter().filter(move |entry| entry.time > time)
    }
}