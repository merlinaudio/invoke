use objc2_core_foundation::{CFRetained, CGPoint};
use objc2_core_graphics::{CGEvent, CGEventField, CGEventType, CGMouseButton, CGScrollEventUnit};

use crate::{Error, EventLocation};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "lowercase", tag = "button"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "napi", napi_derive::napi)]
pub enum MouseButton {
	Left,
	Right,
	Other { code: u8 },
}

#[derive(Debug, thiserror::Error)]
pub enum MouseEventError {
	#[error("Failed to create mouse event")]
	NoEventCreated,

	#[error("Invalid PID")]
	InvalidPid(u32),
}

impl MouseButton {
	pub fn down(&self, at: Option<(f64, f64)>, location: EventLocation) -> Result<(), Error> {
		let event = new_mouse_button_down_event(*self, at, ClickCount::Single)?;
		location.post(&event)?;
		Ok(())
	}

	pub fn double_click_down(&self, at: Option<(f64, f64)>, location: EventLocation) -> Result<(), Error> {
		let event = new_mouse_button_down_event(*self, at, ClickCount::Double)?;
		location.post(&event)?;
		Ok(())
	}

	pub fn triple_click_down(&self, at: Option<(f64, f64)>, location: EventLocation) -> Result<(), Error> {
		let event = new_mouse_button_down_event(*self, at, ClickCount::Triple)?;
		location.post(&event)?;
		Ok(())
	}

	pub fn up(&self, at: Option<(f64, f64)>, location: EventLocation) -> Result<(), Error> {
		let event = new_mouse_button_up_event(*self, at)?;
		location.post(&event)?;
		Ok(())
	}
}

impl From<MouseButton> for CGMouseButton {
	fn from(value: MouseButton) -> Self {
		match value {
			MouseButton::Left => CGMouseButton::Left,
			MouseButton::Right => CGMouseButton::Right,
			MouseButton::Other { .. } => CGMouseButton::Center, // Button must be set on the event using `set_integer_value_field(MouseEventButtonNumber)`, or in `CGEvent::new_mouse_event` (last param)
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum ScrollWheel {
	X,
	Y,
	Z,
}

impl ScrollWheel {
	pub fn scroll(&self, delta: i32, location: EventLocation) -> Result<(), Error> {
		let mut x = 0;
		let mut y = 0;
		let mut z = 0;

		match self {
			ScrollWheel::X => x = delta,
			ScrollWheel::Y => y = delta,
			ScrollWheel::Z => z = delta,
		}

		let event = new_mouse_scroll_event(x, y, z, location)?;
		location.post(&event)?;
		Ok(())
	}
}

fn current_mouse_position() -> Option<CGPoint> {
	let event = CGEvent::new(None)?;
	Some(CGEvent::location(Some(&event)))
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ClickCount {
	Single = 1,
	Double = 2,
	Triple = 3,
}

fn new_mouse_button_down_event(button: MouseButton, point: Option<(f64, f64)>, click_count: ClickCount) -> Result<CFRetained<CGEvent>, Error> {
	let point = point
		.map(|(x, y)| CGPoint::new(x, y))
		.or_else(current_mouse_position)
		.ok_or(Error::EventCreationFailed)?;

	let event_type = match button {
		MouseButton::Left => CGEventType::LeftMouseDown,
		MouseButton::Right => CGEventType::RightMouseDown,
		MouseButton::Other { .. } => CGEventType::OtherMouseDown, // Button must be set using `set_integer_value_field(MouseEventButtonNumber)`
	};

	let event = CGEvent::new_mouse_event(None, event_type, point, button.into()).ok_or(Error::EventCreationFailed)?;

	// Set state (single/double/triple click), and actual button number if not left/right/center (e.g. thumb button)
	if let MouseButton::Other { code: button_number } = button {
		CGEvent::set_integer_value_field(Some(&event), CGEventField::MouseEventButtonNumber, button_number.into());
	}

	CGEvent::set_integer_value_field(Some(&event), CGEventField::MouseEventClickState, click_count as i64);

	Ok(event)
}

fn new_mouse_button_up_event(button: MouseButton, point: Option<(f64, f64)>) -> Result<CFRetained<CGEvent>, Error> {
	let point = point
		.map(|(x, y)| CGPoint::new(x, y))
		.or_else(current_mouse_position)
		.ok_or(Error::EventCreationFailed)?;

	let event_type = match button {
		MouseButton::Left => CGEventType::LeftMouseUp,
		MouseButton::Right => CGEventType::RightMouseUp,
		MouseButton::Other { .. } => CGEventType::OtherMouseUp, // Button must be set using `set_integer_value_field(MouseEventButtonNumber)`
	};

	let event = CGEvent::new_mouse_event(None, event_type, point, button.into()).ok_or(Error::EventCreationFailed)?;

	// Set state (not clicked), and actual button number if not left/right/center (e.g. thumb button)
	if let MouseButton::Other { code: button_number } = button {
		CGEvent::set_integer_value_field(Some(&event), CGEventField::MouseEventButtonNumber, button_number.into());
	};

	Ok(event)
}

fn new_mouse_scroll_event(x: i32, y: i32, z: i32, _location: EventLocation) -> Result<CFRetained<CGEvent>, Error> {
	let wheel_count = if z != 0 {
		3
	} else if x != 0 {
		2
	} else {
		1
	};

	// CGEventCreateScrollWheelEvent2 params: wheel1=vertical(Y), wheel2=horizontal(X), wheel3=Z
	let event = CGEvent::new_scroll_wheel_event2(None, CGScrollEventUnit::Line, wheel_count, y, x, z).ok_or(Error::EventCreationFailed)?;

	Ok(event)
}
