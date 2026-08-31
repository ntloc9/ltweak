//! Shared low-level helpers for CoreGraphics event taps and synthetic events.
//!
//! Features register a raw callback for a set of event types; the tap runs on
//! its own thread with a run loop. macOS only.

use std::os::raw::c_void;

use core_foundation::base::TCFType;
use core_foundation::mach_port::{CFMachPort, CFMachPortRef};
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventField, CGEventFlags, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::sys::CGEventRef;

pub type TapCallback =
    unsafe extern "C" fn(CGEventTapProxy, CGEventType, CGEventRef, *const c_void) -> CGEventRef;

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        events_of_interest: u64,
        callback: TapCallback,
        user_info: *const c_void,
    ) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventGetFlags(event: CGEventRef) -> CGEventFlags;
    fn CGEventGetIntegerValueField(event: CGEventRef, field: CGEventField) -> i64;
}

/// Read the modifier flags on a raw event.
pub fn event_flags(event: CGEventRef) -> CGEventFlags {
    unsafe { CGEventGetFlags(event) }
}

/// Read an integer field on a raw event (scroll deltas, mouse button number…).
pub fn event_integer_field(event: CGEventRef, field: CGEventField) -> i64 {
    unsafe { CGEventGetIntegerValueField(event, field) }
}

/// Post a synthetic key press (down + up) at the annotated session tap.
pub fn post_key(keycode: u16, flags: CGEventFlags) {
    let source = match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
        Ok(s) => s,
        Err(()) => return,
    };
    if let Ok(key_down) = CGEvent::new_keyboard_event(source.clone(), keycode, true) {
        key_down.set_flags(flags);
        key_down.post(CGEventTapLocation::AnnotatedSession);
    }
    if let Ok(key_up) = CGEvent::new_keyboard_event(source, keycode, false) {
        key_up.set_flags(flags);
        key_up.post(CGEventTapLocation::AnnotatedSession);
    }
}

/// Install a head-insert event tap for `events` on a dedicated thread.
pub fn run_event_tap(events: &[CGEventType], callback: TapCallback) {
    if !unsafe { accessibility_sys::AXIsProcessTrusted() } {
        eprintln!(
            "[events] ACCESSIBILITY PERMISSION MISSING.\n\
             Enable it: System Settings > Privacy & Security > Accessibility > add your terminal,\n\
             then restart the app."
        );
    }

    let mut mask: u64 = 0;
    for event_type in events {
        mask |= 1u64 << (*event_type as u32);
    }

    std::thread::spawn(move || {
        let tap_ref = unsafe {
            CGEventTapCreate(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                mask,
                callback,
                std::ptr::null(),
            )
        };

        if tap_ref.is_null() {
            eprintln!("Could not create event tap (Accessibility permission required)");
            return;
        }

        let port = unsafe { CFMachPort::wrap_under_create_rule(tap_ref) };

        let source = match port.create_runloop_source(0) {
            Ok(source) => source,
            Err(()) => {
                eprintln!("Could not create run loop source");
                return;
            }
        };

        let run_loop = CFRunLoop::get_current();
        unsafe {
            CGEventTapEnable(port.as_concrete_TypeRef(), true);
            run_loop.add_source(&source, kCFRunLoopCommonModes);
        }
        CFRunLoop::run_current();
    });
}