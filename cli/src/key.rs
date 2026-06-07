use clap::{Args, Subcommand};
use hid::EventLocation;
use hid::keyboard::Accelerator;

use crate::error::*;
use crate::location::resolve_location;

/// simulate key press, down, up
#[derive(Args)]
pub struct Opts {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
pub enum Command {
	Press(PressOpts),
	Down(DownOpts),
	Up(UpOpts),
}

/// simulate key press (down + up)
#[derive(Args)]
pub struct PressOpts {
	/// key combo (e.g. cmd+shift+e)
	combo: String,
	/// target app bundle identifier
	#[arg(long)]
	app: Option<String>,
}

/// simulate key down
#[derive(Args)]
pub struct DownOpts {
	/// key combo (e.g. cmd+shift+e)
	combo: String,
	/// target app bundle identifier
	#[arg(long)]
	app: Option<String>,
}

/// simulate key up
#[derive(Args)]
pub struct UpOpts {
	/// key combo (e.g. cmd+shift+e)
	combo: String,
	/// target app bundle identifier
	#[arg(long)]
	app: Option<String>,
}

pub fn run(opts: Opts) -> Result {
	let (combo_str, app_ref) = match &opts.command {
		Command::Press(o) => (&o.combo, &o.app),
		Command::Down(o) => (&o.combo, &o.app),
		Command::Up(o) => (&o.combo, &o.app),
	};
	let acc: Accelerator = combo_str.parse().err_code("BadCombo")?;
	let location = match resolve_location(app_ref.as_deref())? {
		Some(pid) => EventLocation::Process { pid },
		None => EventLocation::Hid,
	};

	match opts.command {
		Command::Press(_) => acc.press(location),
		Command::Down(_) => acc.down(location),
		Command::Up(_) => acc.up(location),
	}
	.err_code("HIDKey")
}
