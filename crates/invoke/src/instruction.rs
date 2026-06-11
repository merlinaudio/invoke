use crate::{
	monitor::AppHandle,
	state::keyboard::{clear_override_modifiers, set_override_modifiers, toggle_override_modifiers},
};
use common::accessibility::{self, Action, Attribute, Element, element_handle::ElementHandle, filter::FilterPath};
use hid::{
	EventLocation,
	keyboard::{Accelerator, Key, Modifiers},
	mouse::{MouseButton, ScrollWheel},
};
use objc2_core_foundation::{CFBoolean, CFNumber, CFString};

#[derive(Debug)]
pub enum Error {
	ApplicationNotSet,
	Element(common::accessibility::element::Error),
	Walk(common::accessibility::element::WalkError),
	Hid(hid::Error),
}

pub trait Instruction {
	type Value;
	fn run(&mut self) -> Self::Value;
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Walk {
	pub path: FilterPath,
	pub root: ElementHandle,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RunAction {
	pub element: ElementHandle,
	pub action: Action,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GetAttribute {
	pub element: ElementHandle,
	pub attribute: Attribute,
	pub allow_cached: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SetAttribute {
	pub element: ElementHandle,
	pub attribute: Attribute,
	pub value: SetAttributeValue,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(untagged)]
pub enum SetAttributeValue {
	Bool(bool),
	Number(f64),
	String(String),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Keyboard {
	KeyDown { key: Option<Key>, modifiers: Modifiers, app: AppHandle },
	KeyUp { key: Option<Key>, modifiers: Modifiers, app: AppHandle },
	KeyPress { key: Option<Key>, modifiers: Modifiers, app: AppHandle },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Mouse {
	/// Normal press-and-hold of a button
	ButtonSingleClickDown {
		button: MouseButton,
		app: AppHandle,
	},
	/// Double click and hold
	ButtonDoubleClickDown {
		button: MouseButton,
		app: AppHandle,
	},
	/// Triple click and hold
	ButtonTripleClickDown {
		button: MouseButton,
		app: AppHandle,
	},
	/// Normal release of a button
	ButtonUp {
		button: MouseButton,
		app: AppHandle,
	},

	ScrollY {
		delta: i32,
		app: AppHandle,
	},
	ScrollX {
		delta: i32,
		app: AppHandle,
	},
	ScrollZ {
		delta: i32,
		app: AppHandle,
	},
	Move {
		x: f64,
		y: f64,
		app: AppHandle,
	},
}

// ---------------------------------------------------------------------------------------------------------------------

impl Instruction for Walk {
	type Value = Result<Element, Error>;

	fn run(&mut self) -> Self::Value {
		let element: Element = self.root.try_into().map_err(Error::Element)?;
		log::debug!(
			"Walking from root REF: {root:?} ELEMENT: {element:?} PATH: {path:?}",
			root = self.root,
			path = self.path
		);
		element.walk(&self.path).map_err(Error::Walk)
	}
}

impl Instruction for RunAction {
	type Value = Result<(), Error>;

	fn run(&mut self) -> Self::Value {
		let element: Element = self.element.try_into().map_err(Error::Element)?;
		element.perform_action(&self.action).map_err(Error::Element)
	}
}

impl Instruction for GetAttribute {
	type Value = Result<accessibility::Value, Error>;

	fn run(&mut self) -> Self::Value {
		let mut element: Element = self.element.try_into().map_err(Error::Element)?;

		element.attribute(self.attribute.clone()).map(|value| value.to_owned()).map_err(Error::Element)
	}
}

impl Instruction for SetAttribute {
	type Value = Result<(), Error>;

	fn run(&mut self) -> Self::Value {
		let element: Element = self.element.try_into().map_err(Error::Element)?;

		match &self.value {
			SetAttributeValue::Number(n) => element.set_attribute(&self.attribute, &CFNumber::new_f64(*n)),
			SetAttributeValue::Bool(b) => element.set_attribute(&self.attribute, CFBoolean::new(*b)),
			SetAttributeValue::String(s) => element.set_attribute(&self.attribute, &CFString::from_str(s)),
		}
		.map_err(Error::Element)
	}
}

impl Instruction for Keyboard {
	type Value = Result<(), Error>;

	fn run(&mut self) -> Self::Value {
		use Keyboard::*;

		match self {
			KeyDown { key, modifiers, app } => {
				let acc = Accelerator::new(*key, *modifiers);
				let pid = app.pid().ok_or(Error::ApplicationNotSet)?;
				set_override_modifiers(acc.modifiers);
				toggle_override_modifiers(true);
				acc.down(acc.location(pid)).map_err(Error::Hid)?;
			}
			KeyUp { key, modifiers, app } => {
				let acc = Accelerator::new(*key, *modifiers);
				let pid = app.pid().ok_or(Error::ApplicationNotSet)?;
				clear_override_modifiers();
				toggle_override_modifiers(false);
				acc.up(acc.location(pid)).map_err(Error::Hid)?;
			}
			KeyPress { key, modifiers, app } => {
				let acc = Accelerator::new(*key, *modifiers);
				let pid = app.pid().ok_or(Error::ApplicationNotSet)?;
				let location = acc.location(pid);

				let down_result = acc.down(location); // Do NOT return early; try to simulate key.up even if this fails
				let up_result = acc.up(location);

				if let (Err(down_err), Err(up_err)) = (&down_result, &up_result) {
					log::warn!("Keydown and keyup failed, but can only return one error: down_err={down_err:?} up_err={up_err:?}");
				}

				down_result.or(up_result).map_err(Error::Hid)?
			}
		};

		Ok(())
	}
}

impl Instruction for Mouse {
	type Value = Result<(), Error>;

	fn run(&mut self) -> Self::Value {
		use Mouse::*;

		match self {
			ButtonSingleClickDown { button, .. } => button.down(None, EventLocation::Hid).map_err(Error::Hid)?,
			ButtonDoubleClickDown { button, .. } => button.double_click_down(None, EventLocation::Hid).map_err(Error::Hid)?,
			ButtonTripleClickDown { button, .. } => button.triple_click_down(None, EventLocation::Hid).map_err(Error::Hid)?,
			ButtonUp { button, .. } => button.up(None, EventLocation::Hid).map_err(Error::Hid)?,

			ScrollY { delta, .. } => ScrollWheel::X.scroll(*delta, EventLocation::Hid).map_err(Error::Hid)?,
			ScrollX { delta, .. } => ScrollWheel::Y.scroll(*delta, EventLocation::Hid).map_err(Error::Hid)?,
			ScrollZ { delta, .. } => ScrollWheel::Z.scroll(*delta, EventLocation::Hid).map_err(Error::Hid)?,
			Move { .. } => todo!(),
		};

		Ok(())
	}
}
