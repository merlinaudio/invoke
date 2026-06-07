use crate::util::duration::{DurationExt, NSDateExt};
use clap::{Args, Subcommand};
use objc2::rc::Retained;
use objc2_foundation::{NSPredicate, NSString};
use objc2_os_log::{OSLogEntry, OSLogEnumeratorOptions, OSLogStore};

use crate::error::*;

/// inspect pack sandbox behavior
#[derive(Args)]
pub struct Opts {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
pub enum Command {
	/// print recent Seatbelt denials for Invoke pack processes
	Log,
}

pub fn run(opts: Opts) -> Result {
	match opts.command {
		Command::Log => log(),
	}
}

fn log() -> Result {
	let entries = unsafe {
		let store = OSLogStore::localStoreAndReturnError().err_code("GetOSLogStore")?;
		let position = store.positionWithDate(&600.seconds().ago());

		let predicate = NSPredicate::predicateWithFormat_argumentArray(
			&NSString::from_str("composedMessage CONTAINS 'Sandbox: bun(' AND composedMessage CONTAINS 'deny(' AND composedMessage CONTAINS 'invoke-pack'"),
			None,
		);

		store
			.entriesEnumeratorWithOptions_position_predicate_error(OSLogEnumeratorOptions(0), Some(&position), Some(&predicate))
			.map_err(|e| Error::new("OSLogEntries", format!("{e:?}")))?
	};

	for entry in entries.iter() {
		let entry: Retained<OSLogEntry> = unsafe { Retained::cast_unchecked(entry) };
		let message = unsafe { entry.composedMessage() }.to_string();
		if let Some(message) = sandbox_message(&message) {
			println!("{message}");
		}
	}

	Ok(())
}

fn sandbox_message(line: &str) -> Option<String> {
	Some(line.rsplit_once("Sandbox: ")?.1.lines().next()?.replace(" deny(1) ", " deny "))
}
