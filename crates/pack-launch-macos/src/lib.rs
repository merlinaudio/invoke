//! macOS launcher for Invoke packs.
//!
//! This crate is a concrete adapter for `invoke::pack`: it spawns the Bun pack
//! runtime under Seatbelt, accepts one Unix-socket connection, forwards decoded
//! `Incoming` messages into `Host::receive`, and writes emitted `Outgoing`
//! messages back to the runtime.
//!
//! The pack engine does not know about any of that. It only sees typed protocol
//! messages. The `Process` guard returned by `spawn` owns the macOS child
//! process and socket. The daemon decides where to store that guard and when
//! unmounting should drop it and remove the pack from `Host`.

use std::{
	fs, io,
	os::unix::fs::FileTypeExt,
	path::{Path, PathBuf},
	process::{Child, ChildStdout, Command, Stdio},
	sync::{
		Arc,
		atomic::{AtomicU64, Ordering},
	},
	time::Duration,
};

use invoke::pack::{Host, Pack, PackHooks, PackId, proto::Outgoing};
use tokio::{
	io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
	net::{
		UnixListener, UnixStream,
		unix::{OwnedReadHalf, OwnedWriteHalf},
	},
	sync::mpsc,
	time::timeout,
};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

static NEXT_SOCKET: AtomicU64 = AtomicU64::new(1);

pub struct Process {
	child: Child,
	socket: PathBuf,
}

impl Process {
	/// The pack's stdout — the author's clean output channel (keep it pure so
	/// `… | jq` works). The launcher always pipes it and hands it over untouched;
	/// what to do with the bytes (write a file, stream to a webview, drop them) is
	/// the orchestrator's call. Whoever holds the `Process` must read this or the
	/// pipe backs up. `None` once taken.
	pub fn stdout(&mut self) -> Option<ChildStdout> {
		self.child.stdout.take()
	}
}

impl Drop for Process {
	fn drop(&mut self) {
		_ = self.child.kill();
		_ = self.child.wait();
		_ = ensure_socket_removed(&self.socket).inspect_err(|e| log::warn!("failed to remove socket in Process::Drop: {e}"));
	}
}

/// Remove the socket file, but only if it's actually a socket. Guards against
/// nuking an arbitrary path if `socket` ever ends up pointing somewhere wrong.
fn ensure_socket_removed(socket: &Path) -> io::Result<()> {
	// Check if path exists:
	if !socket.exists() {
		return Ok(());
	}

	let meta = fs::symlink_metadata(socket)?;

	if meta.file_type().is_socket() {
		fs::remove_file(socket)?;
	}

	Ok(())
}

pub async fn spawn(host: Arc<Host>, id: PackId, hooks: PackHooks, runtime: &Path, root: &Path) -> io::Result<Process> {
	let socket = socket_path()?;
	_ = ensure_socket_removed(&socket).inspect_err(|e| log::error!("failed to remove socket in spawn: {e}"));

	let listener = UnixListener::bind(&socket)?;
	let mut command = command(runtime, root, &socket)?;
	let process = Process {
		child: command.spawn()?,
		socket,
	};

	let stream = accept(listener).await?;
	let (reader, writer) = stream.into_split();
	let (outgoing, outgoing_receiver) = mpsc::unbounded_channel();
	let pack = host.attach_with_hooks(
		id,
		move |message| {
			_ = outgoing.send(message);
		},
		hooks,
	);

	tokio::spawn(read_loop(host, pack, reader));
	tokio::spawn(write_loop(writer, outgoing_receiver));

	Ok(process)
}

fn socket_path() -> io::Result<PathBuf> {
	let n = NEXT_SOCKET.fetch_add(1, Ordering::Relaxed);
	let mut socket = std::env::temp_dir().canonicalize()?;
	socket.push(format!("invoke-pack-{}-{n}.sock", std::process::id()));
	Ok(socket)
}

fn command(runtime: &Path, root: &Path, socket: &Path) -> io::Result<Command> {
	let runtime = runtime.canonicalize()?;
	let root = root.canonicalize()?;
	let runtime_dir = runtime.parent().ok_or_else(|| io::Error::other("runtime has no parent"))?;

	if !socket.is_absolute() {
		return Err(io::Error::new(io::ErrorKind::InvalidInput, "pack socket path must be absolute"));
	}

	let runtime_dir = runtime_dir
		.to_str()
		.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "runtime_dir is not valid UTF-8"))?;
	let runtime = runtime
		.to_str()
		.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "runtime is not valid UTF-8"))?;
	let root = root
		.to_str()
		.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "root is not valid UTF-8"))?;
	let socket = socket
		.to_str()
		.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "socket is not valid UTF-8"))?;

	// sandbox-exec uses network-outbound for Unix-domain sockets too. This is
	// still a single local socket grant, not remote networking.
	let mut command = Command::new("/usr/bin/sandbox-exec");
	command
		.arg("-D")
		.arg(format!("PACK_RUNTIME_DIR={runtime_dir}"))
		.arg("-D")
		.arg(format!("PACK_RUNTIME={runtime}"))
		.arg("-D")
		.arg(format!("PACK_ROOT={root}"))
		.arg("-D")
		.arg(format!("PACK_SOCKET={socket}"))
		.arg("-p")
		.arg(seatbelt_profile())
		.arg(runtime)
		// First runtime argument is the socket path to connect to, second is the
		// pack root. Passed as arguments, not an environment variable, so a runtime
		// in any language reads the socket the same way.
		.arg(socket)
		.arg(root)
		.current_dir("/")
		.stdin(Stdio::null())
		// Always pipe stdout so the orchestrator can read it via `Process::stdout`;
		// never inherit, so pack output can't leak into the orchestrator's own (e.g.
		// launchd) logs. stderr inherits: the separate diagnostics channel.
		.stdout(Stdio::piped())
		.stderr(Stdio::inherit());

	Ok(command)
}

fn seatbelt_profile() -> &'static str {
	// `with message "invoke-pack"` is required in this exact format for the CLI
	// to filter for pack denials.
	r#"(version 1)
; Apple uses a similar format ("foo-bar"), not bundle IDs or anything else,
(deny default (with message "invoke-pack"))

; Allow executing the runtime binary, e.g. Invoke.app/Contents/MacOS/invoke-pack-runtime
(allow process-exec (literal (param "PACK_RUNTIME")))

; Required by Bun as of writing (Bun 1.3.14).
(allow sysctl-read (sysctl-name "hw.pagesize_compat"))

; Allow reading the metadata and contents of the root directory, e.g. /
; Added because the Bun runtime requires it as of writing (Bun 1.3.14).
(allow file-read-data file-read-metadata (literal "/"))

; ICU data, read by the system libicucore the runtime links. Required for e.g. Bun-based runtimes.
; The filename encodes the ICU major version, which is based on the macOS release (26 -> icudt78l),
; so pinning one literal denies reads on every other macOS version.
; Grant the dir (read-only, public static data) so it works across releases.
(allow file-read-data file-read-metadata (subpath "/usr/share/icu"))

; Allow reading the metadata of the runtime binary, e.g. Invoke.app/Contents/MacOS/invoke-pack-runtime
(allow file-read-data file-read-metadata (literal (param "PACK_RUNTIME")))
; ...and the runtime directory, e.g. Invoke.app/Contents/MacOS/
(allow file-read-data file-read-metadata (literal (param "PACK_RUNTIME_DIR")))

; Allow reading the pack's root directory, e.g. Application Support/com.getinvoke.invoke/packs/example.com/mypack
(allow file-read-data file-read-metadata (subpath (param "PACK_ROOT")))

; Allow connecting to the unix socket provided to the pack. Lives in the canonicalized
; $TMPDIR, named by pid, e.g. /private/var/folders/.../T/invoke-pack-<pid>-<n>.sock
(allow network-outbound (literal (param "PACK_SOCKET")))
"#
}

async fn accept(listener: UnixListener) -> io::Result<UnixStream> {
	let (stream, _) = timeout(CONNECT_TIMEOUT, listener.accept())
		.await
		.map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "pack did not connect before timeout"))??;
	Ok(stream)
}

async fn read_loop(host: Arc<Host>, pack: Arc<Pack>, reader: OwnedReadHalf) {
	let reader = BufReader::new(reader);
	let mut lines = reader.lines();
	loop {
		let line = match lines.next_line().await {
			Ok(Some(line)) => line,
			Ok(None) => break,
			Err(error) => {
				log::warn!("pack socket read failed: {error}");
				break;
			}
		};

		match serde_json::from_str(&line) {
			Ok(message) => host.receive(&pack, message),
			Err(error) => {
				log::warn!("pack sent an undecodable line: {error}");
			}
		}
	}
}

async fn write_loop(mut writer: OwnedWriteHalf, mut outgoing: mpsc::UnboundedReceiver<Outgoing>) {
	while let Some(message) = outgoing.recv().await {
		let mut line = match serde_json::to_vec(&message) {
			Ok(line) => line,
			Err(error) => {
				log::error!("failed to encode pack message: {error}");
				continue;
			}
		};
		line.push(b'\n');

		if let Err(error) = writer.write_all(&line).await {
			log::warn!("pack socket write failed: {error}");
			break;
		}
	}
}
