//! Shortcut codes that expand to today's date or the time of day.

mod calendar;
mod clock;
mod names;

pub use clock::{Clock, LocalTime};

use calendar::bengali_date;
use names::{BENGALI_MONTHS, GREGORIAN_MONTHS, WEEKDAYS, digits, digits2, part_of_day};

/// What `code` expands to at `time`, best first; empty when `code` is not a shortcut. Codes are Roman spellings of the
/// Bangla words for date, time and weekday.
pub(crate) fn expand(code: &str, time: &LocalTime) -> Vec<String> {
    match code {
        "tarikh" | "tarik" => dates(time),
        "somoy" | "shomoy" => times(time),
        "bar" => weekdays(time),
        _ => Vec::new(),
    }
}

fn dates(time: &LocalTime) -> Vec<String> {
    let day = digits(i64::from(time.day));
    let year = digits(i64::from(time.year));
    let month = GREGORIAN_MONTHS[(time.month as usize).clamp(1, 12) - 1];
    let bengali = bengali_date(time);
    vec![
        format!("{day} {month} {year}"),
        format!("{}/{}/{year}", digits2(time.day), digits2(time.month)),
        format!("{year}-{}-{}", digits2(time.month), digits2(time.day)),
        format!(
            "{} {} {}",
            digits(i64::from(bengali.day)),
            BENGALI_MONTHS[bengali.month],
            digits(i64::from(bengali.year))
        ),
    ]
}

fn times(time: &LocalTime) -> Vec<String> {
    let minute = digits2(time.minute);
    let hour12 = match time.hour % 12 {
        0 => 12,
        hour => hour,
    };
    vec![
        format!("{}:{minute}", digits(i64::from(hour12))),
        format!(
            "{} {}:{minute}",
            part_of_day(time.hour),
            digits(i64::from(hour12))
        ),
        format!("{}:{minute}", digits2(time.hour)),
    ]
}

fn weekdays(time: &LocalTime) -> Vec<String> {
    let weekday = WEEKDAYS[(time.weekday as usize).min(6)];
    let mut found = vec![weekday.to_owned()];
    if let Some(date) = dates(time).into_iter().next() {
        found.push(format!("{weekday}, {date}"));
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tuesday 20 October 2026, 14:05.
    const TUESDAY: LocalTime = LocalTime {
        year: 2026,
        month: 10,
        day: 20,
        hour: 14,
        minute: 5,
        weekday: 2,
    };

    #[test]
    fn a_date_comes_in_four_forms() {
        assert_eq!(
            expand("tarikh", &TUESDAY),
            ["২০ অক্টোবর ২০২৬", "২০/১০/২০২৬", "২০২৬-১০-২০", "৫ কার্তিক ১৪৩৩"]
        );
        assert_eq!(expand("tarik", &TUESDAY), expand("tarikh", &TUESDAY));
    }

    #[test]
    fn the_time_comes_with_and_without_the_part_of_day() {
        assert_eq!(expand("somoy", &TUESDAY), ["২:০৫", "দুপুর ২:০৫", "১৪:০৫"]);
    }

    #[test]
    fn midnight_and_noon_read_as_twelve() {
        let midnight = LocalTime {
            hour: 0,
            minute: 30,
            ..TUESDAY
        };
        assert_eq!(expand("somoy", &midnight)[..2], ["১২:৩০", "রাত ১২:৩০"]);
        let noon = LocalTime {
            hour: 12,
            ..midnight
        };
        assert_eq!(expand("somoy", &noon)[1], "দুপুর ১২:৩০");
    }

    #[test]
    fn the_weekday_alone_and_with_the_date() {
        assert_eq!(
            expand("bar", &TUESDAY),
            ["মঙ্গলবার", "মঙ্গলবার, ২০ অক্টোবর ২০২৬"]
        );
    }

    #[test]
    fn other_codes_expand_to_nothing() {
        assert!(expand("ami", &TUESDAY).is_empty());
        assert!(expand("Tarikh", &TUESDAY).is_empty());
    }
}
