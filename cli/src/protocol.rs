//! The CLI ↔ daemon wire vocabulary. Dead-simple NDJSON: one JSON `Request`
//! line in, one JSON `Reply` line out, as many round-trips as the connection
//! lives for. Same shape on both sides (client `socket::Connection`, daemon
//! `listen::connection`), exactly like libinvoke's pack protocol — serde-tagged
//! enums over newline-framed JSON.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A pack command from the CLI to the `invoke listen` daemon.
///
/// `publisher` is optional: when `None`, the daemon resolves it by finding the
/// sole installed pack with that name (it owns the packs directory).
#[derive(Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Request {
	/// Every installed pack, as `{ publisher: { pack: manifest } }`.
	List,
	/// Filesystem path of a pack's directory.
	Path { publisher: Option<String>, pack: String },
	/// Create a local pack directory with the given `pack.json`; returns its path.
	Init { pack: String, manifest: Value },
	Mount { publisher: Option<String>, pack: String },
	Unmount { publisher: Option<String>, pack: String },
	Reload { publisher: Option<String>, pack: String },
	/// Names of the functions a mounted pack has registered.
	Functions { publisher: Option<String>, pack: String },
	Run {
		publisher: Option<String>,
		pack: String,
		function: String,
		/// Raw JSON payload forwarded to the function.
		payload: Option<String>,
	},
}

/// The daemon's answer to one `Request`.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Reply {
	Ok(Value),
	Err(String),
}
