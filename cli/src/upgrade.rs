use std::fs::canonicalize;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use backon::{BlockingRetryable, ExponentialBuilder};
use clap::Args;
use serde::Deserialize;

use crate::error::*;

/// Upgrade Invoke. Only works when this CLI shipped inside Invoke.app;
/// other installs upgrade via the package manager that installed them.
///
///   invoke upgrade
#[derive(Args)]
pub struct Opts {}

/// Events the bundle's entrypoint streams as NDJSON on stdout. electrobun runs
/// that entrypoint in a Worker and the process exits 0 regardless, so exit
/// codes can't carry results; lines that don't parse are launcher noise.
#[derive(Deserialize)]
#[serde(tag = "update", rename_all = "camelCase")]
enum Update {
	Current { version: String },
	Available { version: String },
	Progress { message: String },
	Installing { version: String },
	Error { message: String },
}

pub fn run(_: Opts) -> Result {
	let exe = std::env::current_exe().and_then(canonicalize).err_code("resolve_cli_path")?;

	let Some(bundle) = exe
		.ancestors()
		.nth(3)
		.filter(|bundle| bundle.extension() == Some("app".as_ref()) && exe.strip_prefix(bundle) == Ok(Path::new("Contents/MacOS/invoke")))
	else {
		return Err(Error::new(
			"NotVendoredThroughAppBundle",
			format!(
				"this CLI is not part of Invoke.app, but \"{}\"\nplease upgrade the Invoke CLI the way it was installed (e.g. through brew, npm, or manually)",
				exe.display()
			),
		));
	};

	println!("Checking for update...");

	// Check first so a no-op upgrade never disturbs a running app.
	match run_updater_command(bundle, "check")? {
		Update::Current { version } => {
			println!("Already up to date: {version}");
			Ok(())
		}
		Update::Available { version } => {
			println!("Update available: {version}");
			quit_app(bundle)?;
			match run_updater_command(bundle, "immediate")? {
				Update::Installing { version } => {
					println!("Updated to {version}");
					Ok(())
				}
				Update::Error { message } => Err(Error::new("UpdaterApply", message)),
				_ => Err(Error::code("UpdaterApply")),
			}
		}
		Update::Error { message } => Err(Error::new("UpdaterCheck", message)),
		_ => Err(Error::code("UpdaterCheck")),
	}
}

/// Run the bundle's updater ("check" or "immediate") via its launcher, print
/// progress, and return the last result event once it exits.
fn run_updater_command(bundle: &Path, mode: &str) -> Result<Update> {
	let mut child = Command::new(bundle.join("Contents/MacOS/launcher"))
		.env("INVOKE_UPDATE", mode)
		.stdout(Stdio::piped())
		.spawn()
		.err_code("UpdaterLaunch")?;

	let reader = BufReader::new(child.stdout.take().err_code("UpdaterLaunch")?);

	// Last-emitted event from the updater is the result.
	let mut result = None;

	for line in reader.lines() {
		match serde_json::from_str(&line.err_code("UpdaterLaunch")?) {
			Ok(Update::Progress { message }) => println!("{message}"),
			Ok(update) => result = Some(update),
			Err(_) => {}
		}
	}

	child.wait().err_code("UpdaterLaunch")?;
	result.err_code("UpdaterNoResult") // The updater always emits something to stdout. If none, something is wrong.
}

/// Gracefully quit the app if it's running, waiting for it to exit so the bundle swap doesn't race a live process.
fn quit_app(bundle: &Path) -> Result {
	let app = bundle.file_stem().and_then(|s| s.to_str()).err_code("NotVendoredThroughAppBundle")?;

	// `application X is running` doesn't launch the app the way a bare `tell` would.
	let running = format!(r#"application "{app}" is running"#);
	if osascript(&running)? != "true" {
		return Ok(());
	}
	osascript(&format!(r#"tell application "{app}" to quit"#))?;

	let quit = || match osascript(&running)?.as_str() {
		"false" => Ok(()),
		_ => Err(Error::new("AppDidNotQuit", format!("{app} did not quit; close it and retry"))),
	};
	quit.retry(ExponentialBuilder::default().with_min_delay(Duration::from_millis(200)).with_max_times(8))
		.sleep(std::thread::sleep)
		.when(|e| e.code == "AppDidNotQuit")
		.call()
}

fn osascript(expr: &str) -> Result<String> {
	let out = Command::new("osascript").args(["-e", expr]).output().err_code("osascript")?;
	if !out.status.success() {
		return Err(Error::new("osascript", String::from_utf8_lossy(&out.stderr)));
	}
	Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
