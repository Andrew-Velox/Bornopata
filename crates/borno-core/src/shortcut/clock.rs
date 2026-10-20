//! The wall clock as the engine sees it.

/// A moment in local time, in the Gregorian calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    pub year: i32,

    /// 1 for January to 12 for December.
    pub month: u32,

    /// 1-based.
    pub day: u32,

    /// 0 to 23.
    pub hour: u32,

    pub minute: u32,

    /// 0 for Sunday to 6 for Saturday.
    pub weekday: u32,
}

/// Implemented by the shell, which knows the time zone. Core never reads the clock itself, so tests can fix it.
pub trait Clock: Send {
    /// The current local time; `None` when it cannot be read.
    fn now(&self) -> Option<LocalTime>;
}
