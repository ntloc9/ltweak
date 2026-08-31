//! Feature: ctrl+scroll acts as Cmd+plus / Cmd+minus (system zoom).
//!
//! The original ctrl+scroll event is swallowed, so apps only see the
//! remapped keystrokes. macOS only.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use core_graphics::event::{CGEventFlags, CGEventType, EventField};

use crate::cg_tap;

/// macOS keycodes: 24 = "=" ('+' with shift), 27 = "-".
const KEYCODE_PLUS: u16 = 24;
const KEYCODE_MINUS: u16 = 27;

/// Minimum gap between remapped keystrokes (ms). Low enough that a fast
/// scroll produces continuous zoom, high enough to avoid spamming.
const COOLDOWN_MS: u64 = 35;

/// Maximum zoom presses per single scroll event.
const MAX_REPEATS: i64 = 6;

/// Scroll delta magnitude divided by this yields zoom presses per event, so
/// fast flicks zoom further than slow drags.
const REPEAT_DIVISOR: i64 = 2;

static LAST_FIRE_MS: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn post_zoom(zoom_in: bool) {
    let keycode = if zoom_in { KEYCODE_PLUS } else { KEYCODE_MINUS };
    cg_tap::post_key(keycode, CGEventFlags::CGEventFlagCommand);
}

fn post_zoom_repeated(zoom_in: bool, count: i64) {
    for _ in 0..count.min(MAX_REPEATS).max(1) {
        post_zoom(zoom_in);
    }
}

unsafe extern "C" fn raw_scroll_callback(
    _proxy: core_graphics::event::CGEventTapProxy,
    event_type: CGEventType,
    event: core_graphics::sys::CGEventRef,
    _user_info: *const std::os::raw::c_void,
) -> core_graphics::sys::CGEventRef {
    if matches!(event_type, CGEventType::ScrollWheel) {
        // When the app is disabled, let the original scroll through untouched.
        if !crate::features::is_enabled() {
            return event;
        }
        let ctrl_held = cg_tap::event_flags(event).contains(CGEventFlags::CGEventFlagControl);
        let delta = cg_tap::event_integer_field(event, EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_1);

        if ctrl_held {
            let now = now_ms();
            let last = LAST_FIRE_MS.load(Ordering::Relaxed);
            if now.saturating_sub(last) >= COOLDOWN_MS {
                if delta != 0 {
                    let zoom_in = delta > 0;
                    let magnitude = delta.abs();
                    // Larger scrolls (fast flicks) map to more zoom presses.
                    let count = (magnitude / REPEAT_DIVISOR).clamp(1, MAX_REPEATS);
                    post_zoom_repeated(zoom_in, count);
                    if zoom_in {
                        println!("[zoom] ctrl+scroll up -> Cmd '+' x{count}");
                    } else {
                        println!("[zoom] ctrl+scroll down -> Cmd '-' x{count}");
                    }
                }
                LAST_FIRE_MS.store(now, Ordering::Relaxed);
            }
            // Return NULL to swallow the scroll event entirely.
            return std::ptr::null_mut();
        }
    }
    // Not ours to handle: pass the original event through untouched.
    event
}

pub fn start() {
    cg_tap::run_event_tap(&[CGEventType::ScrollWheel], raw_scroll_callback);
    println!("[zoom] monitoring started (ctrl+scroll -> zoom, original scroll swallowed)");
}