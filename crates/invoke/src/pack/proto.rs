//! The pack wire protocol.
//!
//! One JSON value per line, both directions — a request (an object) or a
//! response to one (an array):
//!
//!   request    {"id":1,"declareVar":{"name":"theme"}}
//!   event      {"runFunction":{"function":3,"payload":null}}
//!   response   [1,0,7]
//!   response   [1,-1,"no such element"]
//!
//! An array is a response — `[id, status, body]`, where `status` is `OK` (0)
//! or `ERROR` (-1) and `body` is the value or the error message. An object is
//! a request — `{methodName: args}`, plus an `id` when a response is expected;
//! no `id` means fire-and-forget (an event).
//!
//! `HostHandlers` is what the pack sends us (press a key, walk an element);
//! `PackHandlers` is what we send the pack (run a function, render a view) —
//! both externally tagged. `Incoming`/`Outgoing` are the per-direction message
//! types; serde does all (de)serialization, untagged — object vs array is
//! what tells a request from a response.
//!
//! Resource handles (`ElementHandle`, `VarHandle`, `AppHandle`, …) come from
//! the modules that own those resources. Only `Function` and `View` are
//! wire-local: nothing else in the codebase names them.

use common::accessibility::{Action, Attribute, ElementHandle, Notification, filter::FilterPath};
use hid::keyboard::{Key, Modifiers};
use observer::accessibility::NotificationRegistrationHandle;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::instruction::SetAttributeValue;
use crate::monitor::AppHandle;
use crate::when::VarHandle;

/// A response's status code — the middle element of a response array. `ERROR`
/// is `-1`, mirroring macOS error conventions; `OK` is `0`.
pub const OK: i8 = 0;
pub const ERROR: i8 = -1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(transparent)]
pub struct Function(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(transparent)]
pub struct View(pub u32);

/// Binding-only mirror of an opaque JSON value. The wire/Rust type stays
/// `serde_json::Value`; the opaque proto fields carry `#[specta(type = JsonValue)]`
/// so only the *generated* type changes — a recursive JSON value whose numbers are
/// `f64` (→ TS `number`). This sidesteps specta's bigint-forbid on `serde_json::Number`'s
/// `i64`/`u64`, and stays exporter-neutral (no TypeScript-only `unknown` marker leaks
/// into the contract). Never (de)serialized itself.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(untagged)]
pub enum JsonValue {
	// `null` is expressed by the `Option` specta emits for nullable positions; a
	// unit variant here would render as the literal string `"Null"`, so omit it.
	Bool(bool),
	Number(f64),
	String(String),
	Array(Vec<JsonValue>),
	Object(std::collections::BTreeMap<String, JsonValue>),
}

pub trait Request<Response: DeserializeOwned> {
	fn request(self) -> PackHandlers;
}

/// pack → host
#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum HostHandlers {
	RegisterApp {
		bundle_identifier: String,
	},
	DefineFunction {
		function_name: String,
		view: View,
	},
	DeclareVar {
		name: String,
	},
	SetVar {
		var: VarHandle,
		value: bool,
	},
	RunFunction {
		publisher_domain: String,
		pack_name: String,
		function_name: String,
		#[specta(type = JsonValue)]
		payload: serde_json::Value,
	},
	/// The pack finished its initial module load and function registration; it is
	/// now safe to run the functions it declared.
	Ready {},
	KeyboardKeyDown {
		app: AppHandle,
		key: Option<Key>,
		modifiers: Modifiers,
	},
	KeyboardKeyUp {
		app: AppHandle,
		key: Option<Key>,
		modifiers: Modifiers,
	},
	KeyboardKeyPress {
		app: AppHandle,
		key: Option<Key>,
		modifiers: Modifiers,
	},
	ScrollWheelY {
		app: AppHandle,
		delta: i32,
	},
	ScrollWheelX {
		app: AppHandle,
		delta: i32,
	},
	GetAppElement {
		app: AppHandle,
	},
	WalkElement {
		root: ElementHandle,
		filter_path: FilterPath,
	},
	DisposeElement {
		element: ElementHandle,
	},
	PerformElementAction {
		element: ElementHandle,
		action: Action,
	},
	GetElementAttribute {
		element: ElementHandle,
		attribute: Attribute,
	},
	SetElementAttribute {
		element: ElementHandle,
		attribute: Attribute,
		value: SetAttributeValue,
	},
	ObserveElementNotification {
		element: ElementHandle,
		notification_name: Notification,
	},
	UnobserveElementNotification {
		notification: NotificationRegistrationHandle,
	},
}

/// host → pack
#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum PackHandlers {
	RunFunction(RunFunction),
	EndFunction(EndFunction),
	AccessibilityNotification(AccessibilityNotification),
	WorkspaceAppActivated(WorkspaceAppActivated),
	WorkspaceAppDeactivated(WorkspaceAppDeactivated),
	WorkspaceAppTerminated(WorkspaceAppTerminated),
	RenderView(RenderView),
	RunViewAction(RunViewAction),
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RunFunction {
	pub function: Function,
	#[specta(type = JsonValue)]
	pub payload: Value,
}

impl Request<Value> for RunFunction {
	fn request(self) -> PackHandlers {
		PackHandlers::RunFunction(self)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EndFunction {
	pub function: Function,
}

impl Request<()> for EndFunction {
	fn request(self) -> PackHandlers {
		PackHandlers::EndFunction(self)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityNotification {
	pub notification: NotificationRegistrationHandle,
	#[specta(type = JsonValue)]
	pub event: Value,
}

impl Request<()> for AccessibilityNotification {
	fn request(self) -> PackHandlers {
		PackHandlers::AccessibilityNotification(self)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceAppActivated {
	pub app: AppHandle,
}

impl Request<()> for WorkspaceAppActivated {
	fn request(self) -> PackHandlers {
		PackHandlers::WorkspaceAppActivated(self)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceAppDeactivated {
	pub app: AppHandle,
}

impl Request<()> for WorkspaceAppDeactivated {
	fn request(self) -> PackHandlers {
		PackHandlers::WorkspaceAppDeactivated(self)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceAppTerminated {
	pub app: AppHandle,
}

impl Request<()> for WorkspaceAppTerminated {
	fn request(self) -> PackHandlers {
		PackHandlers::WorkspaceAppTerminated(self)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RenderView {
	pub view: View,
}

impl Request<Value> for RenderView {
	fn request(self) -> PackHandlers {
		PackHandlers::RenderView(self)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RunViewAction {
	pub action_id: String,
	#[specta(type = JsonValue)]
	pub args: Value,
	pub view: View,
}

impl Request<Value> for RunViewAction {
	fn request(self) -> PackHandlers {
		PackHandlers::RunViewAction(self)
	}
}

/// A message the pack sends us: a request (an object) or a response to one of our
/// `run`s (an array). Untagged — object vs array picks the variant.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Incoming {
	/// `[id, status, body]` — `status` is `OK` or `ERROR`.
	Response(u32, i8, serde_json::Value),
	Request {
		#[serde(default, skip_serializing_if = "Option::is_none")]
		id: Option<u32>,
		#[serde(flatten)]
		request: HostHandlers,
	},
}

/// A message we send the pack: a request (an object — `id` set when we await a
/// response, unset for a fire-and-forget event) or a response to one of theirs
/// (an array). Untagged — object vs array picks the variant.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Outgoing {
	/// `[id, status, body]` — `status` is `OK` or `ERROR`.
	Response(u32, i8, serde_json::Value),
	Request {
		#[serde(default, skip_serializing_if = "Option::is_none")]
		id: Option<u32>,
		#[serde(flatten)]
		request: PackHandlers,
	},
}

#[cfg(test)]
mod tests {
	use super::*;

	// `Ready {}` is fieldless and reached through `#[serde(flatten)]` on an
	// untagged `Incoming` — exercise the exact wire the TS runtime emits
	// (`transport.call("ready", {})`) to guard that edge.
	#[test]
	fn ready_round_trips_over_the_wire() {
		let incoming: Incoming = serde_json::from_str(r#"{"id":7,"ready":{}}"#).unwrap();
		assert!(matches!(
			incoming,
			Incoming::Request {
				id: Some(7),
				request: HostHandlers::Ready {}
			}
		));
	}
}
