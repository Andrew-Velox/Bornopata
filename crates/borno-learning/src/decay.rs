//! Counts that cool down with time (Newton cooling). A count that only ever grows lets a months-old whim
//! hold a top candidate slot forever; halving it after N unused days keeps the weight following current habits.
//! The granularity is a day (unix day); files carry the last-used day so decay survives a restart.

use std::time::{SystemTime, UNIX_EPOCH};

/// Default half-life in days: a count halves after 90 days unused. 0 turns decay off.
pub const DEFAULT_HALF_LIFE_DAYS: f64 = 90.0;

/// Days of disuse after which an entry counts as really gone and is dropped when saving.
/// Decay only affects ranking weight; entries used once or twice stay in the file as they cool.
pub const COLD_DAYS: u64 = 365;

/// Days since the unix epoch. UTC days differ from local days by half a day, which does not matter for cooling.
pub fn today() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() / 86_400)
}

/// A count that cools: the value and the day it was last used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decay {
    /// Total times recorded; only recording changes it, decay is never written back.
    pub count: u32,

    /// Unix day of the last use.
    pub last_used: u64,
}

impl Decay {
    /// Restores from the saved (count, day).
    pub fn from_parts(count: u32, last_used: u64) -> Self {
        Self { count, last_used }
    }

    /// Records `times` uses and refreshes the last-used day.
    pub fn record(&mut self, times: u32, now: u64) {
        self.count = self.count.saturating_add(times);
        self.last_used = now.max(self.last_used);
    }

    /// Takes back at most `times` uses and returns how many were removed; the caller drops entries that reach zero.
    pub fn unrecord(&mut self, times: u32) -> u32 {
        let removed = times.min(self.count);
        self.count -= removed;
        removed
    }

    /// Effective count: `count × 2^(−days since use / half-life)`, rounded. A non-positive half-life means no decay.
    pub fn effective(&self, now: u64, half_life_days: f64) -> u32 {
        if half_life_days <= 0.0 {
            return self.count;
        }
        let days = now.saturating_sub(self.last_used) as f64;
        (f64::from(self.count) * (-days / half_life_days).exp2()).round() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_decays_by_half_life_and_floor_is_zero() {
        let entry = Decay::from_parts(8, 100);
        assert_eq!(entry.effective(100, 30.0), 8);
        assert_eq!(entry.effective(130, 30.0), 4);
        assert_eq!(entry.effective(160, 30.0), 2);
        assert_eq!(entry.effective(190, 30.0), 1);
        // exactly 0.5 rounds away from zero: still 1 after 120 days, 0 only 10 days later
        assert_eq!(entry.effective(220, 30.0), 1);
        assert_eq!(entry.effective(230, 30.0), 0);
        // a future date (clock set back) does not inflate the count
        assert_eq!(entry.effective(90, 30.0), 8);
    }

    #[test]
    fn zero_half_life_disables_decay() {
        let entry = Decay::from_parts(3, 1);
        assert_eq!(entry.effective(100_000, 0.0), 3);
        assert_eq!(entry.effective(100_000, -1.0), 3);
    }

    #[test]
    fn record_refreshes_last_used_and_unrecord_removes() {
        let mut entry = Decay::from_parts(5, 100);
        entry.record(1, 200);
        assert_eq!(entry.count, 6);
        assert_eq!(entry.last_used, 200);
        // the day never moves backwards
        entry.record(1, 150);
        assert_eq!(entry.last_used, 200);
        assert_eq!(entry.unrecord(2), 2);
        assert_eq!(entry.count, 5);
        assert_eq!(entry.unrecord(9), 5);
        assert_eq!(entry.count, 0);
    }
}
