use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
	pub code: &'static str,
	pub detail: Option<String>,
}

impl Error {
	pub fn code(code: &'static str) -> Self {
		Self { code, detail: None }
	}

	pub fn new(code: &'static str, detail: impl Into<String>) -> Self {
		Self {
			code,
			detail: Some(detail.into()),
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.code)?;
		if let Some(detail) = &self.detail {
			write!(f, ": {detail}")?;
		}
		Ok(())
	}
}

pub type Result<T = ()> = std::result::Result<T, Error>;

pub trait ResultExt<T, E> {
	fn err_code(self, code: &'static str) -> Result<T>;
}

impl<T, E: Display> ResultExt<T, E> for std::result::Result<T, E> {
	fn err_code(self, code: &'static str) -> Result<T> {
		self.map_err(|e| Error {
			code,
			detail: Some(e.to_string()),
		})
	}
}

pub trait OptionExt<T> {
	fn err_code(self, code: &'static str) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
	fn err_code(self, code: &'static str) -> Result<T> {
		self.ok_or(Error { code, detail: None })
	}
}
