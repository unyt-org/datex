use crate::value_updates::{
    update_data::Update, update_strategies::time_based::UpdateId,
};

#[derive(Clone, Debug)]
pub struct UpdateHistoryEntry {
    pub id: UpdateId,
    pub update: Update,
    pub history_hash: u64,
}
