use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Duration;

use backon::{BlockingRetryable, ExponentialBuilder};
use serde_json::Value;

use crate::protocol::{Reply, Request};
use crate::{error::*, service};

/// The Unix socket the `invoke listen` daemon serves and the CLI connects to.
pub fn socket_path() -> Result<&'static Path> {
	static PATH: LazyLock<Option<PathBuf>> = LazyLock::new(|| Some(common::process::application_support_dir()?.join("com.getinvoke.invoke").join(".sock")));
	PATH.as_deref().ok_or(Error::code("AppSupportDir"))
}

/// A live connection to the `invoke listen` daemon. Open once, issue as many
/// requests as you like: each [`request`](Self::request) is one NDJSON
/// round-trip. The buffered reader is owned by the connection, so bytes read
/// past one reply's newline carry into the next request and back-to-back
/// requests stay framed. Closes on drop.
pub struct Connection {
	reader: BufReader<UnixStream>,
	writer: UnixStream,
}

/// Backoff for the post-`kickstart` reconnect: sleep, retry, double the wait, up to
/// [`HEAL_RETRIES`] times (~500ms, 1s, 2s) before giving up. launchd needs a beat to
/// relaunch and rebind; if it isn't back by then, it isn't coming.
const HEAL_BACKOFF: Duration = Duration::from_millis(500);
const HEAL_RETRIES: usize = 3;

impl Connection {
	/// Connect to the daemon. `NotRunning` if nothing is listening.
	pub fn open() -> Result<Self> {
		let stream = UnixStream::connect(socket_path()?).map_err(|e| match e.kind() {
			std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused => Error::code("NotRunning"),
			_ => Error {
				code: "Socket",
				detail: Some(e.to_string()),
			},
		})?;
		let writer = stream.try_clone().err_code("Socket")?;
		Ok(Self {
			reader: BufReader::new(stream),
			writer,
		})
	}

	/// Connect, self-healing a wedged daemon. A `NotRunning` result means the
	/// socket is unreachable — no listener at all, or a stale/orphaned socket file
	/// (e.g. an app update replaced the bundle and stranded the old listener while
	/// its process lived on). Kick the launch agent so launchd starts a fresh daemon
	/// on the current binary, then reconnect until it answers. Any other error is
	/// real and passes straight through untouched.
	pub fn open_or_heal() -> Result<Self> {
		// Fast path: a live daemon answers at once. Only `NotRunning` (no listener /
		// stale socket) is worth healing — surface anything else untouched.
		match Self::open() {
			Err(e) if e.code == "NotRunning" => {}
			result => return result,
		}
		service::kickstart()?;

		// Reconnect with exponential backoff: launchd needs a beat to relaunch and
		// rebind. Retry only while it's still `NotRunning` (not up yet) — a real error
		// means stop and surface it. Backon exhausting its retries leaves the last
		// `NotRunning`, which we swap for a clearer "didn't come back" message.
		Self::open
			.retry(
				ExponentialBuilder::default()
					.with_min_delay(HEAL_BACKOFF)
					.with_factor(2.0)
					.with_max_times(HEAL_RETRIES),
			)
			.sleep(std::thread::sleep)
			.when(|e| e.code == "NotRunning")
			.call()
			.map_err(|e| match e.code {
				"NotRunning" => Error::new("NotRunning", "daemon did not come back after restart — see `invoke service status`"),
				_ => e,
			})
	}

	/// Send one request and read its reply, leaving the connection open.
	pub fn request(&mut self, request: &Request) -> Result<Value> {
		serde_json::to_writer(&mut self.writer, request).err_code("SocketWrite")?;
		self.writer.write_all(b"\n").err_code("SocketWrite")?;
		self.writer.flush().err_code("SocketWrite")?;

		let mut line = String::new();
		self.reader.read_line(&mut line).err_code("SocketRead")?;
		match serde_json::from_str(line.trim_end()).err_code("SocketMessageParse")? {
			Reply::Ok(value) => Ok(value),
			Reply::Err(detail) => Err(Error {
				code: "HostError",
				detail: Some(detail),
			}),
		}
	}
}

impl Request {
	/// Send this request over `conn` and return the reply — the request-side
	/// mirror of [`Connection::request`], so call sites read
	/// `Request::Mount { .. }.send(&mut conn)` (cf. libinvoke's `req.run(pack)`).
	pub fn send(self, conn: &mut Connection) -> Result<Value> {
		conn.request(&self)
	}
}
