//! Crash handling helpers for the Bevy reference client.
//!
//! Panic logs land under the per-user data directory when available
//! (`%LOCALAPPDATA%/Civis/crashes` on Windows), falling back to `./crashes`.

use std::backtrace::Backtrace;
use std::fs::{create_dir_all, write};
#[allow(deprecated)]
use std::panic::PanicInfo;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static CRASH_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Install a process-wide panic hook that writes panic details and backtraces
/// to timestamped files under the Civis crash directory.
#[allow(deprecated)]
pub fn install_crash_handler() {
    std::panic::set_hook(Box::new(|info: &PanicInfo<'_>| {
        let counter = CRASH_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
        let dir = crash_dir();
        let path = crash_log_path(&dir, counter);
        let output = build_crash_log(info);

        eprintln!("[crash] panic captured at {}", path.display());

        if let Err(err) = create_dir_all(&dir) {
            eprintln!("[crash] failed to create crash directory: {err}");
            eprintln!("{output}");
            return;
        }

        if let Err(err) = write(&path, &output) {
            eprintln!("[crash] failed to write panic log: {err}");
            eprintln!("{output}");
            return;
        }

        eprintln!("[crash] panic log write complete");
    }));
}

/// Directory used for crash logs (absolute when local-data is available).
#[must_use]
pub fn crash_dir() -> PathBuf {
    if let Some(base) = std::env::var("LOCALAPPDATA").ok().filter(|s| !s.is_empty()) {
        return PathBuf::from(base).join("Civis").join("crashes");
    }
    if let Some(base) = std::env::var("XDG_DATA_HOME").ok().filter(|s| !s.is_empty()) {
        return PathBuf::from(base).join("civis").join("crashes");
    }
    if let Some(home) = std::env::var("HOME").ok().filter(|s| !s.is_empty()) {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("civis")
            .join("crashes");
    }
    PathBuf::from("crashes")
}

/// Format a crash log body from panic hook info (exported for tests).
#[must_use]
#[allow(deprecated)]
pub fn build_crash_log(info: &PanicInfo<'_>) -> String {
    let location = match info.location() {
        Some(location) => format!("{}:{}", location.file(), location.line()),
        None => "<unknown location>".to_string(),
    };
    let payload = if let Some(message) = info.payload().downcast_ref::<&str>() {
        *message
    } else if let Some(message) = info.payload().downcast_ref::<String>() {
        message.as_str()
    } else {
        "<panic payload non-string>"
    };
    let backtrace = Backtrace::capture();

    format!("panic.location={location}\npanic.payload={payload}\n\nbacktrace:\n{backtrace}\n")
}

fn crash_log_path(dir: &std::path::Path, counter: usize) -> PathBuf {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let pid = std::process::id();
    dir.join(format!("crash-{secs}-{pid}-{counter}.log"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crash_dir_is_non_empty() {
        let dir = crash_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn crash_log_path_includes_pid_and_counter() {
        let path = crash_log_path(std::path::Path::new("/tmp/crashes"), 7);
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(name.contains(&format!("-{}-", std::process::id())));
        assert!(name.ends_with("-7.log"));
        assert!(name.starts_with("crash-"));
    }
}
