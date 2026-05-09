use crate::{prelude::String, values::core_values::error::TimeError};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(pub i64);

impl Instant {
    /// Return ISO 8601 UTC string with millisecond precision (always ends with 'Z')
    pub fn to_iso_string(&self) -> String {
        let ms = self.0;

        let millis = ms.rem_euclid(1000);
        let total_seconds = ms.div_euclid(1000);

        let second = total_seconds.rem_euclid(60);
        let total_minutes = total_seconds.div_euclid(60);

        let minute = total_minutes.rem_euclid(60);
        let total_hours = total_minutes.div_euclid(60);

        let hour = total_hours.rem_euclid(24);
        let days_since_epoch = total_hours.div_euclid(24);

        // Shift epoch from 1970-01-01 to 0000-03-01, bc of some Howard Hinnant's algorithm
        let z = days_since_epoch + 719468;
        let era = z.div_euclid(146097);
        let doe = z.rem_euclid(146097);

        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let mut year = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);

        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };

        if month <= 2 {
            year += 1;
        }

        format!(
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_epoch() {
        // Exactly 1970-01-01T00:00:00.000Z
        let instant = Instant(0);
        assert_eq!(instant.to_iso_string(), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn test_positive_timestamp() {
        // 2001-09-09T01:46:40.000Z (1 Billion seconds Unix party)
        let instant = Instant(1_000_000_000 * 1000);
        assert_eq!(instant.to_iso_string(), "2001-09-09T01:46:40.000Z");
    }

    #[test]
    fn test_millisecond_precision() {
        // Just 1 millisecond past the epoch
        let instant = Instant(1);
        assert_eq!(instant.to_iso_string(), "1970-01-01T00:00:00.001Z");

        // A random fraction of a second
        let instant = Instant(1_000_000_000 * 1000 + 42);
        assert_eq!(instant.to_iso_string(), "2001-09-09T01:46:40.042Z");
    }

    #[test]
    fn test_negative_timestamp_pre_1970() {
        // 1 millisecond before the Unix epoch
        let instant = Instant(-1);
        assert_eq!(instant.to_iso_string(), "1969-12-31T23:59:59.999Z");

        // 1 second before the Unix epoch
        let instant = Instant(-1000);
        assert_eq!(instant.to_iso_string(), "1969-12-31T23:59:59.000Z");
    }

    #[test]
    fn test_leap_year() {
        // 2024-02-29T12:00:00.000Z (Leap day, Noon UTC)
        // Timestamp: 1709208000 seconds
        let instant = Instant(1709208000 * 1000);
        assert_eq!(instant.to_iso_string(), "2024-02-29T12:00:00.000Z");
    }

    #[test]
    fn test_century_leap_year_rule() {
        // 2000-01-01T00:00:00.000Z (2000 IS a leap year)
        let instant = Instant(946684800 * 1000);
        assert_eq!(instant.to_iso_string(), "2000-01-01T00:00:00.000Z");

        // 1900-01-01T00:00:00.000Z (1900 is NOT a leap year)
        let instant = Instant(-2208988800 * 1000);
        assert_eq!(instant.to_iso_string(), "1900-01-01T00:00:00.000Z");
    }

    #[test]
    fn test_far_future() {
        // 9999-12-31T23:59:59.999Z
        let instant = Instant(253402300799999);
        assert_eq!(instant.to_iso_string(), "9999-12-31T23:59:59.999Z");
    }
}
