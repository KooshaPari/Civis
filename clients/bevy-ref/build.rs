//! Build script — embeds the Civis `.exe` icon on Windows release builds.
//!
//! The runtime *window* icon is handled separately at startup
//! (`src/window_icon.rs`); this script handles the **executable file** icon that
//! Explorer, the taskbar, and shortcuts display. It stamps the multi-size
//! `assets/icon/civis.ico` (16/24/32/48/64/128/256) onto the produced `.exe`
//! via the Windows resource compiler.
//!
//! It is gated behind the `exe-icon` cargo feature so the default build (and CI
//! on non-Windows) needs no extra build-dependency or `rc.exe`. Enable with:
//!
//! ```text
//! cargo build -p civ-bevy-ref --features "bevy,egui,exe-icon" --release
//! ```
//!
//! Requires the `winresource` build-dependency, which is itself feature-gated
//! (see Cargo.toml `[build-dependencies]`) so it is only pulled when `exe-icon`
//! is on.

fn main() {
    // Re-run if the icon changes.
    println!("cargo:rerun-if-changed=assets/icon/civis.ico");

    #[cfg(all(windows, feature = "exe-icon"))]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon/civis.ico");
        res.set("ProductName", "Civis");
        res.set("FileDescription", "Civis — civilisation sandbox");
        if let Err(e) = res.compile() {
            // Loud, non-fatal: report but do not abort the whole build if the
            // platform resource compiler is unavailable.
            println!("cargo:warning=exe-icon: failed to embed civis.ico: {e}");
        }
    }
}
