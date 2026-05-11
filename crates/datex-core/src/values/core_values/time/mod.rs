use chrono::*;
use core::{fmt, time::Duration};
use num_integer::Roots;

use crate::{
    prelude::{String, Vec, format, vec},
    // values::core_values::error::TimeError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(pub i128);

impl Instant {
    /// Returns the current UTC time
    pub fn now() -> Self {
        Self(Utc::now().timestamp_millis() as i128) // Current system time
    }

    /// Return ISO 8601 UTC string with millisecond precision (always ends with 'Z')
    pub fn to_iso_string(&self) -> String {
        // Extract seconds and remaining milliseconds from the Instant
        let secs = (self.0.div_euclid(1000)) as i64;
        let millis = (self.0.rem_euclid(1000)) as u32;
        let nanos = millis * 1_000_000;

        let dt = DateTime::from_timestamp(secs, nanos)
            .expect("Timestamp out of range for valid dates");

        format!("{}.{:03}Z", dt.format("%Y-%m-%dT%H:%M:%S"), millis)
    }

    /// converting from iso into Instant(i128)
    pub fn instant_from_iso(s: &str) -> Self {
        let dt = s.parse::<DateTime<Utc>>().expect("Invalid ISO format");

        Self(dt.timestamp_millis() as i128)
    }

    // I always wished for this function in other languages
    /// Return difference between now and other Instant in 'ms'
    /// Return time is always positive
    pub fn difference_between_now(self) -> u64 {
        (self.0 - Instant::now().0).abs() as u64
    }

    /// Add duration (in milliseconds)
    pub fn add_ms(&self, ms: i128) -> Self {
        Instant(self.0 + ms)
    }

    /// Subtract duration (in milliseconds)
    pub fn sub_ms(&self, ms: i128) -> Self {
        Instant(self.0 - ms)
    }

    /// Difference between two Instants (always positive)
    pub fn diff(&self, other: &Instant) -> u64 {
        (self.0 - other.0).abs() as u64
    }

    /// Check if this Instant is after another
    pub fn is_after(&self, other: &Instant) -> bool {
        self.0 > other.0
    }

    /// Check if this Instant is before another
    pub fn is_before(&self, other: &Instant) -> bool {
        self.0 < other.0
    }

    /// Get year, month, day components
    pub fn date_components(&self) -> (i32, u32, u32) {
        let dt: DateTime<Utc> = (*self).into();
        (dt.year(), dt.month(), dt.day())
    }

    /// Get hour, minute, second, millisecond
    pub fn time_components(&self) -> (u32, u32, u32, u32) {
        let dt: DateTime<Utc> = (*self).into();
        (
            dt.hour(),
            dt.minute(),
            dt.second(),
            dt.timestamp_subsec_millis(),
        )
    }

    /// Human readable duration (e.g., "2 days ago", "in 3 hours")
    pub fn human_diff(&self, other: &Instant) -> String {
        let diff_ms = self.diff(other) as i128;
        let diff_secs = diff_ms / 1000;
        let diff_mins = diff_secs / 60;
        let diff_hours = diff_mins / 60;
        let diff_days = diff_hours / 24;

        if diff_days > 0 {
            format!(
                "{} day{} ago",
                diff_days,
                if diff_days > 1 { "s" } else { "" }
            )
        } else if diff_hours > 0 {
            format!(
                "{} hour{} ago",
                diff_hours,
                if diff_hours > 1 { "s" } else { "" }
            )
        } else if diff_mins > 0 {
            format!(
                "{} minute{} ago",
                diff_mins,
                if diff_mins > 1 { "s" } else { "" }
            )
        } else {
            format!(
                "{} second{} ago",
                diff_secs,
                if diff_secs > 1 { "s" } else { "" }
            )
        }
    }

    // /// Sleep until this Instant
    // pub fn sleep_until(&self) {
    //     let now = Instant::now();
    //     if self.0 > now.0 {
    //         let sleep_ms = (self.0 - now.0) as u64;
    //         thread::sleep(std::time::Duration::from_millis(sleep_ms));
    //     }
    // }
}

impl From<&str> for Instant {
    fn from(s: &str) -> Self {
        Instant::instant_from_iso(s)
    }
}

impl From<Instant> for DateTime<Utc> {
    fn from(instant: Instant) -> Self {
        let secs = (instant.0 / 1000) as i64;
        let nanos = ((instant.0 % 1000) * 1_000_000) as u32;
        Utc.timestamp_opt(secs, nanos).unwrap()
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let addr = self.to_iso_string();
        core::write!(f, "{}", addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_epoch() {
        let instant = Instant(0);
        assert_eq!(instant.to_iso_string(), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn test_positive_timestamp() {
        let instant = Instant(1_000_000_000 * 1000);
        assert_eq!(instant.to_iso_string(), "2001-09-09T01:46:40.000Z");
    }

    #[test]
    fn test_millisecond_precision() {
        let instant = Instant(1);
        assert_eq!(instant.to_iso_string(), "1970-01-01T00:00:00.001Z");

        let instant = Instant(1_000_000_000 * 1000 + 42);
        assert_eq!(instant.to_iso_string(), "2001-09-09T01:46:40.042Z");
    }

    #[test]
    fn test_negative_timestamp_pre_1970() {
        let instant = Instant(-1);
        assert_eq!(instant.to_iso_string(), "1969-12-31T23:59:59.999Z");

        let instant = Instant(-1000);
        assert_eq!(instant.to_iso_string(), "1969-12-31T23:59:59.000Z");
    }

    #[test]
    fn test_leap_year() {
        let instant = Instant(1709208000 * 1000);
        assert_eq!(instant.to_iso_string(), "2024-02-29T12:00:00.000Z");
    }

    #[test]
    fn test_century_leap_year_rule() {
        let instant = Instant(946684800 * 1000);
        assert_eq!(instant.to_iso_string(), "2000-01-01T00:00:00.000Z");

        let instant = Instant(-2208988800 * 1000);
        assert_eq!(instant.to_iso_string(), "1900-01-01T00:00:00.000Z");
    }

    #[test]
    fn test_far_future() {
        let instant = Instant(253402300799999);
        assert_eq!(instant.to_iso_string(), "9999-12-31T23:59:59.999Z");
    }

    #[test]
    fn test_instant_from_iso_unix_epoch() {
        let instant = Instant::instant_from_iso("1970-01-01T00:00:00.000Z");
        assert_eq!(instant.0, 0);
    }

    #[test]
    fn test_instant_from_iso_positive_timestamp() {
        let instant = Instant::instant_from_iso("2001-09-09T01:46:40.000Z");
        assert_eq!(instant.0, 1_000_000_000 * 1000);
    }

    #[test]
    fn test_instant_from_iso_with_milliseconds() {
        let instant = Instant::instant_from_iso("2023-12-25T14:30:45.123Z");
        assert_eq!(instant.to_iso_string(), "2023-12-25T14:30:45.123Z");
    }

    #[test]
    fn test_instant_round_trip() {
        let original = Instant(1703518245123);
        let iso_string = original.to_iso_string();
        let parsed = Instant::instant_from_iso(&iso_string);
        assert_eq!(original.0, parsed.0);
        assert_eq!(original.to_iso_string(), parsed.to_iso_string());
    }

    #[test]
    fn test_instant_from_iso_various_timestamps() {
        let test_cases = vec![
            (0, "1970-01-01T00:00:00.000Z"),
            (86400000, "1970-01-02T00:00:00.000Z"),
            (3600000, "1970-01-01T01:00:00.000Z"),
            (61000, "1970-01-01T00:01:01.000Z"),
            (12345, "1970-01-01T00:00:12.345Z"),
            (-3600000, "1969-12-31T23:00:00.000Z"),
        ];

        for (timestamp, iso_string) in test_cases {
            let instant = Instant::instant_from_iso(iso_string);
            assert_eq!(
                instant.0, timestamp,
                "Failed for timestamp: {}",
                iso_string
            );
        }
    }

    // #[test]
    // fn test_now_and_difference() {
    //     let start = Instant::now();
    //     thread::sleep(std::time::Duration::from_millis(15));

    //     let diff = start.difference_between_now();
    //     assert!(
    //         diff >= 10,
    //         "Difference should be at least 10ms, but was {}",
    //         diff
    //     );
    // }

    #[test]
    fn test_add_and_sub_ms() {
        let inst = Instant(1000);
        assert_eq!(inst.add_ms(500).0, 1500);
        assert_eq!(inst.sub_ms(200).0, 800);

        assert_eq!(inst.sub_ms(1500).0, -500);
    }

    #[test]
    fn test_diff_and_comparisons() {
        let inst1 = Instant(1000);
        let inst2 = Instant(2500);

        assert_eq!(inst1.diff(&inst2), 1500);
        assert_eq!(inst2.diff(&inst1), 1500);

        assert!(inst2.is_after(&inst1));
        assert!(!inst1.is_after(&inst2));

        assert!(inst1.is_before(&inst2));
        assert!(!inst2.is_before(&inst1));
    }

    #[test]
    fn test_date_and_time_components() {
        let inst = Instant::instant_from_iso("2023-12-25T14:30:45.123Z");

        assert_eq!(inst.date_components(), (2023, 12, 25));
        assert_eq!(inst.time_components(), (14, 30, 45, 123));
    }

    #[test]
    fn test_human_diff() {
        let base = Instant(10_000_000_000);

        assert_eq!(base.human_diff(&base), "0 second ago");

        assert_eq!(base.human_diff(&base.sub_ms(1000)), "1 second ago");
        assert_eq!(base.human_diff(&base.sub_ms(5000)), "5 seconds ago");

        assert_eq!(base.human_diff(&base.sub_ms(60_000)), "1 minute ago");
        assert_eq!(base.human_diff(&base.sub_ms(180_000)), "3 minutes ago");

        assert_eq!(base.human_diff(&base.sub_ms(3_600_000)), "1 hour ago");
        assert_eq!(base.human_diff(&base.sub_ms(7_200_000)), "2 hours ago");

        assert_eq!(base.human_diff(&base.sub_ms(86_400_000)), "1 day ago");
        assert_eq!(base.human_diff(&base.sub_ms(864_000_000)), "10 days ago");
    }

    // #[test]
    // fn test_sleep_until() {
    //     let start = Instant::now();
    //     let target = start.add_ms(20);

    //     target.sleep_until();

    //     let end = Instant::now();
    //     assert!(
    //         end.0 >= target.0,
    //         "Should have slept until at least the target time"
    //     );
    // }

    #[test]
    fn test_into_datetime() {
        let inst = Instant::instant_from_iso("2024-05-15T08:15:30.500Z");
        let dt: DateTime<Utc> = inst.into();

        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 5);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 8);
        assert_eq!(dt.timestamp_subsec_millis(), 500);
    }
}
