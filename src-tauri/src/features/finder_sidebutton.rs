//! Feature: mouse back/forward buttons navigate in Finder.
//!
//! Button 3 (back) sends Cmd+[ and button 4 (forward) sends Cmd+], which
//! Finder maps to previous/next folder. This feature only runs while Finder
//! is the frontmost app; in any other app the button events pass through
//! untouched, and while Finder is active the original events are swallowed.
//! macOS only.

use core_graphics::event::{CGEventFlags, CGEventType, EventField};

use crate::cg_tap;

/// Mouse button numbers: 3 = back, 4 = forward.
const BUTTON_BACK: i64 = 3;
const BUTTON_FORWARD: i64 = 4;

/// macOS keycodes: 33 = "[" , 30 = "]" (with Cmd = back / forward in Finder).
const KEYCODE_BRACKET_LEFT: u16 = 33;
const KEYCODE_BRACKET_RIGHT: u16 = 30;

/// This feature only ever runs while Finder is frontmost.
const TARGET_BUNDLE_ID: &str = "com.apple.finder";

fn is_finder_frontmost() -> bool {
    let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
    match workspace.frontmostApplication() {
        Some(app) => match app.bundleIdentifier() {
            Some(bundle) => bundle.to_string() == TARGET_BUNDLE_ID,
            None => false,
        },
        None => false,
    }
}

fn navigate(back: bool) {
    let keycode = if back {
        KEYCODE_BRACKET_LEFT
    } else {
        KEYCODE_BRACKET_RIGHT
    };
    cg_tap::post_key(keycode, CGEventFlags::CGEventFlagCommand);
}

unsafe extern "C" fn raw_button_callback(
    _proxy: core_graphics::event::CGEventTapProxy,
    event_type: CGEventType,
    event: core_graphics::sys::CGEventRef,
    _user_info: *const std::os::raw::c_void,
) -> core_graphics::sys::CGEventRef {
    if !matches!(event_type, CGEventType::OtherMouseDown | CGEventType::OtherMouseUp) {
        return event;
    }
    // When the app is disabled, let buttons behave normally.
    if !crate::features::is_enabled() {
        return event;
    }

    let button = cg_tap::event_integer_field(event, EventField::MOUSE_EVENT_BUTTON_NUMBER);
    let is_back = button == BUTTON_BACK;
    let is_forward = button == BUTTON_FORWARD;

    // Only run while Finder is frontmost; everywhere else pass through.
    if (is_back || is_forward) && is_finder_frontmost() {
        if matches!(event_type, CGEventType::OtherMouseDown) {
            navigate(is_back);
            println!(
                "[finder] mouse {} -> Cmd+{}",
                if is_back { "back" } else { "forward" },
                if is_back { "[" } else { "]" }
            );
        }
        // Swallow the original button event (both down and up).
        return std::ptr::null_mut();
    }

    // Not ours to handle: pass the original event through untouched.
    event
}

pub fn start() {
    cg_tap::run_event_tap(
        &[CGEventType::OtherMouseDown, CGEventType::OtherMouseUp],
        raw_button_callback,
    );
    println!("[finder] monitoring started (mouse back/forward -> Finder navigation, Finder only)");
}