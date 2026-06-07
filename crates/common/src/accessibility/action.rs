use objc2_core_foundation::{CFRetained, CFString};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(rename_all = "camelCase", from = "String")
)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Action {
	/// An action referenced by name, for actions not in the predefined list.
	Literal(String),

	Press,
	Increment,
	Decrement,
	Confirm,
	Cancel,
	ShowAlternateUI,
	ShowDefaultUI,
	Raise,
	ShowMenu,
	Pick,
}

impl std::str::FromStr for Action {
	type Err = std::convert::Infallible;

	// TODO: Add a literal/escape syntax (see Attribute::FromStr) for the same
	// name-collision reason.
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"Press" | "press" | "AXPress" => Action::Press,
			"Increment" | "increment" | "AXIncrement" => Action::Increment,
			"Decrement" | "decrement" | "AXDecrement" => Action::Decrement,
			"Confirm" | "confirm" | "AXConfirm" => Action::Confirm,
			"Cancel" | "cancel" | "AXCancel" => Action::Cancel,
			"ShowAlternateUI" | "showAlternateUI" | "AXShowAlternateUI" => Action::ShowAlternateUI,
			"ShowDefaultUI" | "showDefaultUI" | "AXShowDefaultUI" => Action::ShowDefaultUI,
			"Raise" | "raise" | "AXRaise" => Action::Raise,
			"ShowMenu" | "showMenu" | "AXShowMenu" => Action::ShowMenu,
			"Pick" | "pick" | "AXPick" => Action::Pick,
			_ => Action::Literal(s.to_string()),
		})
	}
}

impl Action {
	#[allow(non_snake_case)]
	pub fn to_CFString(&self) -> CFRetained<CFString> {
		self.into()
	}
}

impl From<String> for Action {
	fn from(s: String) -> Self {
		s.parse().unwrap()
	}
}

impl From<&Action> for CFRetained<CFString> {
	fn from(action: &Action) -> CFRetained<CFString> {
		if let Action::Literal(name) = action {
			return CFString::from_str(name);
		}

		CFString::from_static_str(match action {
			Action::Literal(_) => unreachable!(),
			Action::Press => "AXPress",
			Action::Increment => "AXIncrement",
			Action::Decrement => "AXDecrement",
			Action::Confirm => "AXConfirm",
			Action::Cancel => "AXCancel",
			Action::ShowAlternateUI => "AXShowAlternateUI",
			Action::ShowDefaultUI => "AXShowDefaultUI",
			Action::Raise => "AXRaise",
			Action::ShowMenu => "AXShowMenu",
			Action::Pick => "AXPick",
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_all_forms() {
		assert_eq!("press".parse::<Action>().unwrap(), Action::Press);
		assert_eq!("Press".parse::<Action>().unwrap(), Action::Press);
		assert_eq!("AXPress".parse::<Action>().unwrap(), Action::Press);

		assert_eq!("showMenu".parse::<Action>().unwrap(), Action::ShowMenu);
		assert_eq!("ShowMenu".parse::<Action>().unwrap(), Action::ShowMenu);
		assert_eq!("AXShowMenu".parse::<Action>().unwrap(), Action::ShowMenu);
	}

	#[test]
	fn unknown_becomes_named() {
		assert_eq!("AXCustomAction".parse::<Action>().unwrap(), Action::Literal("AXCustomAction".into()));
		assert_eq!("myAction".parse::<Action>().unwrap(), Action::Literal("myAction".into()));
	}
}
