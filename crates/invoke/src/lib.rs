#![allow(clippy::collapsible_if)]
#![feature(try_blocks)]
#![allow(unused_must_use)]

// The pack engine: the consumer-agnostic core. App/process/AX-element tables,
// accessibility queries, keyboard/mouse instruction execution, capability
// declarations, and the plain (callback-as-`impl Fn`) AX observer API.

use common::{
	accessibility::{ElementHandle, Notification, filter::FilterPath},
	main_thread::MainThread,
};
use objc2_app_kit::NSRunningApplication;
use objc2_application_services::AXIsProcessTrusted;
use objc2_foundation::NSString;
use observer::accessibility::NotificationRegistrationHandle;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

use crate::{instruction::Instruction, monitor::AppHandle};

mod ax;
pub mod capability;
mod instruction;
pub mod monitor;
pub mod pack;
pub mod resource;
pub mod when;

pub use capability::Capability;

#[derive(Debug)]
pub enum Error {
	Deserialize(serde_json::Error),
	Serialize(serde_json::Error),
	DeserializeInstruction(serde_json::Error),
	Walk(instruction::Error),
	RunInstruction(instruction::Error),
	AxRegisterObserver(ax::Error),
	AxUnregisterObserver(ax::Error),
	ObserveElementNotification(ax::Error),
	UnobserveElementNotification(ax::Error),
}

pub type Result<T = ()> = std::result::Result<T, Error>;

/// Non-prompting AX trust preflight. Headless AX automation needs to know
/// whether it can run.
pub fn is_process_trusted() -> bool {
	MainThread::run_blocking(move || unsafe { AXIsProcessTrusted() })
}

// ---------------------------------------------------------------------------------------------------------------------

pub fn register_app(bundle_id: String) -> u16 {
	let running_apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(&bundle_id));

	let handle = monitor::register(bundle_id);

	// Cache the live process + AX state if the app is already running.
	if let Some(running_app) = running_apps.firstObject() {
		monitor::update_process(handle, Some(running_app));
	}

	handle.0
}

pub fn unregister_app(app_handle: u16) {
	monitor::unregister(AppHandle(app_handle));
}

// ---------------------------------------------------------------------------------------------------------------------
// Serialization

fn deserialize<T: DeserializeOwned>(value: impl AsRef<str>) -> Result<T> {
	let value = value.as_ref();
	serde_json::from_str(value)
		.inspect_err(|e| log::error!("Failed to deserialize: {e:?}\n{value}"))
		.map_err(Error::Deserialize)
}

fn deserialize_instruction<T: DeserializeOwned>(value: impl AsRef<str>) -> Result<T> {
	let value = value.as_ref();
	serde_json::from_str(value)
		.inspect_err(|e| log::error!("Failed to deserialize instruction: {e:?}\n{value}"))
		.map_err(Error::DeserializeInstruction)
}

fn serialize<T: Serialize + Debug>(value: &T) -> std::result::Result<String, serde_json::Error> {
	serde_json::to_string(value).inspect_err(|e| log::error!("Failed to serialize: {e:?}\n{value:?}"))
}

// ---------------------------------------------------------------------------------------------------------------------
// Instruction API

pub fn dispose_element(element_handle: u32) {
	MainThread::dispatch(move || {
		common::accessibility::element_handle::dispose_element(element_handle.into());
	});
}

pub async fn get_app_element(app_handle: u16) -> Option<u32> {
	MainThread::run(move || AppHandle(app_handle).element_handle().map(|eh| eh.into())).await
}

pub async fn walk(root: u32, path: String) -> Result<u32> {
	let path: FilterPath = deserialize(&path)?;

	let mut walk_instruction = instruction::Walk { root: root.into(), path };

	MainThread::run(move || walk_instruction.run().map(|el| el.retain().into()))
		.await
		.map_err(Error::Walk)
}

pub async fn keyboard(keyboard_instruction: String) -> Result<()> {
	let mut keyboard_instruction: instruction::Keyboard = deserialize_instruction(keyboard_instruction)?;
	MainThread::run(move || keyboard_instruction.run()).await.map_err(Error::RunInstruction)
}

pub async fn mouse(mouse_instruction: String) -> Result<()> {
	let mut mouse_instruction: instruction::Mouse = deserialize_instruction(mouse_instruction)?;
	MainThread::run(move || mouse_instruction.run()).await.map_err(Error::RunInstruction)
}

pub async fn get_attribute(get_attribute_instruction: String) -> Result<String> {
	let mut get_attribute_instruction: instruction::GetAttribute = deserialize_instruction(get_attribute_instruction)?;

	let result = MainThread::run(move || get_attribute_instruction.run().map(|v| convert::accessibility_value_to_json(&v)))
		.await
		.map_err(Error::RunInstruction)?;

	serialize(&result).map_err(Error::Serialize)
}

pub async fn run_action(run_action_instruction: String) -> Result<()> {
	let mut run_action_instruction: instruction::RunAction = deserialize_instruction(run_action_instruction)?;
	MainThread::run(move || run_action_instruction.run()).await.map_err(Error::RunInstruction)
}

pub async fn set_attribute(set_attribute_instruction: String) -> Result<()> {
	let mut set_attribute_instruction: instruction::SetAttribute = deserialize_instruction(set_attribute_instruction)?;
	MainThread::run(move || set_attribute_instruction.run()).await.map_err(Error::RunInstruction)
}

// ---------------------------------------------------------------------------------------------------------------------
// Accessibility observer API
//
// The callback is a plain `impl Fn(String)`.

pub async fn register_accessibility_observer(app_handle: u16) -> Result<()> {
	let app_handle = AppHandle(app_handle);
	MainThread::run(move || try {
		ax::register_observer(app_handle).map_err(Error::AxRegisterObserver)?;
		ax::update_process(app_handle).inspect_err(|e| log::warn!("Could not ax::update_process: {e:?}"));
	})
	.await;
	Ok(())
}

pub async fn unregister_accessibility_observer(app_handle: u16) -> Result<()> {
	let app_handle = AppHandle(app_handle);
	MainThread::run(move || ax::unregister_observer(app_handle).map_err(Error::AxUnregisterObserver)).await
}

pub async fn observe_element_notification(
	element_handle: u32,
	notification: String,
	on_event: impl Fn(String) + Send + Sync + 'static,
) -> Result<NotificationRegistrationHandle> {
	let notification: Notification = notification.parse().unwrap();
	MainThread::run(move || {
		ax::observe_element_notification(ElementHandle::from(element_handle), notification, move |event| {
			on_event(convert::accessibility_event_to_json(event).to_string());
		})
	})
	.await
	.map_err(Error::ObserveElementNotification)
}

pub async fn unobserve_element_notification(notification_registration_handle: NotificationRegistrationHandle) -> Result<()> {
	MainThread::run(move || ax::unobserve_element_notification(notification_registration_handle).map_err(Error::UnobserveElementNotification)).await
}

// ---------------------------------------------------------------------------------------------------------------------
// Convert module

mod convert {
	use std::collections::HashMap;

	use common::accessibility::{element_handle::retain_element, value::Value};
	use serde_json::json;

	pub fn accessibility_value_to_json(value: &Value) -> serde_json::Value {
		value.to_json(&|el| {
			let handle = el.retain();
			json!({ "#e": handle }) // [special-handling-json-revive]
		})
	}
	/// Converts an Accessibility Event to JsonValue
	pub fn accessibility_event_to_json(event: &observer::accessibility::Event) -> serde_json::Value {
		match event {
			observer::accessibility::Event::Notification { name, element, info } => {
				let target_handle = retain_element(unsafe { element.as_ref() });

				json!({
					"name": serde_json::to_value(name).unwrap(),
					"element": { "#e": target_handle },
					"info": info.as_ref().map(|info|
						info.iter()
							.map(|(k, v)| (k.clone(), accessibility_value_to_json(v)))
							.collect::<HashMap<String, serde_json::Value>>()
					)
				})
			}
		}
	}
}

// ---------------------------------------------------------------------------------------------------------------------
// Keyboard modifier state.
//
// Shared between keyboard *output* (`instruction::Keyboard`) and the EventTap
// input-*capture* half, which reads it via `state::keyboard`.

pub mod state {
	// Keyboard modifier-override storage only. Keyboard *output*
	// (`instruction::Keyboard`) sets it; the EventTap *capture* half reads
	// `override_modifiers()` from here.
	pub mod keyboard {
		use hid::keyboard::Modifiers;
		use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};

		static OVERRIDE_MODIFIERS: AtomicU16 = AtomicU16::new(Modifiers::empty().bits());
		static SHOULD_OVERRIDE_MODIFIERS: AtomicBool = AtomicBool::new(false);

		pub fn override_modifiers() -> Modifiers {
			if !SHOULD_OVERRIDE_MODIFIERS.load(Ordering::Relaxed) {
				return Modifiers::empty();
			}
			Modifiers::from_bits(OVERRIDE_MODIFIERS.load(Ordering::Relaxed)).unwrap()
		}
		pub fn set_override_modifiers(modifiers: Modifiers) {
			OVERRIDE_MODIFIERS.store(modifiers.bits(), Ordering::Relaxed);
		}
		pub fn clear_override_modifiers() {
			OVERRIDE_MODIFIERS.store(Modifiers::empty().bits(), Ordering::Relaxed);
		}

		pub fn toggle_override_modifiers(should_override: bool) {
			SHOULD_OVERRIDE_MODIFIERS.store(should_override, Ordering::Relaxed);
		}
	}
}
