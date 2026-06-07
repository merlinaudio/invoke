use std::io::{self, IsTerminal, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::*;

/// Read a JSON line from stdin if it's a pipe. Returns `None` if stdin is a terminal or empty.
pub fn read_json_line<T: DeserializeOwned>() -> Result<Option<T>> {
	let stdin = io::stdin();
	if stdin.is_terminal() {
		return Ok(None);
	}

	let mut line = String::new();
	stdin.read_line(&mut line).err_code("StdinRead")?;

	let line = line.trim();
	if line.is_empty() {
		return Ok(None);
	}

	serde_json::from_str(line).map(Some).err_code("StdinParse")
}

/// Write compact JSON line to stdout.
pub fn write_json_line(val: &impl Serialize) -> Result {
	let mut w = io::stdout().lock();
	serde_json::to_writer(&mut w, val).err_code("StdoutWrite")?;
	w.write_all(b"\n").err_code("StdoutWrite")
}
