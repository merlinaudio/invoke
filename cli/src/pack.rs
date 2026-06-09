use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use clap::{Args, Subcommand};
use serde_json::json;
use serde_json::Value;

use crate::error::*;
use crate::pipe;
use crate::protocol::Request;
use crate::socket;

const PACK_SEED: &str = r#"import { app, menubar, Role, Subrole, Vars } from "invoke";

export const finder = await app("com.apple.finder");

// Grab one window of Finder.
//
// Hint: Try to make queries specific and unambiguous.
// If multiple windows match the query, a random one may be returned.
const window = finder.$({ role: Role.WINDOW, subrole: Subrole.STANDARD_WINDOW });

// Grab a button inside `window`.
//
// Elements can be queried from another element.
//
// Strings support globbing by default:
// "?" matches any single character, "*" matches any sequence.
//
// This says: find a button inside the window we already named.
//            The button must have an identifier starting with "TrackView.Device",
//            followed by any character sequence.
//
// Queries are evaluated lazily. Nothing happens until you do something with `button`.
const button = window.$({ role: Role.BUTTON, identifier: "TrackView.Device*" });

// Vars are named booleans that whoever invokes this function can read.
//
// For example, a keyboard key binding can say `when: "windowFocused"` to only trigger if `when.windowFocused = true`.
//
// Many packs use this to only enable shortcuts in certain contexts, such as to let users bind single-letter shortcuts
// but keep inputs working with an "inputFocused" var, or to let overlapping, context-sensitive shortcuts coexist.
//
// Ultimately, it is at the discretion of whoever invokes the function to use (or ignore!) the Vars you expose.
const when = Vars({
	windowFocused: false,
});

finder.on("focusedWindowChanged", async () => {
	when.windowFocused = Boolean(await window.focused);
});

// All exported functions get registered as pack functions automatically.
//
// In the UI, users can bind a shortcut to this function on their keyboard, mouse, MIDI, etc.
// In the CLI, you can call this function by running:
//
//     $ invoke pack run {{packName}} doThing
//
export async function doThing() {
	// Press button. If the button doesn't exist, this throws.
	await button.press();

	// You can also simulate HID events, though apps sometimes localize these,
	// and usually the app needs to be focused in order for these to work, so prefer manipulating UI elements directly.
	// Some apps also localize their shortcuts or change them depending on the selected keyboard layout.
	await finder.key.press("cmd+n");

	// Simulate holding down a key for a second:
	await finder.key.down("cmd");
	setTimeout(() => finder.key.up("cmd"), 1000);
	
	// Simulate scrolling down 3 lines:
	await finder.scrolly.y(-3);
}

// There's a convenience function to press a menu bar item by title.
// Keep in mind that this depends on locale.
// If you expect the pack to be distributed to other users, consider whether you want to localize.
export async function newTab() {
	await menubar(finder, "File", "New Tab"); // Presses File->New Tab
}
"#;

/// manage and run pack functions
#[derive(Args)]
pub struct Opts {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
pub enum Command {
	Init(InitOpts),
	List(ListOpts),
	Run(RunOpts),
	/// print a pack's directory path
	Path(PackRef),
	Mount(PackRef),
	Unmount(PackRef),
	Reload(PackRef),
}

/// create a local pack scaffold and print its index.ts path
#[derive(Args)]
pub struct InitOpts {
	/// pack name (e.g. "abletonlive")
	pack: String,
	/// display name written to pack.json (e.g. "Ableton Live")
	#[arg(short = 'n', long)]
	display_name: String,
}

/// list installed packs, packs by publisher, or functions in a pack
#[derive(Args)]
pub struct ListOpts {
	/// publisher domain (disambiguates when multiple packs share a name)
	#[arg(short = 'p', long)]
	publisher: Option<String>,
	/// optional pack name
	pack: Option<String>,
}

/// run a pack function
#[derive(Args)]
pub struct RunOpts {
	/// publisher domain (disambiguates when multiple packs share a name)
	#[arg(short = 'p', long)]
	publisher: Option<String>,
	/// pack name (e.g. abletonlive)
	pack: String,
	/// function name (e.g. zoomIn)
	function: String,
	/// raw JSON payload forwarded to the function
	payload: Option<String>,
}

/// reference to a pack by name, optionally disambiguated by publisher
#[derive(Args)]
pub struct PackRef {
	/// publisher domain (disambiguates when multiple packs share a name)
	#[arg(short = 'p', long)]
	publisher: Option<String>,
	/// pack name (e.g. abletonlive)
	pack: String,
}

fn init_pack(conn: &mut socket::Connection, o: InitOpts) -> Result {
	let icon_text = o.display_name.chars().take(4).collect::<String>();

	let manifest = json!({
		"version": { "major": 0, "minor": 1, "patch": 0 },
		"display": {
			"name": o.display_name,
			"icon": {
				"background": 0x5591f7,
				"color": 0xcae1fa,
				"text": icon_text,
			},
		},
	});

	// The daemon creates the directory + pack.json under the local publisher and
	// returns the pack's path; we seed index.ts ourselves.
	let path = Request::Init {
		pack: o.pack.clone(),
		manifest,
	}
	.send(conn)?;
	let path = path.as_str().err_code("UnexpectedResponse")?;
	let index_path = Path::new(path).join("index.ts");

	let mut file = OpenOptions::new().write(true).create_new(true).open(&index_path).err_code("SeedPack")?;
	let seed = PACK_SEED.replace("{{packName}}", &o.pack);
	file.write_all(seed.as_bytes()).err_code("SeedPack")?;

	println!("{}", index_path.display());

	Ok(())
}

/// Mount the pack before an operation that needs it live. Mounting is idempotent
/// on the daemon, so this no-ops when the pack is already up; it prints to stderr
/// (keeping stdout clean for piping) only when it actually mounts. Wired in only
/// at the call sites that require a mount — running a function and listing a
/// pack's functions — so it can never fire on `unmount`/`init`/etc.
fn ensure_mounted(conn: &mut socket::Connection, publisher: Option<String>, pack: &str) -> Result {
	let newly_mounted = Request::Mount {
		publisher,
		pack: pack.to_owned(),
	}
	.send(conn)?;
	if newly_mounted.as_bool().unwrap_or(false) {
		eprintln!("mounting {pack}...");
	}
	Ok(())
}

pub fn run(opts: Opts) -> Result {
	let mut conn = socket::Connection::open_or_heal()?;
	let result = match opts.command {
		Command::Init(opts) => return init_pack(&mut conn, opts),
		Command::Path(p) => {
			// Print the path raw (not JSON-quoted) so it's usable in `cd "$(invoke pack path x)"`.
			let path = Request::Path {
				publisher: p.publisher,
				pack: p.pack,
			}
			.send(&mut conn)?;
			println!("{}", path.as_str().err_code("UnexpectedResponse")?);
			return Ok(());
		}
		Command::List(opts) => match opts.pack {
			None => {
				let all = Request::List.send(&mut conn)?;
				match opts.publisher {
					None => all,
					Some(publisher) => all.get(&publisher).cloned().unwrap_or(Value::Null),
				}
			}
			Some(pack) => {
				ensure_mounted(&mut conn, opts.publisher.clone(), &pack)?;
				Request::Functions {
					publisher: opts.publisher,
					pack,
				}
				.send(&mut conn)?
			}
		},
		Command::Run(o) => {
			ensure_mounted(&mut conn, o.publisher.clone(), &o.pack)?;
			Request::Run {
				publisher: o.publisher,
				pack: o.pack,
				function: o.function,
				payload: o.payload,
			}
			.send(&mut conn)?
		}
		Command::Mount(p) => Request::Mount {
			publisher: p.publisher,
			pack: p.pack,
		}
		.send(&mut conn)?,
		Command::Unmount(p) => Request::Unmount {
			publisher: p.publisher,
			pack: p.pack,
		}
		.send(&mut conn)?,
		Command::Reload(p) => Request::Reload {
			publisher: p.publisher,
			pack: p.pack,
		}
		.send(&mut conn)?,
	};
	pipe::write_json_line(&result)
}

/// Catchall handler for `invoke <pack> [function] [payload]`. One arg lists functions;
/// two or three args run the function. More args are an error.
pub fn run_catchall(args: Vec<String>) -> Result {
	let opts = match args.len() {
		1 => Opts {
			command: Command::List(ListOpts {
				publisher: None,
				pack: Some(args.into_iter().next().unwrap()),
			}),
		},
		2 => {
			let mut it = args.into_iter();
			Opts {
				command: Command::Run(RunOpts {
					publisher: None,
					pack: it.next().unwrap(),
					function: it.next().unwrap(),
					payload: None,
				}),
			}
		}
		3 => {
			let mut it = args.into_iter();
			Opts {
				command: Command::Run(RunOpts {
					publisher: None,
					pack: it.next().unwrap(),
					function: it.next().unwrap(),
					payload: it.next(),
				}),
			}
		}
		n => return Err(Error::new("TooManyArgs", format!("expected 1, 2, or 3 args, got {n}"))),
	};
	run(opts)
}
