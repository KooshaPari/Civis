//! Crash handling helpers for the Bevy reference client.
use std::backtrace::Backtrace;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;
use std::panic::PanicInfo;
use std::sync::atomic::{AtomicUsize, Ordering};

static CRASH_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Install a process-wide panic hook that writes panic details and backtraces
/// to numbered files under `./crashes`.
pub fn install_crash_handler() {
    std::panic::set_hook(Box::new(|info: &PanicInfo| {
        let counter = CRASH_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
        let path = crash_log_path(counter);
        let output = build_crash_log(info);

        eprintln!("[crash] panic captured at {}", path.display());

        if let Err(err) = create_dir_all("crashes") {
            eprintln!("[crash] failed to create crash directory: {err}");
            eprintln!("{}", output);
            return;
        }

        if let Err(err) = write(&path, output) {
            eprintln!("[crash] failed to write panic log: {err}");
            eprintln!("{}", output);
            return;
        }

        eprintln!("[crash] panic log write complete");
    }));
}

fn build_crash_log(info: &PanicInfo) -> String {
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

    format!(
        "panic.location={}\npanic.payload={payload}\n\nbacktrace:\n{backtrace}\n",
        location
    )
}

fn crash_log_path(counter: usize) -> PathBuf {
    let mut path = PathBuf::from("crashes");
    path.push(format!("{counter}.log"));
    path
}
