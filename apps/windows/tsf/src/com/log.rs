//! The DLL's own log file: one file per day, `%LOCALAPPDATA%\Bornopata\logs\tsf.<YYYY-MM-DD>.log`, next to the Server's,
//! and only the last [`KEEP_DAYS`] days.
//!
//! It does not use the `tracing` subscriber: the host process may have installed one of its own. Every failure is
//! swallowed, a log must not bring the host down. The file is opened for every line: the DLL is loaded into every
//! application, many of them append to the same file at once, and this is the simplest way to do that safely. The date
//! is read with `GetLocalTime`; this code runs in every process with a text box, so it stays light.

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use borno_platform::paths;
use windows::Win32::Foundation::SYSTEMTIME;
use windows::Win32::System::SystemInformation::GetLocalTime;

/// How many days of logs are kept.
const KEEP_DAYS: u32 = 7;

/// The day of the last line this process wrote (year * 400 + month * 31 + day, only for comparing); old files are
/// removed when it changes.
static LAST_DAY: AtomicU32 = AtomicU32::new(0);

pub(crate) fn log(message: &str) {
    let dir = paths::log_dir();
    let time = unsafe { GetLocalTime() };
    let day = day_key(&time);
    if LAST_DAY.swap(day, Ordering::Relaxed) != day {
        let _ = std::fs::create_dir_all(&dir);
        prune(&dir, day);
    }
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(file_name(&time)))
    else {
        return;
    };
    let _ = writeln!(
        file,
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03} [pid {}] {message}",
        time.wYear,
        time.wMonth,
        time.wDay,
        time.wHour,
        time.wMinute,
        time.wSecond,
        time.wMilliseconds,
        std::process::id()
    );
}

fn file_name(time: &SYSTEMTIME) -> String {
    format!(
        "tsf.{:04}-{:02}-{:02}.log",
        time.wYear, time.wMonth, time.wDay
    )
}

/// A number that grows with the date, for telling old files from new ones by their names.
fn day_key(time: &SYSTEMTIME) -> u32 {
    u32::from(time.wYear) * 400 + u32::from(time.wMonth) * 31 + u32::from(time.wDay)
}

/// Removes the `tsf.*.log` files older than [`KEEP_DAYS`] days. Months count as 31 days here, so a file may live a
/// little longer than a week, never shorter.
fn prune(dir: &PathBuf, today: u32) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(date) = name
            .strip_prefix("tsf.")
            .and_then(|rest| rest.strip_suffix(".log"))
        else {
            continue;
        };
        let mut parts = date.split('-').filter_map(|part| part.parse::<u32>().ok());
        let (Some(year), Some(month), Some(day)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if today.saturating_sub(year * 400 + month * 31 + day) > KEEP_DAYS {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}
