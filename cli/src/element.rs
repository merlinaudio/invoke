use clap::{Args, Subcommand};
use common::accessibility::{
	Element,
	action::Action,
	attribute::Attribute,
	filter::{Filter, FilterPath},
	orientation::Orientation,
	role::Role,
	sort_direction::SortDirection,
	subrole::Subrole,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::error::*;
use crate::pipe;

/// Query and manipulate accessibility elements.
///
/// Every subcommand takes an app bundle ID and a JSON query path that walks
/// the element tree. The query is a JSON array of filter steps, where each
/// step is an object whose keys are attributes and values are match patterns:
///
///   invoke element get com.app.id '[{"role": "menuBar"}, {"title": "Edit"}]'
///
/// String values match as globs (use * and ?). Role and subrole accept
/// camelCase names ("window", "menuBar") — they're normalized to AX constants
/// internally.
///
/// Use `walk` to explore the tree structure, or `get children` to list
/// an element's direct children. Commands that mutate (set, perform)
/// output a descriptor that can be piped into other commands:
///
///   invoke element perform com.app.id '[{"role": "button"}]' press | invoke element get value
#[derive(Args)]
pub struct Opts {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
pub enum Command {
	Walk(WalkOpts),
	Get(GetOpts),
	Set(SetOpts),
	Perform(PerformOpts),
	Actions(ActionsOpts),
}

/// Recursively walk an element's children, printing a tree of attributes.
///
///   invoke element walk com.ableton.live '[{"role": "window"}]'
///   invoke element walk com.ableton.live '[{"role": "menuBar"}]' -d 1
#[derive(Args)]
pub struct WalkOpts {
	/// app bundle ID (e.g. com.ableton.live)
	app: String,
	/// JSON query path (e.g. '[{"role": "window"}]'); defaults to the app root
	#[arg(default_value = "[]")]
	query: String,
	/// max depth to recurse into children
	#[arg(short = 'd', long, default_value_t = 3)]
	depth: usize,
	/// keep attributes with empty/zero/default values (omitted by default)
	#[arg(long)]
	full: bool,
}

/// Get attribute values of an element. With no attribute names, returns
/// every attribute present on the element. Pass specific names to select.
///
///   invoke element get com.app.id '[{"role": "menuBar"}]'
///   invoke element get com.app.id '[{"role": "menuBar"}]' children title
///   invoke element perform ... | invoke element get title value
#[derive(Args)]
pub struct GetOpts {
	/// app bundle ID, JSON query path, and optional attribute names
	args: Vec<String>,
}

/// Set an attribute value on an element.
///
///   invoke element set com.app.id '[{"role": "textField"}]' value "hello"
///   invoke element perform ... | invoke element set value "hello"
#[derive(Args)]
pub struct SetOpts {
	/// app bundle ID, JSON query path, attribute name, and value
	args: Vec<String>,
}

/// Perform an accessibility action on an element (press, increment, etc).
///
///   invoke element perform com.app.id '[{"role": "button", "title": "Play"}]' press
///   invoke element set ... | invoke element perform increment
#[derive(Args)]
pub struct PerformOpts {
	/// app bundle ID, JSON query path, and action name
	args: Vec<String>,
}

/// List available actions on an element.
///
///   invoke element actions com.app.id '[{"role": "button", "title": "Play"}]'
///   invoke element perform ... | invoke element actions
#[derive(Args)]
pub struct ActionsOpts {
	/// app bundle ID and JSON query path
	args: Vec<String>,
}

pub fn run(opts: Opts) -> Result {
	pipe::write_json_line(&exec(opts.command)?)
}

fn exec(command: Command) -> Result<Json> {
	match command {
		Command::Walk(o) => {
			let el = walk(&o.app, &o.query)?;
			Ok(walk_tree(&el, o.depth, o.full))
		}
		Command::Get(o) => get(o.args),
		Command::Set(o) => {
			let (app, query, rest) = resolve(o.args, 2)?;
			let attr: Attribute = rest[0].parse().err_code("UnknownAttribute")?;
			walk(&app, &query)?.set_string_attribute(&attr, &rest[1]).err_code("SetAttribute")?;
			Ok(descriptor(app, query))
		}
		Command::Perform(o) => {
			let (app, query, rest) = resolve(o.args, 1)?;
			let action: Action = rest[0].parse().unwrap();
			let el = walk(&app, &query)?;
			let actions = el.available_actions().err_code("AvailableActions")?;
			if !actions.contains(&action) {
				let requested = action.to_CFString().to_string();
				let available = serde_json::to_string(&actions).unwrap();
				return Err(Error::new("ActionUnavailable", format!("requested {requested}; available: {available}")));
			}
			el.perform_action(&action).err_code("PerformAction")?;
			Ok(descriptor(app, query))
		}
		Command::Actions(o) => {
			let (app, query, _) = resolve(o.args, 0)?;
			let actions = walk(&app, &query)?.available_actions().err_code("AvailableActions")?;
			Ok(serde_json::to_value(actions).unwrap())
		}
	}
}

/// Get attributes of the element resolved by `args` (app, query, optional attr names).
/// Used by both `element get` and `app get`.
pub fn get(args: Vec<String>) -> Result<Json> {
	let (app, query, extra) = resolve(args, 0)?;
	let el = walk(&app, &query)?;

	// If the user asked for specific attrs, keep nulls so they know the attr exists but has no value.
	// If listing all attrs, strip nulls to keep output clean for exploration.
	let explicit = !extra.is_empty();
	let attrs: Vec<Attribute> = if explicit {
		extra.iter().map(|name| name.parse().unwrap()).collect()
	} else {
		el.available_attributes()
			.err_code("AvailableAttributes")?
			.into_iter()
			.map(|name| name.parse().unwrap())
			.collect()
	};

	let obj: serde_json::Map<String, Json> = el
		.attributes(&attrs)
		.into_iter()
		.filter_map(|(attr, val)| {
			let key = match serde_json::to_value(&attr).unwrap() {
				Json::String(s) => s,
				_ => attr.to_CFString().to_string(), // Literal() attrs — pass through raw
			};
			let json_val = val
				.map(|v| normalize_value(&attr, v.to_json(&|el| walk_tree(el, 0, false))))
				.unwrap_or(Json::Null);
			if !explicit && json_val.is_null() {
				return None;
			}
			Some((key, json_val))
		})
		.collect();
	Ok(Json::Object(obj))
}

fn descriptor(app: String, query: String) -> Json {
	serde_json::to_value(ElementDescriptor { app, query }).unwrap()
}

/// Resolve app + query + extra args. If stdin is a pipe, app/query come from
/// the piped descriptor and all positional args are "extra". Otherwise the first
/// two positional args are app/query and the rest are extra.
fn resolve(args: Vec<String>, min_extra: usize) -> Result<(String, String, Vec<String>)> {
	if let Some(desc) = pipe::read_json_line::<ElementDescriptor>()? {
		if args.len() < min_extra {
			return Err(Error::code("MissingArgs"));
		}
		return Ok((desc.app, desc.query, args));
	}
	if args.len() < 2 + min_extra {
		return Err(Error::code("MissingArgs"));
	}
	let mut args = args;
	let app = args.remove(0);
	let query = args.remove(0);
	Ok((app, query, args))
}

const DEFAULT_ATTRS: &[Attribute] = &[
	Attribute::Identifier,
	Attribute::Title,
	Attribute::Description,
	Attribute::Role,
	Attribute::Subrole,
	Attribute::Value,
	Attribute::LabelValue,
];

/// Snapshot an element's basic attrs, recursing into children up to `depth` levels.
/// At depth 0, just snapshots the element itself (used by `element get` for inline
/// element-typed values like parent, closeButton, etc.).
/// Unless `full`, attributes whose value is empty/zero/default are omitted to cut noise.
/// When recursion stops at a node that still has children, reports `children: <count>`
/// so callers don't mistake it for a leaf.
fn walk_tree(el: &Element, depth: usize, full: bool) -> Json {
	let mut obj = serde_json::Map::new();

	for (attr, val) in el.attributes(DEFAULT_ATTRS) {
		let Some(val) = val else { continue };
		let json_val = normalize_value(&attr, val.to_json(&|el| walk_tree(el, 0, full)));
		if !full && is_empty_value(&json_val) {
			continue;
		}
		let key = match serde_json::to_value(&attr).unwrap() {
			Json::String(s) => s,
			_ => attr.to_CFString().to_string(),
		};
		obj.insert(key, json_val);
	}

	let children = el.children().unwrap_or_default();
	if depth > 0 {
		let children: Vec<Json> = children
			.iter()
			.map(|child| walk_tree(&Element::new(child), depth - 1, full))
			.collect();
		if !children.is_empty() {
			obj.insert("children".into(), Json::Array(children));
		}
	} else if !children.is_empty() {
		// Recursion stopped here; report child count so this isn't mistaken for a leaf.
		obj.insert("children".into(), Json::from(children.len()));
	}

	Json::Object(obj)
}

/// A value that carries no information — empty string, zero, false, empty array/object, null.
/// `walk_tree` skips these unless `--full`, since AX trees are full of unset defaults.
fn is_empty_value(v: &Json) -> bool {
	match v {
		Json::Null => true,
		Json::String(s) => s.is_empty(),
		Json::Number(n) => n.as_f64() == Some(0.0),
		Json::Array(a) => a.is_empty(),
		Json::Object(o) => o.is_empty(),
		_ => false,
	}
}

/// For attributes whose AX values are known constant enums, parse the raw AX string
/// through the enum so serde produces camelCase output instead of raw "AX..." strings.
fn normalize_value(attr: &Attribute, json_val: Json) -> Json {
	let Json::String(ref s) = json_val else { return json_val };
	match attr {
		Attribute::Role => serde_json::to_value(&s.parse::<Role>().unwrap()).unwrap(),
		Attribute::Subrole => serde_json::to_value(&s.parse::<Subrole>().unwrap()).unwrap(),
		Attribute::Orientation => serde_json::to_value(&s.parse::<Orientation>().unwrap()).unwrap(),
		Attribute::SortDirection => serde_json::to_value(&s.parse::<SortDirection>().unwrap()).unwrap(),
		_ => json_val,
	}
}

fn walk(app: &str, query_json: &str) -> Result<Element> {
	let pid = common::process::pid_for_bundle(app).err_code("NoRunningApp")?;
	let el = Element::new_application(pid).err_code("Application")?;
	let path = parse_filter_path(query_json)?;
	if path.is_empty() {
		return Ok(el);
	}
	el.walk(&path).err_code("Walk")?.err_code("NoElement")
}

/// `[{"role": "window", "title": "Foo"}]` → `[[{"role": {"literal":"AXWindow"}}, {"title": {"glob":"Foo"}}]]`
/// Serde deserializes enums as single-key objects, so `{"role": "...", "title": "..."}`
/// can't deserialize as one step directly. We split each key into its own object first.
/// Plain string values for role/subrole are normalized through the enum and wrapped as literal;
/// all other string values are wrapped as glob.
fn parse_filter_path(json: &str) -> Result<FilterPath> {
	let steps: Vec<serde_json::Map<String, Json>> = serde_json::from_str(json).err_code("BadQuery")?;
	steps
		.into_iter()
		.map(|obj| {
			obj.into_iter()
				.map(|(k, v)| {
					let v = transform_value(&k, v)?;
					serde_json::from_value::<Filter>(serde_json::json!({ k: v })).err_code("BadFilter")
				})
				.collect()
		})
		.collect()
}

/// Mirrors `Element.transformStep` from preload.ts: strings become globs
/// (with role/subrole normalization), objects that aren't already {glob}/{literal}
/// are recursively converted into arrays of filter pairs.
fn transform_value(k: &str, v: Json) -> Result<Json> {
	Ok(match v {
		Json::String(s) => match k {
			"role" => serde_json::json!({ "literal": s.parse::<Role>().unwrap().to_CFString().to_string() }),
			"subrole" => serde_json::json!({ "literal": s.parse::<Subrole>().unwrap().to_CFString().to_string() }),
			_ => serde_json::json!({ "glob": s }),
		},
		Json::Object(ref map) if !map.contains_key("glob") && !map.contains_key("literal") => {
			let arr: Vec<Json> = map
				.iter()
				.map(|(ik, iv)| Ok(serde_json::json!({ ik.clone(): transform_value(ik, iv.clone())? })))
				.collect::<Result<_>>()?;
			Json::Array(arr)
		}
		v => v,
	})
}

#[derive(Serialize, Deserialize)]
pub struct ElementDescriptor {
	#[serde(rename = "a", alias = "app")]
	pub app: String,
	#[serde(rename = "q", alias = "query")]
	pub query: String,
}
