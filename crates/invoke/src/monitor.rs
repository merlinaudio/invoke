use common::{
	accessibility::{self, ElementHandle, element_handle},
	main_thread::MainThread,
};
use objc2::rc::Retained;
use objc2_app_kit::NSRunningApplication;
use objc2_application_services::{AXError, AXUIElement};
use std::{
	ops::{Deref, DerefMut},
	ptr::NonNull,
	sync::LazyLock,
	time::Instant,
};

use crate::ax;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AppHandle(pub u16);

impl AppHandle {
	pub const MAX: u16 = u16::MAX;

	pub fn element_handle(self) -> Option<ElementHandle> {
		app_handles().pin_owned().get(&self).and_then(|app| app.element)
	}

	pub fn pid(self) -> Option<u32> {
		app_handles()
			.pin_owned()
			.get(&self)
			.and_then(|app| app.running_app.as_ref().map(|running_app| running_app.unwrap().processIdentifier() as u32))
	}
}
impl Deref for AppHandle {
	type Target = u16;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for AppHandle {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Debug)]
pub struct App {
	pub bundle_id: String,
	pub process_id: Option<u32>,
	pub running_app: Option<MainThread<Retained<NSRunningApplication>>>,
	// A raw handle, not a `resource::Element` guard — it lives as long as the
	// app handle, which survives relaunches.
	pub element: Option<ElementHandle>,
}

// ---------------------------------------------------------------------------------------------------------------------

static APP_HANDLES: LazyLock<papaya::HashMap<AppHandle, App>> = LazyLock::new(Default::default);

// ---------------------------------------------------------------------------------------------------------------------

fn app_handles() -> &'static papaya::HashMap<AppHandle, App> {
	&APP_HANDLES
}

pub fn next_app_handle() -> Option<AppHandle> {
	let len = app_handles().len() + 1;
	if len >= usize::from(AppHandle::MAX) {
		return None;
	}
	Some(AppHandle(len as u16))
}

pub fn app_handle_by_bundle_id(bundle_id: impl AsRef<str>) -> Option<AppHandle> {
	let bundle_id = bundle_id.as_ref();
	let app_handles = app_handles().pin();
	let app_handle = app_handles.iter().find(|(_, app)| app.bundle_id == bundle_id);
	app_handle.map(|(app_handle, _)| *app_handle)
}

pub fn app_handle_by_element(el: &AXUIElement) -> Option<AppHandle> {
	let mut pid_out = 0;
	let ax_error = unsafe { el.pid(NonNull::new_unchecked(&mut pid_out)) };
	if ax_error != AXError::Success {
		return None;
	}
	app_handle_by_process_id(pid_out as u32)
}

pub fn app_handle_by_process_id(process_id: u32) -> Option<AppHandle> {
	let app_handles = app_handles().pin();
	let app_handle = app_handles.iter().find(|(_, app)| app.process_id.is_some_and(|pid| pid == process_id));
	app_handle.map(|(app_handle, _)| *app_handle)
}

/// Refresh the live macOS process behind an app handle.
///
/// App handles survive relaunches. When the app is running, this stores the
/// current running app, process id, app AX element, and retargets its AX observer.
/// When the app terminates, this clears the process state.
pub fn update_process(app_handle: AppHandle, running_app: Option<Retained<NSRunningApplication>>) {
	let is_app_running = running_app.is_some();

	MainThread::dispatch(move || update_app_handle(app_handle, running_app));

	if is_app_running {
		MainThread::dispatch(move || {
			match ax::update_process(app_handle) {
				Ok(_) => log::info!("Updated AX for {app_handle:?}"),

				Err(ax::Error::AppNotRunning) => log::debug!("App {app_handle:?} not running, skipping AX update"),
				Err(e) => log::error!("Error updating AX for {app_handle:?}: {e:?}"),
			};
		});
	}
}

fn update_app_handle(app_handle: AppHandle, running_app: Option<Retained<NSRunningApplication>>) {
	let pid = running_app.as_ref().map(|ra| ra.processIdentifier() as u32);

	let start = Instant::now();

	let app_handles = app_handles().pin();

	let app = app_handles.update(app_handle, |app| App {
		bundle_id: app.bundle_id.clone(),
		running_app: running_app.clone().map(MainThread::new),
		process_id: pid,
		element: match pid {
			Some(pid) => accessibility::Element::new_application(pid)
				.inspect_err(|e| log::warn!("Couldn't get AXUIElement for application {running_app:?}: {e:?}"))
				.ok()
				.map(|new_app_element| {
					if let Some(existing_app_element) = &app.element {
						// Keep JS-held element handles valid across app relaunches.
						log::info!("updating ElementHandle {existing_app_element:?} for app {app_handle:?} to element {new_app_element:?}");
						element_handle::update(*existing_app_element, new_app_element)
					} else {
						// First process we have seen for this app; create its JS-facing element handle.
						let new_app_element_handle = new_app_element.retain();
						log::info!("assigned new ElementHandle {new_app_element_handle:?} for app {app_handle:?} to element {new_app_element:?}");
						new_app_element_handle
					}
				}),
			None => app.element,
		},
	});

	log::info!(
		"updated AppHandle to {:?} took {:?}",
		app.and_then(|a| a.running_app.as_ref().map(|ra| ra.processIdentifier())),
		start.elapsed()
	);
}

pub fn register(bundle_id: String) -> AppHandle {
	// None means the u16 handle space is exhausted.
	let app_handle = next_app_handle().expect("Couldn't get next app handle");

	app_handles().pin().insert(
		app_handle,
		App {
			bundle_id,
			process_id: None,
			running_app: None,
			element: None,
		},
	);

	app_handle
}

pub fn unregister(app_handle: AppHandle) {
	app_handles().pin().remove(&app_handle);
}
