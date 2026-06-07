use objc2_core_foundation::{CFRetained, CFString};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum SortDirection {
	/// A sort direction referenced by name, for sort directions not in the predefined list.
	Literal(String),

	Ascending,
	Descending,
	Unknown,
}

impl std::str::FromStr for SortDirection {
	type Err = std::convert::Infallible;

	// TODO: Add a literal/escape syntax (see Attribute::FromStr) for the same
	// name-collision reason.
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"Ascending" | "ascending" | "AXAscendingSortDirection" => SortDirection::Ascending,
			"Descending" | "descending" | "AXDescendingSortDirection" => SortDirection::Descending,
			"Unknown" | "unknown" | "AXUnknownSortDirection" => SortDirection::Unknown,
			_ => SortDirection::Literal(s.to_string()),
		})
	}
}

impl SortDirection {
	#[allow(non_snake_case)]
	pub fn to_CFString(&self) -> CFRetained<CFString> {
		self.into()
	}
}

impl From<&SortDirection> for CFRetained<CFString> {
	fn from(dir: &SortDirection) -> CFRetained<CFString> {
		if let SortDirection::Literal(name) = dir {
			return CFString::from_str(name);
		}

		CFString::from_static_str(match dir {
			SortDirection::Literal(_) => unreachable!(),
			SortDirection::Ascending => "AXAscendingSortDirection",
			SortDirection::Descending => "AXDescendingSortDirection",
			SortDirection::Unknown => "AXUnknownSortDirection",
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_all_forms() {
		assert_eq!("ascending".parse::<SortDirection>().unwrap(), SortDirection::Ascending);
		assert_eq!("Ascending".parse::<SortDirection>().unwrap(), SortDirection::Ascending);
		assert_eq!("AXAscendingSortDirection".parse::<SortDirection>().unwrap(), SortDirection::Ascending);
	}

	#[test]
	fn unknown_becomes_named() {
		assert_eq!(
			"AXRandomSortDirection".parse::<SortDirection>().unwrap(),
			SortDirection::Literal("AXRandomSortDirection".into())
		);
		assert_eq!("shuffled".parse::<SortDirection>().unwrap(), SortDirection::Literal("shuffled".into()));
	}
}
