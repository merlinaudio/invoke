use clap::{Args, Subcommand};
use hid::EventLocation;
use hid::mouse::ScrollWheel;

use crate::error::*;
use crate::location::resolve_location;

/// simulate scroll wheel
#[derive(Args)]
pub struct Opts {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
pub enum Command {
	Y(YOpts),
	X(XOpts),
}

/// scroll vertically (positive = down)
#[derive(Args)]
pub struct YOpts {
	/// scroll delta (lines)
	delta: i32,
	/// target app bundle identifier
	#[arg(long)]
	app: Option<String>,
}

/// scroll horizontally (positive = right)
#[derive(Args)]
pub struct XOpts {
	/// scroll delta (lines)
	delta: i32,
	/// target app bundle identifier
	#[arg(long)]
	app: Option<String>,
}

pub fn run(opts: Opts) -> Result {
	let (wheel, delta, app) = match &opts.command {
		Command::Y(o) => (ScrollWheel::Y, o.delta, &o.app),
		Command::X(o) => (ScrollWheel::X, o.delta, &o.app),
	};

	let location = match resolve_location(app.as_deref())? {
		Some(pid) => EventLocation::Process { pid },
		None => EventLocation::Hid,
	};

	wheel.scroll(delta, location).err_code("HIDScroll")
}
