use crate::value_updates::update_strategies::time_based::{
    UpdateHistoryEntry, UpdateId,
};

use crate::prelude::*;
#[derive(Clone, Debug)]
pub struct SyncState {
    /// Last operation known by this endpoint
    pub last_update: Option<UpdateId>,

    /// Hash of the history at this point
    pub history_hash: u64,
}

#[derive(Clone, Debug)]
pub struct UpdateBatch {
    pub from: Option<UpdateId>,
    pub updates: Vec<UpdateHistoryEntry>,
    pub final_hash: u64,
}
