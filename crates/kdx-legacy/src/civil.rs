//! Civil dates from `Time1900`, for the text files the server writes. The
//! original writes its process-local time; the lab server runs in UTC and
//! so does this, which keeps the files comparable across hosts.

use crate::time::Time1900;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A calendar date and time of day, UTC.
pub struct Civil {
    /// Year.
    pub year: i64,
    /// Month, 1 to 12.
    pub month: u32,
    /// Day of the month, 1 to 31.
    pub day: u32,
    /// Hour, 0 to 23.
    pub hour: u32,
    /// Minute.
    pub minute: u32,
    /// Second.
    pub second: u32,
    /// 0 = Sunday.
    pub weekday: u32,
}

impl Civil {
    /// The civil date of a time, in UTC.
    pub const fn from_time(t: Time1900) -> Civil {
        let unix = t.to_unix_ms().div_euclid(1000);
        let days = unix.div_euclid(86_400);
        let secs = unix.rem_euclid(86_400) as u32;
        let (year, month, day) = civil_from_days(days);
        Civil {
            year,
            month,
            day,
            hour: secs / 3600,
            minute: secs / 60 % 60,
            second: secs % 60,
            weekday: (days + 4).rem_euclid(7) as u32,
        }
    }

    /// `(hour on a 12-hour clock, "AM" | "PM")`.
    pub const fn twelve_hour(&self) -> (u32, &'static str) {
        let h = self.hour % 12;
        (
            if h == 0 { 12 } else { h },
            if self.hour < 12 { "AM" } else { "PM" },
        )
    }

    /// `Sun` to `Sat`.
    pub const fn weekday_name(&self) -> &'static str {
        ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][self.weekday as usize]
    }

    /// `Jan` to `Dec`.
    pub const fn month_name(&self) -> &'static str {
        [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ][(self.month - 1) as usize]
    }
}

/// Days since 1970-01-01 to a proleptic Gregorian date (Howard Hinnant's
/// `civil_from_days`).
const fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}
