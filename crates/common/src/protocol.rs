use std::io::{self, Write};

pub const HEADER_START: u8 = 0x01;
pub const TEXT_START: u8 = 0x02;
pub const LINE_END: u8 = b'\n';

/// Validate that `s` contains no wire delimiters, return its bytes.
fn check(s: &str) -> io::Result<&[u8]> {
	let b = s.as_bytes();
	if b.iter().any(|&c| c == HEADER_START || c == TEXT_START || c == LINE_END) {
		return Err(io::Error::new(io::ErrorKind::InvalidData, "delimiter in field"));
	}
	Ok(b)
}

/// A parsed or to-be-written wire message: `{id}\x01{header}\x02{text}`.
pub struct Message<'a> {
	pub id: &'a str,
	pub header: &'a str,
	pub text: &'a str,
}

impl<'a> Message<'a> {
	/// Parse a line (without trailing newline) into its three parts.
	pub fn parse(line: &'a str) -> Option<Self> {
		let b = line.as_bytes();
		let h = b.iter().position(|&c| c == HEADER_START)?;
		let t = b[h + 1..].iter().position(|&c| c == TEXT_START)? + h + 1;
		Some(Self {
			id: &line[..h],
			header: &line[h + 1..t],
			text: &line[t + 1..],
		})
	}

	/// Write the message followed by a newline.
	///
	/// Returns `InvalidData` if any field contains a delimiter.
	pub fn write(&self, w: &mut impl Write) -> io::Result<()> {
		w.write_all(check(self.id)?)?;
		w.write_all(&[HEADER_START])?;
		w.write_all(check(self.header)?)?;
		w.write_all(&[TEXT_START])?;
		w.write_all(check(self.text)?)?;
		w.write_all(&[LINE_END])
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_valid() {
		let msg = Message::parse("42\x01pack.run\x02{\"a\":1}").unwrap();
		assert_eq!(msg.id, "42");
		assert_eq!(msg.header, "pack.run");
		assert_eq!(msg.text, "{\"a\":1}");
	}

	#[test]
	fn parse_empty_text() {
		let msg = Message::parse("0\x010\x02").unwrap();
		assert_eq!(msg.id, "0");
		assert_eq!(msg.header, "0");
		assert_eq!(msg.text, "");
	}

	#[test]
	fn parse_empty_id() {
		let msg = Message::parse("\x01h\x02t").unwrap();
		assert_eq!(msg.id, "");
		assert_eq!(msg.header, "h");
		assert_eq!(msg.text, "t");
	}

	#[test]
	fn parse_missing_header_start() {
		assert!(Message::parse("no delimiters").is_none());
	}

	#[test]
	fn parse_missing_text_start() {
		assert!(Message::parse("0\x01header_only").is_none());
	}

	#[test]
	fn write_produces_correct_bytes() {
		let mut buf = Vec::new();
		Message {
			id: "5",
			header: "pack.run",
			text: "{}",
		}
		.write(&mut buf)
		.unwrap();
		assert_eq!(buf, b"5\x01pack.run\x02{}\n");
	}

	#[test]
	fn roundtrip() {
		let mut buf = Vec::new();
		Message {
			id: "99",
			header: "ns.route",
			text: "hello",
		}
		.write(&mut buf)
		.unwrap();

		let line = std::str::from_utf8(&buf[..buf.len() - 1]).unwrap();
		let msg = Message::parse(line).unwrap();
		assert_eq!(msg.id, "99");
		assert_eq!(msg.header, "ns.route");
		assert_eq!(msg.text, "hello");
	}

	#[test]
	fn write_rejects_delimiter_in_id() {
		let mut buf = Vec::new();
		assert!(
			Message {
				id: "0\x01",
				header: "h",
				text: "t"
			}
			.write(&mut buf)
			.is_err()
		);
	}

	#[test]
	fn write_rejects_delimiter_in_header() {
		let mut buf = Vec::new();
		assert!(
			Message {
				id: "0",
				header: "h\x02x",
				text: "t"
			}
			.write(&mut buf)
			.is_err()
		);
	}

	#[test]
	fn write_rejects_newline_in_text() {
		let mut buf = Vec::new();
		assert!(
			Message {
				id: "0",
				header: "h",
				text: "line1\nline2"
			}
			.write(&mut buf)
			.is_err()
		);
	}
}
