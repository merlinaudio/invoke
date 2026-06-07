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

	let handle = refcon as usize as NotificationRegistrationHandle;

	let known_notifications = KNOWN_NOTIFICATIONS.pin();
	let Some(notification_registration) = known_notifications.get(&handle) else {
		log::error!("Notification registration not found for handle: {handle}");
		return;
	};

	unsafe {
		// macOS passes NULL info for most notifications (focusedUIElementChanged,
		// valueChanged, etc.). The objc2 binding incorrectly types this as NonNull —
		// Apple's C header is `CFDictionaryRef` which is nullable. Round-trip through
		// usize to strip the nonnull invariant so the null check isn't optimized away.
		let info_ptr = info.as_ptr() as usize as *const CFDictionary;
		let event = Event::Notification {
			element,
			name: notification.as_ref().to_string().parse().unwrap(),
			info: if info_ptr.is_null() {
				None
			} else {
				Some(cfdictionary_to_hashmap(&*info_ptr))
			},
		};

		notification_registration.on_event.call((event,));
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
	on_event: Box<dyn Fn(Event) + Send + Sync>,
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

impl ObserverGuard<AccessibilityObserver> {
	pub fn update_pid(&mut self, new_pid: u32) -> Result<(), Error> {
		if self.inner.pid.is_some_and(|pid| pid == new_pid) {
			return Ok(());
		}

		match self.sleep() {
			// There may be no old AXObserver to remove, for example before the app
			// has launched. That is fine; update_pid is about creating the new one.
			Err(Error::ObserverNotCreated) => Ok(()),
			res => res,
		}?;

		let observer = AccessibilityObserver::create_observer(new_pid)?;

		for (handle, reg) in self.inner.notifications.iter() {
			let element = reg.element_handle.inner();
			let element = element.as_ref().ok_or(Error::ElementNotFound)?;

			let notification_cfstring = reg.notification.to_CFString();

			unsafe { observer.add_notification(element, &notification_cfstring, *handle as usize as *mut c_void) }
				.to_result()
				.inspect_err(|e| {
					log::error!(
						"Failed to add notification {:?} to {:?} (element={element:?}): {e:?}",
						reg.notification,
						reg.element_handle,
					)
				})?;
		}

		self.inner.observer = Some(observer);
		self.inner.pid = Some(new_pid);

		self.listen()?;

		Ok(())
	}

	pub fn add_notification(
		&mut self,
		element_handle: ElementHandle,
		notification: Notification,
		on_event: Box<dyn Fn(Event) + Send + Sync>,
	) -> Result<NotificationRegistrationHandle, Error> {
		let registration = Arc::new(NotificationRegistration {
			element_handle,
			notification: notification.clone(),
			on_event,
		});

		let handle = KNOWN_NOTIFICATIONS.insert(registration.clone());

		self.inner.notifications.insert(handle, registration);

		log::info!("Observing {notification:?} for {element_handle:?} (observer={:?})", self.inner.observer);

		// The important part already happened above:
		// inner.notifications and KNOWN_NOTIFICATIONS now remember this request.
		//
		// Try to attach it to the current AXObserver too, but do not fail the
		// whole call if macOS says no right now. Some elements only become
		// observable later, and update_pid retries stored notifications.
		if let Some(observer) = &self.inner.observer
			&& let Some(element) = element_handle.inner().as_ref()
		{
			_ = unsafe { observer.add_notification(element, &notification.to_CFString(), handle as usize as *mut c_void) }
				.to_result()
				.inspect_err(|e| log::error!("Failed to add notification {notification:?} to {element_handle:?} (element={element:?}): {e:?}",));
		}

		Ok(handle)
	}

	pub fn remove_notification(&mut self, handle: NotificationRegistrationHandle) -> Result<(), Error> {
		let known_notifications = KNOWN_NOTIFICATIONS.pin();
		let registration = known_notifications.remove(&handle).ok_or(Error::NotificationNotFound)?;
		self.inner.notifications.remove(&handle);

		if let Some(observer) = &self.inner.observer {
			let ax_error =
				unsafe { observer.remove_notification(registration.element_handle.inner().as_ref().unwrap(), &registration.notification.to_CFString()) };
			if ax_error != AXError::Success {
				log::warn!(
					"Removed notification for element #{} from our state, but AXObserverRemoveNotification failed: {ax_error:?}",
					registration.element_handle
				);
			}
		};

		log::info!("Removed notification {:?} from {:?}", registration.notification, self.inner.observer);

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
