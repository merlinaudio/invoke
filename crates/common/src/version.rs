use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(try_from = "&str", into = "String")]
pub struct Version {
	major: u16,
	minor: u16,
	patch: u16,
	extra: Option<String>,
}

impl Version {
	pub fn new(major: u16, minor: u16, patch: u16, extra: Option<String>) -> Self {
		Self { major, minor, patch, extra }
	}
}

impl Display for Version {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if let Some(extra) = &self.extra {
			write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, extra)
		} else {
			write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
		}
	}
}

impl From<Version> for String {
	fn from(value: Version) -> Self {
		value.to_string()
	}
}

impl TryFrom<&str> for Version {
	type Error = &'static str;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Self::from_str(value)
	}
}

impl FromStr for Version {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// Allow stripping exactly one 'v' from the start.
		// So this is valid: "v1.0.0".
		let s = s.strip_prefix('v').unwrap_or(s);

		// Now split at "-" in case it's there. Anything after the "-" is the "extra" part. Anything before is the version parts (major.minor.patch).
		let (s, extra) = s.split_once('-').map_or((s, None), |(s, extra)| (s, Some(extra)));

		// Split the start of the string at every "."
		let parts: Vec<&str> = s.split('.').collect();
		if parts.len() != 3 {
			return Err("must have exactly 3 parts");
		}

		// parts[n] is fine because there are 3 parts (n=0,1,2) for sure, so can't go OOB
		Ok(Self {
			major: parts[0].parse().map_err(|_| "major part must be a number")?,
			minor: parts[1].parse().map_err(|_| "minor part must be a number")?,
			patch: parts[2].parse().map_err(|_| "patch part must be a number")?,
			extra: extra.map(ToString::to_string),
		})
	}
}

mod tests {
	#![allow(unused)]
	use super::*;

	#[test]
	fn test_from_str() {
		assert_eq!(Version::from_str("v1.0.0").unwrap(), Version::new(1, 0, 0, None));
		assert_eq!(Version::from_str("v1.0.0-beta").unwrap(), Version::new(1, 0, 0, Some("beta".to_string())));
		assert_eq!(
			Version::from_str("1.0.0-...0-.0-").unwrap(),
			Version::new(1, 0, 0, Some("...0-.0-".to_string()))
		);

		assert!(Version::from_str("1.0-0.beta").is_err());
	}
}
