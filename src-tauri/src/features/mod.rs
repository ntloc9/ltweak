//! Registry of optional features. Each feature is a self-contained module
//! with a `start()` entrypoint. Add new features here as they land.

#[cfg(target_os = "macos")]
pub mod ctrl_scroll_zoom;
#[cfg(target_os = "macos")]
pub mod finder_sidebutton;

use std::sync::atomic::{AtomicBool, Ordering};

/// Master switch for the whole app. When off, features pass events through
/// untouched instead of remapping them.
static ENABLED: AtomicBool = AtomicBool::new(true);

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

/// Start every enabled feature. Called once at app launch.
pub fn start() {
    #[cfg(target_os = "macos")]
    ctrl_scroll_zoom::start();
    #[cfg(target_os = "macos")]
    finder_sidebutton::start();
}