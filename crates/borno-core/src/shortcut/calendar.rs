//! The Bengali calendar as Bangladesh keeps it (the 1987 revision): the year starts on 14 April and the first five
//! months have 31 days, the rest 30, except Falgun, which has 31 in the year that holds 29 February.

use super::LocalTime;

/// A date in the Bengali calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BengaliDate {
    pub year: i32,

    /// 0 for Boishakh to 11 for Choitro.
    pub month: usize,

    pub day: u32,
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Days since 1970-01-01 of a Gregorian date (proleptic).
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month = i64::from(month);
    let shifted = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * shifted + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn month_lengths(start_year: i32) -> [i64; 12] {
    let falgun = if is_leap(start_year + 1) { 31 } else { 30 };
    [31, 31, 31, 31, 31, 30, 30, 30, 30, 30, falgun, 30]
}

pub(super) fn bengali_date(time: &LocalTime) -> BengaliDate {
    let before_new_year = (time.month, time.day) < (4, 14);
    let start_year = if before_new_year {
        time.year - 1
    } else {
        time.year
    };
    let mut offset =
        days_from_civil(time.year, time.month, time.day) - days_from_civil(start_year, 4, 14);
    let mut month = 0;
    for length in month_lengths(start_year) {
        if offset < length {
            break;
        }
        offset -= length;
        month += 1;
    }
    BengaliDate {
        year: start_year - 593,
        month: month.min(11),
        day: offset as u32 + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on(year: i32, month: u32, day: u32) -> BengaliDate {
        bengali_date(&LocalTime {
            year,
            month,
            day,
            hour: 0,
            minute: 0,
            weekday: 0,
        })
    }

    fn date(year: i32, month: usize, day: u32) -> BengaliDate {
        BengaliDate { year, month, day }
    }

    #[test]
    fn the_year_turns_on_14_april() {
        assert_eq!(on(2026, 4, 14), date(1433, 0, 1));
        assert_eq!(on(2026, 4, 13), date(1432, 11, 30));
        assert_eq!(on(2024, 4, 14), date(1431, 0, 1));
    }

    #[test]
    fn month_starts_match_the_official_calendar() {
        assert_eq!(on(2026, 10, 20), date(1433, 6, 5));
        assert_eq!(on(2026, 10, 16), date(1433, 6, 1));
        assert_eq!(on(2026, 10, 15), date(1433, 5, 30));
        assert_eq!(on(2026, 12, 15), date(1433, 8, 1));
        assert_eq!(on(2027, 1, 14), date(1433, 9, 1));
        assert_eq!(on(2027, 2, 13), date(1433, 10, 1));
        assert_eq!(on(2027, 3, 15), date(1433, 11, 1));
    }

    #[test]
    fn falgun_has_31_days_when_it_holds_a_leap_day() {
        assert_eq!(on(2024, 2, 29), date(1430, 10, 17));
        assert_eq!(on(2024, 3, 14), date(1430, 10, 31));
        assert_eq!(on(2024, 3, 15), date(1430, 11, 1));
        assert_eq!(on(2025, 3, 14), date(1431, 10, 30));
        assert_eq!(on(2025, 3, 15), date(1431, 11, 1));
    }
}
