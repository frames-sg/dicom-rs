//! Fully specified calendar values used by the DICOM date, time and range APIs.
//!
//! DICOM additionally permits a leap second. A clock time keeps its fractional
//! second separate from its whole-second index so that leap seconds survive
//! comparisons and conversion back to a DICOM TM or DT.

use std::cmp::Ordering;
use std::convert::TryInto;
use std::fmt;
use std::hash::{Hash, Hasher};

/// A Gregorian date without a time zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NaiveDate(jiff::civil::Date);

impl NaiveDate {
    /// Constructs a date, rejecting invalid calendar days.
    pub fn from_ymd_opt(year: i32, month: u32, day: u32) -> Option<Self> {
        jiff::civil::Date::new(
            year.try_into().ok()?,
            month.try_into().ok()?,
            day.try_into().ok()?,
        )
        .ok()
        .map(Self)
    }

    pub fn year(self) -> i32 {
        self.0.year().into()
    }
    pub fn month(self) -> u32 {
        self.0.month() as u32
    }
    pub fn day(self) -> u32 {
        self.0.day() as u32
    }
    pub(crate) fn days_in_month(self) -> u32 {
        self.0.days_in_month() as u32
    }
}

impl fmt::Display for NaiveDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A clock time without a time zone. A fraction at or above one second
/// on second 59 represents second 60 of the same minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NaiveTime {
    seconds: u32,
    nanos: u32,
}

impl NaiveTime {
    pub fn from_hms_opt(hour: u32, minute: u32, second: u32) -> Option<Self> {
        Self::from_hms_micro_opt(hour, minute, second, 0)
    }

    pub fn from_hms_milli_opt(hour: u32, minute: u32, second: u32, milli: u32) -> Option<Self> {
        Self::from_hms_micro_opt(hour, minute, second, milli.checked_mul(1000)?)
    }

    /// Accepts DICOM second 60 or its representation as an extended fraction on second 59.
    pub fn from_hms_micro_opt(hour: u32, minute: u32, second: u32, micro: u32) -> Option<Self> {
        let (second, micro) = if second == 60 && micro < 1_000_000 {
            (59, micro + 1_000_000)
        } else {
            (second, micro)
        };
        if hour >= 24
            || minute >= 60
            || second >= 60
            || micro >= 2_000_000
            || (micro >= 1_000_000 && second != 59)
        {
            return None;
        }
        Some(Self {
            seconds: hour * 3600 + minute * 60 + second,
            nanos: micro * 1000,
        })
    }

    pub fn hour(self) -> u32 {
        self.seconds / 3600
    }
    pub fn minute(self) -> u32 {
        self.seconds / 60 % 60
    }
    pub fn second(self) -> u32 {
        self.seconds % 60
    }
    pub fn nanosecond(self) -> u32 {
        self.nanos
    }
}

impl fmt::Display for NaiveTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let second = self.second() + self.nanos / 1_000_000_000;
        write!(f, "{:02}:{:02}:{:02}", self.hour(), self.minute(), second)?;
        let fraction = self.nanos % 1_000_000_000;
        if fraction == 0 {
            return Ok(());
        }
        if fraction % 1_000_000 == 0 {
            write!(f, ".{:03}", fraction / 1_000_000)
        } else if fraction % 1000 == 0 {
            write!(f, ".{:06}", fraction / 1000)
        } else {
            write!(f, ".{fraction:09}")
        }
    }
}

/// A date and clock time without a time zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NaiveDateTime {
    date: NaiveDate,
    time: NaiveTime,
}

impl NaiveDateTime {
    pub fn new(date: NaiveDate, time: NaiveTime) -> Self {
        Self { date, time }
    }
    pub fn date(self) -> NaiveDate {
        self.date
    }
    pub fn time(self) -> NaiveTime {
        self.time
    }
}

impl From<jiff::civil::DateTime> for NaiveDateTime {
    fn from(dt: jiff::civil::DateTime) -> Self {
        let seconds = dt.hour() as u32 * 3600 + dt.minute() as u32 * 60 + dt.second() as u32;
        Self::new(
            NaiveDate(dt.date()),
            NaiveTime {
                seconds,
                nanos: dt.subsec_nanosecond() as u32,
            },
        )
    }
}

impl fmt::Display for NaiveDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.date, self.time)
    }
}

/// A fixed difference from UTC, in seconds east of Greenwich.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixedOffset(i32);

impl FixedOffset {
    pub fn east_opt(seconds: i32) -> Option<Self> {
        (-86_399..=86_399)
            .contains(&seconds)
            .then_some(Self(seconds))
    }

    pub fn west_opt(seconds: i32) -> Option<Self> {
        Self::east_opt(seconds.checked_neg()?)
    }
    pub fn local_minus_utc(self) -> i32 {
        self.0
    }
    pub fn at(self, local: NaiveDateTime) -> OffsetDateTime {
        OffsetDateTime {
            local,
            offset: self,
        }
    }
}

impl fmt::Display for FixedOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.0.unsigned_abs();
        let sign = if self.0 < 0 { '-' } else { '+' };
        write!(f, "{sign}{:02}:{:02}", seconds / 3600, seconds / 60 % 60)?;
        if seconds % 60 != 0 {
            write!(f, ":{:02}", seconds % 60)?;
        }
        Ok(())
    }
}

/// A local calendar value with a fixed UTC offset. Equality and ordering
/// compare instants, including the fractional part of a leap second.
#[derive(Debug, Clone, Copy)]
pub struct OffsetDateTime {
    local: NaiveDateTime,
    offset: FixedOffset,
}

impl OffsetDateTime {
    pub fn naive_local(self) -> NaiveDateTime {
        self.local
    }
    pub fn offset(&self) -> &FixedOffset {
        &self.offset
    }

    fn instant(self) -> (i64, u32) {
        let epoch = jiff::civil::date(1970, 1, 1);
        let midnight = self.local.date.0.duration_since(epoch).as_secs();
        (
            midnight + i64::from(self.local.time.seconds) - i64::from(self.offset.0),
            self.local.time.nanos,
        )
    }
}

impl PartialEq for OffsetDateTime {
    fn eq(&self, other: &Self) -> bool {
        self.instant() == other.instant()
    }
}
impl Eq for OffsetDateTime {}
impl PartialOrd for OffsetDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for OffsetDateTime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.instant().cmp(&other.instant())
    }
}
impl Hash for OffsetDateTime {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.instant().hash(state)
    }
}
impl fmt::Display for OffsetDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.local, self.offset)
    }
}

/// The local wall clock and its current UTC offset from the same instant.
pub fn local_now() -> OffsetDateTime {
    let now = jiff::Zoned::now();
    FixedOffset(now.offset().seconds()).at(now.datetime().into())
}

pub(crate) fn utc_now() -> NaiveDateTime {
    jiff::tz::Offset::UTC
        .to_datetime(jiff::Timestamp::now())
        .into()
}

#[cfg(test)]
mod tests {
    use crate::value::{AsRange, DicomDateTime};
    use std::convert::TryFrom;

    #[test]
    fn leap_second_range_survives_offsets_at_midnight() {
        let local: DicomDateTime = "20170101005960.5+0100".parse().unwrap();
        let utc: DicomDateTime = "20161231235960.5+0000".parse().unwrap();
        let next: DicomDateTime = "20170101000000.000000+0000".parse().unwrap();
        assert_eq!(local.earliest().unwrap(), utc.earliest().unwrap());
        let last = local.latest().unwrap();
        assert!(last < next.earliest().unwrap());
        let encoded = DicomDateTime::try_from(last.as_datetime().unwrap()).unwrap();
        assert_eq!(encoded.to_encoded(), "20170101005960.599999+0100");
    }
}
