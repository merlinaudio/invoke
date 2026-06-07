use std::{
	borrow::{Borrow, BorrowMut},
	fmt::{self, Display, Formatter},
	ops::{Deref, DerefMut},
	path::Path,
	str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(try_from = "&str", into = "String")]
pub struct Alphanum(String);

impl Display for Alphanum {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl FromStr for Alphanum {
	type Err = &'static str;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.chars().any(|c| !c.is_ascii_alphanumeric()) {
			return Err("Must be alphanumeric");
		}
		Ok(Alphanum(s.to_string()))
	}
}

impl TryFrom<&str> for Alphanum {
	type Error = &'static str;
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Self::from_str(value)
	}
}

impl From<Alphanum> for String {
	fn from(value: Alphanum) -> Self {
		value.0
	}
}

impl<'a> From<&'a Alphanum> for &'a str {
	fn from(value: &'a Alphanum) -> Self {
		&value.0
	}
}

impl Borrow<str> for Alphanum {
	fn borrow(&self) -> &str {
		&self.0
	}
}

impl BorrowMut<str> for Alphanum {
	fn borrow_mut(&mut self) -> &mut str {
		&mut self.0
	}
}

impl Deref for Alphanum {
	type Target = str;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Alphanum {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl AsRef<str> for Alphanum {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl AsRef<Path> for Alphanum {
	fn as_ref(&self) -> &Path {
		Path::new(&self.0)
	}
}
