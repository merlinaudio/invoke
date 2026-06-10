use clap::{Parser, Subcommand};
use std::process;

pub const LOCAL_PUBLISHER_DOMAIN: &str = "invoke.localhost";

mod app;
mod element;
mod error;
mod key;
mod location;
mod pack;
mod pipe;
mod protocol;
mod sandbox;
mod scroll;
mod service;
mod socket;
mod upgrade;

mod util;

/// invoke CLI
#[derive(Parser)]
#[command(name = "invoke", version, about, disable_help_subcommand = true)]
struct Global {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	App(app::Opts),
	Element(element::Opts),
	Key(key::Opts),
	Scroll(scroll::Opts),
	Pack(pack::Opts),
	/// print a pack's stdout log (`-f` to follow)
	Logs(pack::LogsOpts),
	Sandbox(sandbox::Opts),
	Service(service::Opts),
	Upgrade(upgrade::Opts),

	/// `invoke <pack> [function]` — dispatches to `pack list` / `pack run`.
	#[command(external_subcommand)]
	Catchall(Vec<String>),
}

fn main() {
	let global = Global::parse();

	let result = match global.command {
		Command::App(opts) => app::run(opts),
		Command::Element(opts) => element::run(opts),
		Command::Key(opts) => key::run(opts),
		Command::Scroll(opts) => scroll::run(opts),
		Command::Pack(opts) => pack::run(opts),
		Command::Logs(opts) => pack::logs(opts),
		Command::Sandbox(opts) => sandbox::run(opts),
		Command::Service(opts) => service::run(opts),
		Command::Upgrade(opts) => upgrade::run(opts),
		Command::Catchall(args) => pack::run_catchall(args),
	};

	if let Err(e) = result {
		eprintln!("error: {e}");
		process::exit(1);
	}
}
