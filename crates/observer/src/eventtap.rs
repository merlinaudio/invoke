//! CoreGraphics event tap observer.
//!
//! This module is where raw [`CGEvent`] traffic becomes the crate's mouse and
//! keyboard event surface. Callers get semantic events like key down, mouse up,
//! scroll, and modifier changes; they do not need to read CoreGraphics fields or
//! handle tap-disabled notifications themselves.
//!
//! The callback returns [`CallbackResult`] because CoreGraphics asks the tap,
//! synchronously, what should happen to the event: pass it through, drop it, or
//! mutate it. That policy belongs here because it is part of event tap
//! semantics.
//!
//! This observer still does not know about shortcuts or app commands. Apps map
//! [`Event`] into their own trigger language.

use crate::{Observer, ObserverGuard};
use hid::{
	keyboard::{Key, Modifiers},
	mouse::MouseButton,
};
use objc2_core_foundation::{CFMachPort, CFRetained, CFRunLoop, CFRunLoopSource, kCFRunLoopCommonModes};
use objc2_core_graphics::{CGEvent, CGEventField, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType};
use smallvec::SmallVec;
use std::{
	ffi::c_void,
	mem::variant_count,
	ptr::{NonNull, null_mut},
};
use thiserror::Error;

pub struct EventtapObserver {
	runloop_source: CFRetained<CFRunLoopSource>,
	mach_port: CFRetained<CFMachPort>,
}

#[derive(Debug, Error)]
pub enum Error {
	#[error("AX permissions not available")]
	NotTrusted,

	#[error("Failed to create event tap")]
	CreationFailed,

	#[error("No current run loop available")]
	NoCurrentRunLoop,
}

// TODO maybe implement CGEvent::from(Event) and allow using it in `instruction`s

#[derive(Clone, PartialEq, PartialOrd, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Event {
	MouseUp(MouseButton),
	MouseDown(MouseButton),
	MouseDragged(MouseButton),
	MouseMoved { x: f64, y: f64 },
	MouseWheelScroll { y: f64, x: f64, z: f64 },
	KeyboardKeyDown(Key),
	KeyboardKeyUp(Key),
	KeyboardModifiersChanged(Modifiers),
	// TabletPointer, TODO implement HID tablet pen
	// TabletProximity, TODO implement HID tablet pen
}

#[derive(Clone, Debug, PartialEq)]
pub enum CallbackResult {
	Passthrough,
	Drop,
	ApplyChanges(SmallVec<[Change; variant_count::<Change>()]>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Change {
	SetModifiers(Modifiers),
}

/// Why macOS pulled our tap out of the event stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisableReason {
	/// The callback (or the run loop it lives on) took too long to return.
	Timeout,
	/// Disabled at the system's request (e.g. secure input).
	UserInput,
}

type OnEventCallback = dyn Fn(Event) -> CallbackResult;
type OnDisabledCallback = dyn Fn(DisableReason);

struct CallbackContext {
	on_event: Box<OnEventCallback>,
	on_disabled: Box<OnDisabledCallback>,
	our_pid: i64,
}

const NULL: *mut CGEvent = null_mut();

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn INVEventtapObserverCallback(
	_proxy: CGEventTapProxy,
	cg_event_type: CGEventType,
	cg_event: NonNull<CGEvent>,
	user_info: *mut c_void,
) -> *mut CGEvent {
	// Returning...
	//
	//           NULL = Drop event
	// Original event = Pass through. Can be modified!
	//      New event = Replace original
	//
	// Ref: https://developer.apple.com/documentation/coregraphics/cgeventtapcallback?language=objc#Discussion

	// Pass through as-is
	let passthrough = cg_event.as_ptr();

	let context = unsafe { (user_info as *mut CallbackContext).as_ref() };
	let Some(context) = context else {
		log::error!("Callback context is null");
		return passthrough;
	};

	// This MUST be handled before cg_event is ever dereferenced. Even though we're passed NonNull,
	// the CGEvent pointer MAY BE NULL. At time of writing, this only happens with TapDisabledByTimeout or TapDisabledByUserInput.
	// Makes sense: there is no event to handle - it's just a notification from macOS that the event tap has been disabled.
	//
	// ...So we're actually returning NULL here, usually, because *mut CGEvent is NULL.
	//
	// If NULL ISN'T handled here, we DELIBERATELY PANIC BELOW! (Because catching the relevant CGEventTypes here indicates a bug)
	// So handle it here!
	if let CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput = cg_event_type {
		let reason = match cg_event_type {
			CGEventType::TapDisabledByTimeout => DisableReason::Timeout,
			_ => DisableReason::UserInput,
		};
		// macOS disabled the tap — either our callback ran long (Timeout), or the system
		// grabbed input (UserInput, e.g. an app becoming frontmost). Recovery policy belongs
		// to whoever owns the observer, so we just report it; the owner re-enables (or not).
		context.on_disabled.call((reason,));
		return passthrough;
	}

	// Actual handling begins here!
	let event = unsafe {
		let cg_event = Some(cg_event.as_ref());

		match cg_event {
			Some(cg_event) => log::trace!("CGEvent: {:?}", cg_event),
			None => panic!("CGEvent is NULL, but never should be at this point."), // NULL usually when TapDisabledByTimeout or TapDisabledByUserInput
		}

		let source_pid = CGEvent::integer_value_field(cg_event, CGEventField::EventSourceUnixProcessID);

		if source_pid == context.our_pid {
			// Ignore all events from ourselves. Otherwise infinite loops abound!
			return passthrough;
		}

		match cg_event_type {
			CGEventType::LeftMouseUp => Event::MouseUp(MouseButton::Left),
			CGEventType::LeftMouseDown => Event::MouseDown(MouseButton::Left),
			CGEventType::LeftMouseDragged => Event::MouseDragged(MouseButton::Left),

			CGEventType::RightMouseUp => Event::MouseUp(MouseButton::Right),
			CGEventType::RightMouseDown => Event::MouseDown(MouseButton::Right),
			CGEventType::RightMouseDragged => Event::MouseDragged(MouseButton::Right),

			CGEventType::OtherMouseUp => Event::MouseUp(get_mouse_button(cg_event)),
			CGEventType::OtherMouseDown => Event::MouseDown(get_mouse_button(cg_event)),
			CGEventType::OtherMouseDragged => Event::MouseDragged(get_mouse_button(cg_event)),

			CGEventType::MouseMoved => {
				let location = CGEvent::location(cg_event);
				Event::MouseMoved { x: location.x, y: location.y }
			}

			CGEventType::KeyDown => match get_event_keyboard_key(cg_event) {
				(Some(key), _) => Event::KeyboardKeyDown(key),
				(None, unicode_string) => {
					log::error!("Could not convert event's unichar {unicode_string:?} to a KeyboardKey");
					return passthrough;
				}
			},
			CGEventType::KeyUp => match get_event_keyboard_key(cg_event) {
				(Some(key), _) => Event::KeyboardKeyUp(key),
				(None, unicode_string) => {
					log::error!("Could not convert event's unichar {unicode_string:?} to a KeyboardKey");
					return passthrough;
				}
			},

			CGEventType::FlagsChanged => Event::KeyboardModifiersChanged(CGEvent::flags(cg_event).into()),

			CGEventType::ScrollWheel => Event::MouseWheelScroll {
				y: CGEvent::double_value_field(cg_event, CGEventField::ScrollWheelEventDeltaAxis1),
				x: CGEvent::double_value_field(cg_event, CGEventField::ScrollWheelEventDeltaAxis2),
				z: CGEvent::double_value_field(cg_event, CGEventField::ScrollWheelEventDeltaAxis3),
			},

			_ => return passthrough,
		}
	};

	let started = std::time::Instant::now();
	let result = context.on_event.call((event,));
	let elapsed = started.elapsed();
	// macOS disables the tap if a callback runs long. Surface slow handlers before that happens.
	if elapsed > std::time::Duration::from_millis(250) {
		log::warn!("Eventtap callback took {elapsed:?} — risks TapDisabledByTimeout");
	}

	match result {
		CallbackResult::Passthrough => passthrough,
		CallbackResult::Drop => NULL,
		CallbackResult::ApplyChanges(changes) => {
			for change in changes {
				match change {
					Change::SetModifiers(modifiers) => {
						unsafe { CGEvent::set_flags(Some(cg_event.as_ref()), modifiers.into()) };
					}
				}
			}

			cg_event.as_ptr()
		}
	}
}

fn get_mouse_button(cg_event: Option<&CGEvent>) -> MouseButton {
	MouseButton::Other {
		code: CGEvent::integer_value_field(cg_event, CGEventField::MouseEventButtonNumber) as u8,
	}
}

impl EventtapObserver {
	pub fn new(
		on_event: impl Fn(Event) -> CallbackResult + 'static + Send + Sync,
		on_disabled: impl Fn(DisableReason) + 'static + Send + Sync,
	) -> Result<ObserverGuard<Self>, Error> {
		let event_mask = {
			CGEventType::LeftMouseDown.0
				| CGEventType::LeftMouseUp.0
				| CGEventType::RightMouseDown.0
				| CGEventType::RightMouseUp.0
				| CGEventType::MouseMoved.0
				| CGEventType::LeftMouseDragged.0
				| CGEventType::RightMouseDragged.0
				| CGEventType::KeyDown.0
				| CGEventType::KeyUp.0
				| CGEventType::FlagsChanged.0
				| CGEventType::ScrollWheel.0
				| CGEventType::TabletPointer.0
				| CGEventType::TabletProximity.0
				| CGEventType::OtherMouseDown.0
				| CGEventType::OtherMouseUp.0
				| CGEventType::OtherMouseDragged.0
				| CGEventType::TapDisabledByTimeout.0
				| CGEventType::TapDisabledByUserInput.0
		};

		let context = CallbackContext {
			on_event: Box::new(on_event),
			on_disabled: Box::new(on_disabled),
			our_pid: std::process::id().into(),
		};

		let mach_port = unsafe {
			match CGEvent::tap_create(
				CGEventTapLocation::HIDEventTap,
				CGEventTapPlacement::HeadInsertEventTap,
				CGEventTapOptions::Default,
				event_mask.into(),
				Some(INVEventtapObserverCallback),
				Box::into_raw(Box::new(context)).cast(),
			) {
				Some(mp) => mp,
				None => return Err(Error::CreationFailed),
			}
		};

		let runloop_source = match CFMachPort::new_run_loop_source(None, Some(&mach_port), 0) {
			Some(rs) => rs,
			None => return Err(Error::CreationFailed),
		};

		Ok(ObserverGuard {
			inner: EventtapObserver { mach_port, runloop_source },
		})
	}
}

fn get_event_keyboard_key(e: Option<&CGEvent>) -> (Option<Key>, i64) {
	// let unicode_string = unsafe {
	// 	let mut out_unicode_string = [0; 4];
	// 	CGEvent::keyboard_get_unicode_string(e, 4, &mut 4, out_unicode_string.as_mut_ptr());
	// 	out_unicode_string
	// };

	// let unicode = String::from_utf16_lossy(&unicode_string);
	// log::debug!("unicode: {unicode}");

	let keycode = CGEvent::integer_value_field(e, CGEventField::KeyboardEventKeycode);

	(Key::try_from(keycode as u16).ok(), keycode)
}

impl Observer for EventtapObserver {
	type Error = Error;
	type Event = Event;

	// This means "make sure eventtap is listening".
	// Calling it twice must not put the same tap into the run loop twice.
	fn listen(&self) -> Result<(), Self::Error> {
		use objc2_application_services::AXIsProcessTrusted;

		if !unsafe { AXIsProcessTrusted() } {
			return Err(Error::NotTrusted);
		}

		let Some(runloop) = CFRunLoop::main() else {
			return Err(Error::NoCurrentRunLoop);
		};

		let mode = unsafe { kCFRunLoopCommonModes };

		if !runloop.contains_source(Some(&self.runloop_source), mode) {
			runloop.add_source(Some(&self.runloop_source), mode);
		}

		CGEvent::tap_enable(&self.mach_port, true);

		Ok(())
	}

	fn sleep(&self) -> Result<(), Self::Error> {
		let Some(runloop) = CFRunLoop::main() else {
			return Err(Error::NoCurrentRunLoop);
		};

		let mode = unsafe { kCFRunLoopCommonModes };

		if runloop.contains_source(Some(&self.runloop_source), mode) {
			runloop.remove_source(Some(&self.runloop_source), mode);
		}

		CGEvent::tap_enable(&self.mach_port, false);

		Ok(())
	}
}
