use std::{
	borrow::{Borrow, BorrowMut},
	fmt::{self, Display},
	ops::{Deref, DerefMut},
	str::FromStr,
};

use crate::alphanum::Alphanum;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(try_from = "&str", into = "String")]
pub struct Identifier {
	parts: Vec<Alphanum>,
}

impl Display for Identifier {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let Self { parts } = self;
		write!(f, "{}", parts.join("."))
	}
}

impl FromStr for Identifier {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let parts: Vec<Alphanum> = s.split('.').map(Alphanum::from_str).collect::<Result<_, _>>()?;

		for part in parts.iter() {
			if part.is_empty() {
				return Err("Parts must not be empty");
			}
		}

		Ok(Self { parts })
	}
}

impl TryFrom<&str> for Identifier {
	type Error = &'static str;
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Self::from_str(value)
	}
}

impl From<Identifier> for String {
	fn from(value: Identifier) -> Self {
		value.to_string()
	}
}

impl Borrow<Vec<Alphanum>> for Identifier {
	fn borrow(&self) -> &Vec<Alphanum> {
		&self.parts
	}
}

impl BorrowMut<Vec<Alphanum>> for Identifier {
	fn borrow_mut(&mut self) -> &mut Vec<Alphanum> {
		&mut self.parts
	}
}

impl Deref for Identifier {
	type Target = Vec<Alphanum>;

	fn deref(&self) -> &Self::Target {
		&self.parts
	}
}

impl DerefMut for Identifier {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.parts
	}
}
