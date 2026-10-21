//! The system clock in the local time zone.

use borno_core::{Clock, LocalTime};

pub struct SystemClock;

#[cfg(unix)]
impl Clock for SystemClock {
    fn now(&self) -> Option<LocalTime> {
        // SAFETY: `time` accepts a null pointer, and `localtime_r` fills the `tm` it is given, which is
        // zero-initialised plain data, without keeping any pointer.
        let moment = unsafe {
            let now = libc::time(std::ptr::null_mut());
            let mut moment: libc::tm = std::mem::zeroed();
            if libc::localtime_r(&now, &mut moment).is_null() {
                return None;
            }
            moment
        };
        Some(LocalTime {
            year: moment.tm_year + 1900,
            month: u32::try_from(moment.tm_mon + 1).ok()?,
            day: u32::try_from(moment.tm_mday).ok()?,
            hour: u32::try_from(moment.tm_hour).ok()?,
            minute: u32::try_from(moment.tm_min).ok()?,
            weekday: u32::try_from(moment.tm_wday).ok()?,
        })
    }
}

#[cfg(windows)]
impl Clock for SystemClock {
    fn now(&self) -> Option<LocalTime> {
        let mut moment =
            std::mem::MaybeUninit::<windows_sys::Win32::Foundation::SYSTEMTIME>::uninit();
        // SAFETY: `GetLocalTime` fills the whole `SYSTEMTIME` it is given and keeps no pointer.
        let moment = unsafe {
            windows_sys::Win32::System::SystemInformation::GetLocalTime(moment.as_mut_ptr());
            moment.assume_init()
        };
        Some(LocalTime {
            year: i32::from(moment.wYear),
            month: u32::from(moment.wMonth),
            day: u32::from(moment.wDay),
            hour: u32::from(moment.wHour),
            minute: u32::from(moment.wMinute),
            weekday: u32::from(moment.wDayOfWeek),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_reads_a_plausible_moment() {
        let now = SystemClock.now().unwrap();
        assert!(now.year >= 2026);
        assert!((1..=12).contains(&now.month));
        assert!((1..=31).contains(&now.day));
        assert!(now.hour < 24 && now.minute < 60 && now.weekday < 7);
    }
}
