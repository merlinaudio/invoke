//! `invoke service` — the standalone orchestrator and its launch-agent wrapper.
//!
//! `run` is the orchestator itself: it owns an `invoke::pack::Host`, spawns pack
//! runtimes through the macOS launcher, and serves the same Unix socket the
//! CLI's pack commands connect to (dead-simple NDJSON, see `crate::protocol`).
//! This is the piece that lets the CLI host packs without Invoke.app.
//!
//! `install`/`status` register that orchestator as a per-user launchd agent (via the
//! `service-manager` crate) so it starts at login and stays up in the background
//! — surfacing under System Settings → Login Items & Extensions ("Allow in the
//! Background"). There is deliberately no uninstall: the CLI never deletes files.
//!
//! Threading: every AX/keyboard/mouse op in libinvoke is marshaled onto the
//! main thread's dispatch queue, and AX observers fire on the main run loop. So
//! the main thread runs `CFRunLoop::run()` forever, and all of tokio (the
//! socket server, pack I/O, `Host::receive`) runs on a side runtime.

use std::collections::HashMap;
use std::env;
use std::fs::canonicalize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use clap::{Args, Subcommand};
use objc2_core_foundation::CFRunLoop;
use serde_json::{Map, Value};
use service_manager::{
	LaunchdServiceManager, RestartPolicy, ServiceInstallCtx, ServiceLabel, ServiceManager, ServiceStartCtx, ServiceStatus, ServiceStatusCtx,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use invoke::pack::{Function, Host, PackHooks, PackId};
use pack_launch_macos::Process;

use crate::LOCAL_PUBLISHER_DOMAIN;
use crate::error::*;
use crate::protocol::{Reply, Request};

/// manage the pack-hosting orchestator (run it, or install it as a launch agent)
#[derive(Args)]
pub struct Opts {
	#[command(subcommand)]
	command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
	/// run the orchestator in the foreground (what the launch agent invokes)
	Run(RunOpts),
	/// install `invoke service run` to start at login (launch agent). also starts it now.
	Install,
	/// start the installed launch agent (also enables start-at-login)
	Start,
	/// show the plist path and whether launchd has the agent loaded
	Status,
}

#[derive(Args)]
struct RunOpts {
	/// path to the pack runtime binary
	#[arg(long, default_value_os_t = default_runtime())]
	runtime: PathBuf,
	/// socket to listen on (defaults to the shared app socket)
	#[arg(long)]
	socket: Option<PathBuf>,
}

/// launchd job label and the plist filename derived from it.
const LABEL: &str = "com.getinvoke.invoke";

pub fn run(opts: Opts) -> Result {
	match opts.command {
		Cmd::Run(opts) => run_orchestrator(opts),
		Cmd::Install => install(),
		Cmd::Start => start(),
		Cmd::Status => status(),
	}
}

// ---- launch agent management ------------------------------------------------

fn label() -> ServiceLabel {
	LABEL.parse().expect("LABEL is a valid service label")
}

/// `~/Library/LaunchAgents/<LABEL>.plist`. `service-manager` writes/loads via
/// `dirs::home_dir()` (its `user_agent_dir_path`); on macOS that's just `$HOME`,
/// identical to `std::env::home_dir()`, so the no-clobber guard below resolves
/// the same file the crate operates on — without a second copy of `dirs`.
fn launchagent_plist_path() -> Result<PathBuf> {
	Ok(std::env::home_dir()
		.err_code("HomeDir")?
		.join("Library/LaunchAgents")
		.join(format!("{LABEL}.plist")))
}

/// Register the orchestator as a per-user launch agent, then start it.
///
/// We refuse rather than let the crate's `install()` overwrite an existing plist
/// (it would `launchctl remove` + truncate; no `O_EXCL` is exposed, and the race
/// is benign for a user-run verb). Because `RestartPolicy::OnFailure` makes the
/// crate emit a `KeepAlive` dict, it also writes `Disabled=true` — so `install()`
/// alone loads the job dormant. The trailing `start()` strips `Disabled` and
/// reloads, which runs it now and lets `RunAtLoad` start it at every login.
/// `ServiceInstallCtx` has no `Default`, hence every field is spelled out.
fn install() -> Result {
	let path = launchagent_plist_path()?;
	let hint = format!("launchctl unload {path} && rm {path}", path = path.display());
	if path.exists() {
		return Err(Error::new(
			"AlreadyInstalled",
			format!("{path} exists; remove it first — {hint}", path = path.display()),
		));
	}

	let program = env::current_exe().and_then(canonicalize).err_code("CurrentExe")?;

	LaunchdServiceManager::user()
		.install(ServiceInstallCtx {
			label: label(),
			program,
			args: vec!["service".into(), "run".into()],
			autostart: true,
			restart_policy: RestartPolicy::OnFailure {
				delay_secs: Some(6),
				max_retries: Some(10),
				reset_after_secs: None,
			},
			contents: None,
			username: None,
			working_directory: None,
			environment: None,
		})
		.err_code("Install")?;

	start()?;
	println!("installed and started {}\nto remove: {hint}", path.display());
	Ok(())
}

/// Start (or restart) the installed agent. Clears the `Disabled` flag the crate
/// sets for `KeepAlive` jobs, so it also starts at login thereafter.
fn start() -> Result {
	LaunchdServiceManager::user().start(ServiceStartCtx { label: label() }).err_code("Start")
}

/// Force-restart the launch agent so launchd execs the current on-disk binary and
/// rebinds the socket. `kickstart -k` kills the running instance first, so it
/// recovers a *wedged* daemon (process alive but listener dead — e.g. an update
/// stranded the old socket file) as well as a stopped one. `launchctl start` is no
/// good here: it no-ops on a `KeepAlive` job launchd already believes is running.
pub fn kickstart() -> Result {
	let plist = launchagent_plist_path()?;
	if !plist.exists() {
		return Err(Error::new("NotInstalled", "agent not installed — run `invoke service install`"));
	}
	// Restart through the plist (no uid/domain target needed — same as service-manager's
	// `start`): `unload` terminates the job, KeepAlive and all, then `load` brings it
	// back fresh on the current binary. `launchctl start` won't do — it no-ops on a
	// KeepAlive job launchd already believes is running.
	let _ = Command::new("launchctl").arg("unload").arg(&plist).output(); // ignored: job may not be loaded
	let out = Command::new("launchctl").arg("load").arg(&plist).output().err_code("Kickstart")?;
	if out.status.success() {
		Ok(())
	} else {
		Err(Error::new("Kickstart", String::from_utf8_lossy(&out.stderr).trim().to_owned()))
	}
}

fn status() -> Result {
	let state = match LaunchdServiceManager::user().status(ServiceStatusCtx { label: label() }).err_code("Status")? {
		ServiceStatus::Running => "running",
		ServiceStatus::Stopped(_) => "installed, not running",
		ServiceStatus::NotInstalled => "not installed",
	};
	println!("plist:  {}\nstatus: {state}", launchagent_plist_path()?.display());
	Ok(())
}

// ---- the orchestator -------------------------------------------------------------

/// A mounted pack: the launcher guard whose drop kills the child, plus the
/// function names the pack registered (collected through the `function_defined`
/// hook so `Functions` can list them without a libinvoke change).
struct Mount {
	_process: Process,
	functions: Arc<Mutex<Vec<String>>>,
}

struct Orchestrator {
	host: Arc<Host>,
	mounts: Mutex<HashMap<PackId, Mount>>,
	runtime: PathBuf,
	packs_dir: PathBuf,
}

/// A handler result: a JSON value, or a human-readable error string (becomes `Reply::Err`).
type Handled = std::result::Result<Value, String>;

/// How long `mount` waits for a freshly-spawned pack to signal `Ready` before
/// giving up and returning anyway (the pack stays mounted).
const READY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

fn run_orchestrator(opts: RunOpts) -> Result {
	ensure_executable(&opts.runtime)?;
	let socket = match opts.socket {
		Some(socket) => socket,
		None => crate::socket::socket_path()?.to_path_buf(),
	};
	let packs_dir = common::process::application_support_dir()
		.err_code("AppSupportDir")?
		.join("com.getinvoke.invoke")
		.join("packs");

	let orchestrator = Arc::new(Orchestrator {
		host: Arc::new(Host::new()),
		mounts: Mutex::new(HashMap::new()),
		runtime: opts.runtime,
		packs_dir,
	});

	let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().err_code("Tokio")?;
	rt.spawn(serve(orchestrator, socket));

	// Blocks the main thread forever, draining the main dispatch queue (so
	// libinvoke's `MainThread::run` resolves) and AX observer callbacks. `rt`
	// stays owned in this scope, keeping the server alive.
	CFRunLoop::run();
	Ok(())
}

/// Default for `--runtime`
///
/// The pack runtime ships next to the `invoke` binary in
/// every layout — `Contents/MacOS` in the .app, the package's dir under npm, `libexec/`
/// under brew, the cargo `bin/` dir, ...
///
/// So we just look beside ourselves.
///
/// The app bundle ships the runtime beside the CLI binary; keep them together or this breaks.
fn default_runtime() -> PathBuf {
	// canonicalize first because the cli is usually a symlink (/opt/homebrew/bin/invoke, <npm-prefix>/bin/invoke),
	// and the runtime sits next to the real file, not the symlink.
	std::env::current_exe()
		.and_then(std::fs::canonicalize)
		.ok()
		.and_then(|exe| Some(exe.parent()?.join("invoke-pack-runtime")))
		.unwrap_or_else(|| PathBuf::from("invoke-pack-runtime"))
}

/// The pack runtime must be an existing executable file. Validate it at startup
/// so a bad `--runtime` (or a broken bundle) fails here, not when the first pack
/// is mounted and the launcher tries to exec it.
fn ensure_executable(path: &Path) -> Result {
	use std::os::unix::fs::PermissionsExt;
	let meta = std::fs::metadata(path).map_err(|e| Error::new("BadRuntime", format!("pack runtime {}: {e}", path.display())))?;
	if !meta.is_file() {
		return Err(Error::new("BadRuntime", format!("pack runtime is not a file: {}", path.display())));
	}
	if meta.permissions().mode() & 0o111 == 0 {
		return Err(Error::new("BadRuntime", format!("pack runtime is not executable: {}", path.display())));
	}
	Ok(())
}

async fn serve(orchestator: Arc<Orchestrator>, socket: PathBuf) {
	if !invoke::is_process_trusted() {
		log::warn!("Accessibility is not granted to this process — pack AX calls will fail. Grant it in System Settings → Privacy & Security → Accessibility.");
	}

	// A leftover socket file blocks bind(). Reclaim it ONLY when it is provably a
	// dead socket — never anything that could be real user data. Two gates: the
	// path's own inode is a socket (`symlink_metadata`, so a regular file/dir or
	// a symlink is left untouched), AND nothing is listening (so we never yank a
	// live socket out from under another invoke orchestator or Invoke.app).
	{
		use std::os::unix::fs::FileTypeExt;
		let dead_socket = std::fs::symlink_metadata(&socket).map(|m| m.file_type().is_socket()).unwrap_or(false) && UnixStream::connect(&socket).await.is_err();
		if dead_socket {
			let _ = std::fs::remove_file(&socket);
		}
	}

	let listener = match UnixListener::bind(&socket) {
		Ok(listener) => listener,
		Err(error) => {
			log::error!(
				"failed to bind {}: {error} (is another invoke orchestator or Invoke.app already running?)",
				socket.display()
			);
			return;
		}
	};
	log::info!("invoke listening on {socket:?}");

	loop {
		match listener.accept().await {
			Ok((stream, _)) => {
				tokio::spawn(connection(orchestator.clone(), stream));
			}
			Err(error) => log::warn!("accept failed: {error}"),
		}
	}
}

/// One client connection: read NDJSON requests, write one NDJSON reply each.
async fn connection(orchestator: Arc<Orchestrator>, stream: UnixStream) {
	let (reader, mut writer) = stream.into_split();
	let mut lines = BufReader::new(reader).lines();

	while let Ok(Some(line)) = lines.next_line().await {
		if line.trim().is_empty() {
			continue;
		}

		let reply = match serde_json::from_str::<Request>(&line) {
			Ok(request) => match orchestator.handle(request).await {
				Ok(value) => Reply::Ok(value),
				Err(detail) => Reply::Err(detail),
			},
			Err(error) => Reply::Err(format!("bad request: {error}")),
		};

		let mut bytes = serde_json::to_vec(&reply).unwrap_or_else(|_| br#"{"err":"encode failed"}"#.to_vec());
		bytes.push(b'\n');
		if writer.write_all(&bytes).await.is_err() {
			break;
		}
	}
}

impl Orchestrator {
	async fn handle(self: &Arc<Self>, request: Request) -> Handled {
		match request {
			Request::List => Ok(self.list()),
			Request::Init { pack, manifest } => self.init(&pack, &manifest),
			Request::Path { publisher, pack } => {
				let dir = self.pack_dir(&self.resolve(publisher, &pack)?, &pack);
				Ok(Value::from(dir.to_string_lossy().into_owned()))
			}
			Request::Mount { publisher, pack } => self.mount(self.resolve(publisher, &pack)?, pack).await,
			Request::Unmount { publisher, pack } => self.unmount(&PackId::new(self.resolve(publisher, &pack)?, &pack)),
			Request::Reload { publisher, pack } => {
				let publisher = self.resolve(publisher, &pack)?;
				let _ = self.unmount(&PackId::new(&publisher, &pack));
				self.mount(publisher, pack).await
			}
			Request::Functions { publisher, pack } => {
				let id = PackId::new(self.resolve(publisher, &pack)?, &pack);
				self.mounts
					.lock()
					.unwrap()
					.get(&id)
					.map(|mount| Value::from(mount.functions.lock().unwrap().clone()))
					.ok_or_else(|| not_mounted(&id))
			}
			Request::Run {
				publisher,
				pack,
				function,
				payload,
			} => {
				let id = PackId::new(self.resolve(publisher, &pack)?, &pack);
				let pack = self.host.get(&id).ok_or_else(|| not_mounted(&id))?;
				// The pack runtime's `payload` is an opaque, separately-encoded JSON string
				// (it re-parses with a reviver that rebuilds Element handles). The CLI's
				// `--payload` is already serialized JSON text, so pass it straight through;
				// absent → the string "null".
				pack.run_function(&function, Value::String(payload.unwrap_or_else(|| "null".to_owned()))).await
			}
		}
	}

	fn pack_dir(&self, publisher: &str, pack: &str) -> PathBuf {
		self.packs_dir.join(publisher).join(pack)
	}

	/// `{ publisher: { pack: manifest } }` for every installed pack on disk.
	fn list(&self) -> Value {
		let mut out = Map::new();
		let Ok(publishers) = std::fs::read_dir(&self.packs_dir) else {
			return Value::Object(out); // no packs dir yet → empty
		};
		for publisher in publishers.flatten() {
			let Ok(packs) = std::fs::read_dir(publisher.path()) else { continue };
			let mut entries = Map::new();
			for pack in packs.flatten() {
				let manifest = pack.path().join("pack.json");
				if let Ok(text) = std::fs::read_to_string(&manifest)
					&& let Ok(value) = serde_json::from_str::<Value>(&text)
				{
					entries.insert(pack.file_name().to_string_lossy().into_owned(), value);
				}
			}
			if !entries.is_empty() {
				out.insert(publisher.file_name().to_string_lossy().into_owned(), Value::Object(entries));
			}
		}
		Value::Object(out)
	}

	/// Resolve `publisher`. If given, returned as-is. Otherwise find the sole
	/// installed pack with this name; error on zero or multiple matches.
	fn resolve(&self, publisher: Option<String>, pack: &str) -> std::result::Result<String, String> {
		if let Some(publisher) = publisher {
			return Ok(publisher);
		}
		let publishers = std::fs::read_dir(&self.packs_dir).map_err(|e| e.to_string())?;
		let mut matches: Vec<String> = publishers
			.flatten()
			.filter(|publisher| publisher.path().join(pack).join("pack.json").is_file())
			.map(|publisher| publisher.file_name().to_string_lossy().into_owned())
			.collect();
		match matches.len() {
			0 => Err(format!("no installed pack named '{pack}'")),
			1 => Ok(matches.pop().unwrap()),
			_ => Err(format!(
				"multiple packs named '{pack}': {}; disambiguate with -p <publisher>",
				matches.join(", ")
			)),
		}
	}

	/// Create a local pack directory with `pack.json`; return its path so the
	/// client can drop an `index.ts` seed in.
	fn init(&self, pack: &str, manifest: &Value) -> Handled {
		let dir = self.pack_dir(LOCAL_PUBLISHER_DOMAIN, pack);
		if dir.join("pack.json").exists() {
			return Err(format!("pack already exists: {}", dir.display()));
		}
		std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
		let text = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())? + "\n";
		std::fs::write(dir.join("pack.json"), text).map_err(|e| e.to_string())?;
		Ok(Value::from(dir.to_string_lossy().into_owned()))
	}

	/// Mount the pack if it isn't already. Idempotent so callers can auto-mount
	/// before a `run` without racing an explicit mount: returns `true` if this
	/// call started the pack, `false` if it was already up.
	async fn mount(self: &Arc<Self>, publisher: String, pack: String) -> Handled {
		let id = PackId::new(&publisher, &pack);
		if self.mounts.lock().unwrap().contains_key(&id) {
			return Ok(Value::Bool(false));
		}

		let root = self.pack_dir(&publisher, &pack);
		if !root.join("pack.json").is_file() {
			return Err(format!("no pack at {}", root.display()));
		}

		let functions = Arc::new(Mutex::new(Vec::new()));
		let hooks = PackHooks {
			function_defined: Some(Box::new({
				let functions = functions.clone();
				move |function: &Function| functions.lock().unwrap().push(function.name.clone())
			})),
			..Default::default()
		};

		let process = pack_launch_macos::spawn(self.host.clone(), id.clone(), hooks, &self.runtime, &root)
			.await
			.map_err(|e| e.to_string())?;

		self.mounts.lock().unwrap().insert(id.clone(), Mount { _process: process, functions });

		// `spawn` returns as soon as the pack's socket connects — before the pack
		// runtime has executed its entrypoint and registered its functions. Wait
		// for its `Ready` signal so a subsequent `run` finds them. Best-effort: a
		// pack that errors during init never signals ready, so don't wedge the
		// mount forever — the pack is mounted either way.
		if let Some(pack) = self.host.get(&id) {
			if tokio::time::timeout(READY_TIMEOUT, pack.ready()).await.is_err() {
				log::warn!("pack {}/{} did not signal ready within {READY_TIMEOUT:?}", id.publisher_domain, id.pack_name);
			}
		}
		Ok(Value::Bool(true))
	}

	/// Drop the launcher guard (kills the child) and stop exposing the pack.
	fn unmount(&self, id: &PackId) -> Handled {
		if self.mounts.lock().unwrap().remove(id).is_none() {
			return Err(not_mounted(id));
		}
		self.host.remove(id);
		Ok(Value::Null)
	}
}

fn not_mounted(id: &PackId) -> String {
	format!("pack not mounted: {} — run `invoke pack mount {}` first", id.pack_name, id.pack_name)
}
