use std::path::Path;
use std::process::Command;
use std::time::Duration;

use backon::{BlockingRetryable, ExponentialBuilder};
use clap::Args;

use crate::error::*;

/// Upgrade Invoke. Only works when this CLI shipped inside Invoke.app;
/// other installs upgrade via the package manager that installed them.
///
///   invoke upgrade
#[derive(Args)]
pub struct Opts {}

// Same convention as `dnf check-update`: 0 = up to date, 100 = update available.
const UPDATE_AVAILABLE: i32 = 100;

pub fn run(_: Opts) -> Result {
	let exe = std::env::current_exe().and_then(std::fs::canonicalize).err_code("resolve_cli_path")?;

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

	// Check first so a no-op upgrade never disturbs a running app.
	match updater(bundle, "check")? {
		0 => return Ok(()),
		UPDATE_AVAILABLE => {
			quit_app(bundle)?;
			match updater(bundle, "immediate")? {
				0 => Ok(()),
				_ => Err(Error::code("UpdaterFailedToApply")),
			}
		}
		_ => return Err(Error::code("UpdaterFailedToCheck")),
	}
}

/// The bundle's own entrypoint handles INVOKE_UPDATE without loading anything
/// else: "check" exits 10 if an update is available, "immediate" applies it
/// (relaunching the app afterward).
fn updater(bundle: &Path, mode: &str) -> Result<i32> {
	let status = Command::new(bundle.join("Contents/MacOS/launcher"))
		.env("INVOKE_UPDATE", mode)
		.status()
		.err_code("UpdaterFailedToLaunch")?;

	status.code().err_code("UpdaterFailedToLaunch")
}

/// Gracefully quit the app if it's running, waiting for it to exit so the
/// bundle swap doesn't race a live process.
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
