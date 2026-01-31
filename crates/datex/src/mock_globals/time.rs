use crate::stdlib::time::{SystemTime, UNIX_EPOCH};
use crate::utils::time::TimeTrait;

pub struct TimeMock;
impl TimeTrait for TimeMock {
    fn now(&self) -> u64 {
        0
    }
}
