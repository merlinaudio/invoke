use clap::{Args, Subcommand};
use serde_json::Value as Json;

use crate::element;
use crate::error::*;
use crate::pipe;

/// Query running applications.
///
///   invoke app list
///   invoke app get com.ableton.live
///   invoke app get com.ableton.live title children
#[derive(Args)]
pub struct Opts {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
pub enum Command {
	Get(GetOpts),
	List(ListOpts),
}

/// Get attributes of an application's top-level accessibility element.
/// Lists all available attributes by default, or pass specific names.
///
///   invoke app get com.ableton.live
///   invoke app get com.ableton.live children title
#[derive(Args)]
pub struct GetOpts {
	/// app bundle ID (e.g. com.ableton.live)
	app: String,
	/// attribute names to fetch (omit for all)
	attrs: Vec<String>,
}

/// List running applications that can be queried via accessibility.
///
///   invoke app list
#[derive(Args)]
pub struct ListOpts {}

pub fn run(opts: Opts) -> Result {
	pipe::write_json_line(&exec(opts.command)?)
}

fn exec(command: Command) -> Result<Json> {
	match command {
		Command::Get(o) => {
			let mut args = vec![o.app, "[]".into()];
			args.extend(o.attrs);
			element::get(args)
		}
		Command::List(_) => {
			let list: Vec<Json> = common::process::running_applications()
				.iter()
				.filter_map(|app| {
					let bundle_id = app.bundleIdentifier()?.to_string();
					let name = app.localizedName().map(|n| n.to_string()).unwrap_or_default();
					Some(serde_json::json!({ "id": bundle_id, "name": name }))
				})
				.collect();
			Ok(Json::Array(list))
		}
	}
}
