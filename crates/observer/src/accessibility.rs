//! Accessibility notification observer.
//!
//! This module wraps AXObserver registration for specific accessibility
//! elements. It reports the notification name, element, and optional AX payload;
//! it does not interpret those notifications as app commands.
//!
//! The observer keeps AX permissions, callback registration, and nullable
//! platform payload handling inside this crate so callers can work with one
//! Rust event shape.

use crate::{Observer, ObserverGuard};
use common::accessibility::value::cfdictionary_to_hashmap;
use common::{
	accessibility::{ElementHandle, Notification, Value},
	handle_map::ConcurrentHandleMapU32,
};
use objc2_application_services::{AXError, AXIsProcessTrusted, AXObserver, AXUIElement};
use objc2_core_foundation::{CFDictionary, CFRetained, CFRunLoop, CFString, Type, kCFRunLoopCommonModes};
use std::{
	collections::HashMap,
	ffi::c_void,
	hash::Hash,
	ptr::{NonNull, null_mut},
	sync::{Arc, LazyLock},
};
use thiserror::Error;

static KNOWN_NOTIFICATIONS: LazyLock<ConcurrentHandleMapU32<Arc<NotificationRegistration>>> = LazyLock::new(ConcurrentHandleMapU32::new);

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum Error {
	#[error("AX permissions not available")]
	NotTrusted,

	#[error("Failed to create accessibility observer")]
	CreationFailed,

	#[error("AXError: {0}")]
	AX(i32),

	#[error("No current run loop available")]
	NoCurrentRunLoop,

	#[error("Notification not found")]
	NotificationNotFound,

	#[error("Element not found")]
	ElementNotFound,

	#[error("Observer not created")]
	ObserverNotCreated,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Event {
	Notification {
		name: Notification,
		element: NonNull<AXUIElement>,
		info: Option<HashMap<String, Value>>,
	},
}

#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn INVAccessibilityObserverCallback(
	_observer: NonNull<AXObserver>,
	element: NonNull<AXUIElement>,
	notification: NonNull<CFString>,
	info: NonNull<CFDictionary>,
	refcon: *mut c_void,
) {
	log::info!("Accessibility observer callback: {element:?}, {notification:?}, {info:?}, {refcon:?}");

	// When we register, we pass the element handle to macOS as the refcon.
	// macOS hands it back here, along with the notification name.
	// So we can look up every subscriber watching this element for this notification.
	let element_handle = ElementHandle::from(refcon as usize as u32);

	unsafe {
		let notification: Notification = notification.as_ref().to_string().parse().unwrap();

		// macOS passes NULL info for most notifications (focusedUIElementChanged,
		// valueChanged, etc.). The objc2 binding incorrectly types this as NonNull —
		// Apple's C header is `CFDictionaryRef` which is nullable. Round-trip through
		// usize to strip the nonnull invariant so the null check isn't optimized away.
		let info_ptr = info.as_ptr() as usize as *const CFDictionary;

		let event = Event::Notification {
			element,
			name: notification.clone(),
			info: if info_ptr.is_null() {
				None
			} else {
				Some(cfdictionary_to_hashmap(&*info_ptr))
			},
		};

		// Collect the matching subscribers before we call them.
		// Otherwise we would be reading the map while a handler adds or removes a subscription.
		let subscribers: Vec<_> = KNOWN_NOTIFICATIONS
			.pin()
			.iter()
			.filter(|(_, reg)| reg.element_handle == element_handle && reg.notification == notification)
			.map(|(_, reg)| reg.clone())
			.collect();

		for subscriber in subscribers {
			subscriber.on_event.call((&event,));
		}
	}
}

pub type NotificationRegistrationHandle = u32;

pub fn get_element_handle_for_registration(handle: NotificationRegistrationHandle) -> Option<ElementHandle> {
	let known_notifications = KNOWN_NOTIFICATIONS.pin();
	known_notifications.get(&handle).map(|r| r.element_handle)
}

struct NotificationRegistration {
	element_handle: ElementHandle,
	notification: Notification,
	on_event: Box<dyn Fn(&Event) + Send + Sync>,
}

pub struct AccessibilityObserver {
	pid: Option<u32>,
	observer: Option<CFRetained<AXObserver>>,
	notifications: HashMap<NotificationRegistrationHandle, Arc<NotificationRegistration>>,
}

impl AccessibilityObserver {
	pub fn new() -> Result<ObserverGuard<Self>, Error> {
		Ok(ObserverGuard {
			inner: Self {
				pid: None,
				observer: None,
				notifications: HashMap::new(),
			},
		})
	}

	fn create_observer(pid: u32) -> Result<CFRetained<AXObserver>, Error> {
		let trusted = unsafe { AXIsProcessTrusted() };
		if !trusted {
			return Err(Error::NotTrusted);
		}

		let pid = pid
			.try_into()
			.inspect_err(|e| log::error!("Failed to convert pid to i32: {e:?}"))
			.map_err(|_| Error::CreationFailed)?;

		let mut out_observer_raw = null_mut::<AXObserver>();
		let out_observer = NonNull::new(&mut out_observer_raw).unwrap();

		unsafe { AXObserver::create_with_info_callback(pid, Some(INVAccessibilityObserverCallback), out_observer) }.to_result()?;

		let observer = unsafe { out_observer.as_ref().as_ref() }.ok_or(Error::CreationFailed)?;

		log::info!("create_observer(): Created {observer:?}");

		Ok(observer.retain())
	}
}

trait ToResult {
	fn to_result(self) -> Result<(), Error>;
}

impl ToResult for AXError {
	fn to_result(self) -> Result<(), Error> {
		if self != AXError::Success { Err(Error::AX(self.0)) } else { Ok(()) }
	}
}

const AX_ERROR_NOTIFICATION_ALREADY_REGISTERED: i32 = -25209;

// Ask macOS to start sending us this notification.
// We register once per `(element,notification)` and pass the element handle as the refcon, so the callback can find every subscriber.
// macOS returns -25209 when it is already registered, which is what we want, so we treat that as success and never track who is first.
fn register(observer: &CFRetained<AXObserver>, element_handle: ElementHandle, notification: &Notification) -> Result<(), Error> {
	let element = element_handle.inner();
	let element = element.as_ref().ok_or(Error::ElementNotFound)?;
	let refcon = u32::from(element_handle) as usize as *mut c_void;
	match unsafe { observer.add_notification(element, &notification.to_CFString(), refcon) }.to_result() {
		Err(Error::AX(AX_ERROR_NOTIFICATION_ALREADY_REGISTERED)) => Ok(()),
		result => result,
	}
}

impl ObserverGuard<AccessibilityObserver> {
	pub fn update_pid(&mut self, new_pid: u32) -> Result<(), Error> {
		// A new process needs a fresh AXObserver.
		// The same process keeps its observer, and we just re-register below.
		if self.inner.pid != Some(new_pid) {
			match self.sleep() {
				// There may be no old AXObserver to remove, for example before the app
				// has launched. That is fine; we are creating the new one.
				Err(Error::ObserverNotCreated) => Ok(()),
				res => res,
			}?;

			self.inner.observer = Some(AccessibilityObserver::create_observer(new_pid)?);
			self.inner.pid = Some(new_pid);
			self.listen()?;
		}

		// Register our notifications with the observer.
		// macOS can refuse one right after the app launches, with -25204.
		// This runs on every activation, so a refused one is retried once macOS is ready.
		// register() is idempotent, so already-registered ones are fine.
		let Some(observer) = &self.inner.observer else { return Ok(()) };
		for r in self.inner.notifications.values() {
			_ = register(observer, r.element_handle, &r.notification)
				.inspect_err(|e| log::error!("Failed to register {:?} on {:?}: {e:?}", r.notification, r.element_handle));
		}

		Ok(())
	}

	pub fn add_notification(
		&mut self,
		element_handle: ElementHandle,
		notification: Notification,
		on_event: Box<dyn Fn(&Event) + Send + Sync>,
	) -> Result<NotificationRegistrationHandle, Error> {
		let registration = Arc::new(NotificationRegistration {
			element_handle,
			notification: notification.clone(),
			on_event,
		});
		let handle = KNOWN_NOTIFICATIONS.insert(registration.clone());
		self.inner.notifications.insert(handle, registration);

		// macOS sends every event to our one callback, which passes it to all the subscribers.
		// macOS can refuse right after the app launches. update_pid registers again when the app next activates.
		if let Some(observer) = &self.inner.observer {
			_ = register(observer, element_handle, &notification)
				.inspect_err(|e| log::error!("Failed to observe {notification:?} on {element_handle:?}: {e:?}"));
		}

		Ok(handle)
	}

	pub fn remove_notification(&mut self, handle: NotificationRegistrationHandle) -> Result<(), Error> {
		let registration = KNOWN_NOTIFICATIONS.pin().remove(&handle).cloned().ok_or(Error::NotificationNotFound)?;
		self.inner.notifications.remove(&handle);

		// While other subscribers still want this (element, notification), leave the macOS registration alone...
		let others_remain = self
			.inner
			.notifications
			.values()
			.any(|r| r.element_handle == registration.element_handle && r.notification == registration.notification);

		// ...but if nobody else subscribes (i.e. we just removed the last subscriber),
		// remove the macOS registration.
		if !others_remain
			&& let Some(observer) = &self.inner.observer
			&& let Some(element) = registration.element_handle.inner().as_ref()
		{
			let ax_error = unsafe { observer.remove_notification(element, &registration.notification.to_CFString()) };
			if ax_error != AXError::Success {
				log::warn!(
					"Stopped observing {:?} in our state, but AXObserverRemoveNotification failed: {ax_error:?}",
					registration.notification
				);
			}
		}

		Ok(())
	}
}

impl From<AXError> for Error {
	fn from(error: AXError) -> Self {
		Error::AX(error.0)
	}
}

impl Observer for AccessibilityObserver {
	type Error = Error;
	type Event = Event;

	// Put the AXObserver into the main run loop.
	// Calling it twice must not add the same observer twice.
	fn listen(&self) -> Result<(), Self::Error> {
		let Some(runloop) = CFRunLoop::main() else {
			return Err(Error::NoCurrentRunLoop);
		};

		let Some(observer) = &self.observer else {
			return Err(Error::ObserverNotCreated);
		};

		unsafe {
			runloop.add_source(Some(&observer.run_loop_source()), kCFRunLoopCommonModes);
		}

		log::info!("listen(): Added {:?} to run loop", self.observer);

		Ok(())
	}

	fn sleep(&self) -> Result<(), Self::Error> {
		let Some(runloop) = CFRunLoop::main() else {
			return Err(Error::NoCurrentRunLoop);
		};

		let Some(observer) = &self.observer else {
			return Err(Error::ObserverNotCreated);
		};

		unsafe {
			runloop.remove_source(Some(&observer.run_loop_source()), kCFRunLoopCommonModes);
		}

		log::info!("sleep(): Removed {:?} from run loop", self.observer);

		Ok(())
	}
}
