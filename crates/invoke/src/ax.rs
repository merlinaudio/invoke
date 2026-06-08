use std::{
	collections::HashMap,
	sync::{LazyLock, RwLock},
};

use common::{
	accessibility::{ElementHandle, Notification},
	main_thread::MainThread,
};
use observer::{
	ObserverGuard,
	accessibility::{AccessibilityObserver, NotificationRegistrationHandle},
};

use crate::monitor::{self, AppHandle};

/// AX observers keyed by app handle.
///
/// This stores ObserverGuards, not live AXObservers directly. The guard remembers
/// requested notifications even when the app is not running yet.
static AX_OBSERVERS: LazyLock<RwLock<HashMap<AppHandle, MainThread<ObserverGuard<AccessibilityObserver>>>>> = LazyLock::new(Default::default);

#[derive(Debug)]
pub enum Error {
	Accessibility(observer::accessibility::Error),

	AppNotRunning,

	AppNotFound,

	ElementNotFound,
}

pub fn register_observer(app_handle: AppHandle) -> Result<(), Error> {
	let mut ax_observers_w = AX_OBSERVERS.write().unwrap();

	let mut observer = AccessibilityObserver::new().map_err(Error::Accessibility)?;
	if let Some(pid) = app_handle.pid() {
		// Store the observer guard even if this first attach fails. When the app
		// activates again, monitor::update_process calls ax::update_process and retries.
		if let Err(e) = observer.update_pid(pid) {
			log::warn!("Cannot create AXObserver for {:?}: {e:?}. Will retry when app activates.", app_handle);
		}
	}

	ax_observers_w.insert(app_handle, MainThread::new(observer));

	log::info!("Registered accessibility observer for app handle: {:?}.", app_handle);

	Ok(())
}

pub fn update_process(app_handle: AppHandle) -> Result<(), Error> {
	let mut ax_observers_w = AX_OBSERVERS.write().unwrap();
	let observer = ax_observers_w.get_mut(&app_handle).ok_or(Error::AppNotRunning)?;

	observer
		.update_pid(app_handle.pid().ok_or(Error::AppNotRunning)?)
		.map_err(Error::Accessibility)?;

	Ok(())
}

pub fn unregister_observer(app_handle: AppHandle) -> Result<(), Error> {
	// Dropping the guard calls sleep(), which removes the AXObserver from the run loop.
	let mut ax_observers_w = AX_OBSERVERS.write().unwrap();
	ax_observers_w.remove(&app_handle).ok_or(Error::AppNotRunning)?;

	log::info!("Unregistered accessibility observer for app handle: {:?}", app_handle);

	Ok(())
}

/// Observe an element for accessibility events.
///
/// The returned handle owns this specific element + notification registration.
/// Use it later to stop observing this notification.
pub fn observe_element_notification(
	element_handle: ElementHandle,
	notification: Notification,
	on_event: impl Fn(&observer::accessibility::Event) + 'static + Send + Sync,
) -> Result<NotificationRegistrationHandle, Error> {
	let app_handle = {
		let el = element_handle.inner().ok_or(Error::ElementNotFound)?;
		monitor::app_handle_by_element(&el).ok_or(Error::AppNotFound)?
	};

	let mut ax_observers_w = AX_OBSERVERS.write().unwrap();

	let notif_registration = ax_observers_w
		.get_mut(&app_handle)
		.ok_or(Error::AppNotRunning)?
		.add_notification(element_handle, notification, Box::new(on_event))
		.map_err(Error::Accessibility)?;

	Ok(notif_registration)
}

pub fn unobserve_element_notification(notif_handle: NotificationRegistrationHandle) -> Result<(), Error> {
	let app_handle = {
		let el = observer::accessibility::get_element_handle_for_registration(notif_handle)
			.and_then(|el| el.inner())
			.ok_or(Error::ElementNotFound)?;
		monitor::app_handle_by_element(&el).ok_or(Error::AppNotFound)?
	};

	let mut ax_observers_w = AX_OBSERVERS.write().unwrap();

	ax_observers_w
		.get_mut(&app_handle)
		.ok_or(Error::AppNotRunning)?
		.remove_notification(notif_handle)
		.map_err(Error::Accessibility)?;

	Ok(())
}
