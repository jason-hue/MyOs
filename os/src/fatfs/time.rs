use crate::timer::{get_time_ms, get_time_sec};

const MIN_YEAR: u16 = 1980;
const MAX_YEAR: u16 = 2107;
const MIN_MONTH: u16 = 1;
const MAX_MONTH: u16 = 12;
const MIN_DAY: u16 = 1;
const MAX_DAY: u16 = 31;

/// A DOS compatible date.
///
/// Used by `DirEntry` time-related methods.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub struct Date {
    /// Full year - [1980, 2107]
    pub year: u16,
    /// Month of the year - [1, 12]
    pub month: u16,
    /// Day of the month - [1, 31]
    pub day: u16,
}

impl Date {
    /// Creates a new `Date` instance.
    ///
    /// * `year` - full year number in the range [1980, 2107]
    /// * `month` - month of the year in the range [1, 12]
    /// * `day` - a day of the month in the range [1, 31]
    ///
    /// # Panics
    ///
    /// Panics if one of provided arguments is out of the supported range.
    #[must_use]
    pub fn new(year: u16, month: u16, day: u16) -> Self {
        assert!((MIN_YEAR..=MAX_YEAR).contains(&year), "year out of range");
        assert!(
            (MIN_MONTH..=MAX_MONTH).contains(&month),
            "month out of range"
        );
        assert!((MIN_DAY..=MAX_DAY).contains(&day), "day out of range");
        Self { year, month, day }
    }

    pub(crate) fn decode(dos_date: u16) -> Self {
        let (year, month, day) = (
            (dos_date >> 9) + MIN_YEAR,
            (dos_date >> 5) & 0xF,
            dos_date & 0x1F,
        );
        Self { year, month, day }
    }

    pub(crate) fn encode(self) -> u16 {
        ((self.year - MIN_YEAR) << 9) | (self.month << 5) | self.day
    }
}

/// A DOS compatible time.
///
/// Used by `DirEntry` time-related methods.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub struct Time {
    /// Hours after midnight - [0, 23]
    pub hour: u16,
    /// Minutes after the hour - [0, 59]
    pub min: u16,
    /// Seconds after the minute - [0, 59]
    pub sec: u16,
    /// Milliseconds after the second - [0, 999]
    pub millis: u16,
}

impl Time {
    /// Creates a new `Time` instance.
    ///
    /// * `hour` - number of hours after midnight in the range [0, 23]
    /// * `min` - number of minutes after the hour in the range [0, 59]
    /// * `sec` - number of seconds after the minute in the range [0, 59]
    /// * `millis` - number of milliseconds after the second in the range [0, 999]
    ///
    /// # Panics
    ///
    /// Panics if one of provided arguments is out of the supported range.
    #[must_use]
    pub fn new(hour: u16, min: u16, sec: u16, millis: u16) -> Self {
        assert!(hour <= 23, "hour out of range");
        assert!(min <= 59, "min out of range");
        assert!(sec <= 59, "sec out of range");
        assert!(millis <= 999, "millis out of range");
        Self {
            hour,
            min,
            sec,
            millis,
        }
    }

    pub(crate) fn decode(dos_time: u16, dos_time_hi_res: u8) -> Self {
        let hour = dos_time >> 11;
        let min = (dos_time >> 5) & 0x3F;
        let sec = (dos_time & 0x1F) * 2 + u16::from(dos_time_hi_res / 100);
        let millis = u16::from(dos_time_hi_res % 100) * 10;
        Self {
            hour,
            min,
            sec,
            millis,
        }
    }

    pub(crate) fn encode(self) -> (u16, u8) {
        let dos_time = (self.hour << 11) | (self.min << 5) | (self.sec / 2);
        let dos_time_hi_res = (self.millis / 10) + (self.sec % 2) * 100;
        // safe cast: value in range [0, 199]
        #[allow(clippy::cast_possible_truncation)]
        (dos_time, dos_time_hi_res as u8)
    }

    pub fn sec(self) -> u64 {
        (self.hour * 60 * 60 + self.min * 60 + self.sec) as u64
    }

    pub fn msec(self) -> u64 {
        self.sec() as u64 * 1000 + self.millis as u64
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
}

impl DateTime {
    #[must_use]
    pub fn new(date: Date, time: Time) -> Self {
        Self { date, time }
    }

    pub(crate) fn decode(dos_date: u16, dos_time: u16, dos_time_hi_res: u8) -> Self {
        Self::new(
            Date::decode(dos_date),
            Time::decode(dos_time, dos_time_hi_res),
        )
    }
}
#[inline]
fn get_current_date() -> Date {
    Date::new(1980, 1, 1)
}
#[inline]
fn get_current_time() -> Time {
    let current = get_time_ms();
    let millis = current / 1000;
    let sec = current / (1000 * 60);
    let min = current / (1000 * 60 * 60);
    let hour = current / (1000 * 60 * 60 * 60);

    Time::new(hour as u16, min as u16, sec as u16, millis as u16)
}

pub fn get_current_date_time() -> DateTime {
    DateTime::new(get_current_date(), get_current_time())
}
