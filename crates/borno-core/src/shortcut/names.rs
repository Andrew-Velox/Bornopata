//! Bangla names and digits for dates and times.

pub(super) const GREGORIAN_MONTHS: [&str; 12] = [
    "জানুয়ারি",
    "ফেব্রুয়ারি",
    "মার্চ",
    "এপ্রিল",
    "মে",
    "জুন",
    "জুলাই",
    "আগস্ট",
    "সেপ্টেম্বর",
    "অক্টোবর",
    "নভেম্বর",
    "ডিসেম্বর",
];

pub(super) const BENGALI_MONTHS: [&str; 12] = [
    "বৈশাখ",
    "জ্যৈষ্ঠ",
    "আষাঢ়",
    "শ্রাবণ",
    "ভাদ্র",
    "আশ্বিন",
    "কার্তিক",
    "অগ্রহায়ণ",
    "পৌষ",
    "মাঘ",
    "ফাল্গুন",
    "চৈত্র",
];

/// Sunday first, as `LocalTime::weekday` counts.
pub(super) const WEEKDAYS: [&str; 7] = [
    "রবিবার",
    "সোমবার",
    "মঙ্গলবার",
    "বুধবার",
    "বৃহস্পতিবার",
    "শুক্রবার",
    "শনিবার",
];

/// The part of the day a Bangla speaker names before the time.
pub(super) fn part_of_day(hour: u32) -> &'static str {
    match hour {
        0..=3 => "রাত",
        4..=5 => "ভোর",
        6..=11 => "সকাল",
        12..=14 => "দুপুর",
        15..=16 => "বিকেল",
        17..=18 => "সন্ধ্যা",
        _ => "রাত",
    }
}

/// `value` in Bangla digits.
pub(super) fn digits(value: i64) -> String {
    value
        .to_string()
        .chars()
        .map(|ch| match ch.to_digit(10) {
            Some(digit) => char::from_u32('০' as u32 + digit).unwrap_or(ch),
            None => ch,
        })
        .collect()
}

/// `value` in Bangla digits, at least two of them.
pub(super) fn digits2(value: u32) -> String {
    format!(
        "{}{}",
        if value < 10 { "০" } else { "" },
        digits(i64::from(value))
    )
}
