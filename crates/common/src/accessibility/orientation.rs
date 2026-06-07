use objc2_core_foundation::{CFRetained, CFString};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Orientation {
	/// An orientation referenced by name, for orientations not in the predefined list.
	Literal(String),

	Horizontal,
	Vertical,
	Unknown,
}

impl std::str::FromStr for Orientation {
	type Err = std::convert::Infallible;

	// TODO: Add a literal/escape syntax (see Attribute::FromStr) for the same
	// name-collision reason.
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"Horizontal" | "horizontal" | "AXHorizontalOrientation" => Orientation::Horizontal,
			"Vertical" | "vertical" | "AXVerticalOrientation" => Orientation::Vertical,
			"Unknown" | "unknown" | "AXUnknownOrientation" => Orientation::Unknown,
			_ => Orientation::Literal(s.to_string()),
		})
	}
}

impl Orientation {
	#[allow(non_snake_case)]
	pub fn to_CFString(&self) -> CFRetained<CFString> {
		self.into()
	}
}

impl From<&Orientation> for CFRetained<CFString> {
	fn from(orientation: &Orientation) -> CFRetained<CFString> {
		if let Orientation::Literal(name) = orientation {
			return CFString::from_str(name);
		}

		CFString::from_static_str(match orientation {
			Orientation::Literal(_) => unreachable!(),
			Orientation::Horizontal => "AXHorizontalOrientation",
			Orientation::Vertical => "AXVerticalOrientation",
			Orientation::Unknown => "AXUnknownOrientation",
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_all_forms() {
		assert_eq!("horizontal".parse::<Orientation>().unwrap(), Orientation::Horizontal);
		assert_eq!("Horizontal".parse::<Orientation>().unwrap(), Orientation::Horizontal);
		assert_eq!("AXHorizontalOrientation".parse::<Orientation>().unwrap(), Orientation::Horizontal);
	}

	#[test]
	fn unknown_becomes_named() {
		assert_eq!(
			"AXDiagonalOrientation".parse::<Orientation>().unwrap(),
			Orientation::Literal("AXDiagonalOrientation".into())
		);
		assert_eq!("sideways".parse::<Orientation>().unwrap(), Orientation::Literal("sideways".into()));
	}
}
