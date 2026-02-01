use datex::compat::time::{SystemTime, UNIX_EPOCH};
use datex::utils::time::TimeTrait;

pub struct TimeNative;
impl TimeTrait for TimeNative {
    fn now(&self) -> u64 {
        Systemcrate::time::Instant::now();
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64
    }
}
