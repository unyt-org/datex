use crate::{
    shared_values::SharedContainer,
    value_updates::update_strategies::time_based::UpdateId,
};

#[derive(Clone)]
pub struct Snapshot {
    /// Value after every update up to last_update
    pub value: SharedContainer,
    /// Hash of complete history until snapshot
    pub history_hash: u64,
    /// Last applied update
    pub last_update: Option<UpdateId>,
}

impl Snapshot {
    pub fn new(value: SharedContainer) -> Self {
        Self {
            value,
            history_hash: 0,
            last_update: None,
        }
    }
}
