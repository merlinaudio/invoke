//! Tiny JSON-to-JSON templating.
//!
//! This crate intentionally has a very small language:
//! string values starting with `$` read from context, object keys starting with
//! `$` call registered functions, and `$$` escapes either form by chopping off
//! one leading `$`.

#![cfg_attr(test, feature(test))]

#[cfg(test)]
extern crate test;

use std::collections::HashMap;

use serde_json::{Map, Value};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("missing template variable ${0}")]
	MissingVariable(String),

	#[error("template variable name cannot be empty")]
	EmptyVariableName,

	#[error("template function name cannot be empty")]
	EmptyFunctionName,

	#[error("unknown template function ${0}")]
	UnknownFunction(String),

	#[error("template function object ${0} must not have sibling keys")]
	FunctionObjectWithSiblings(String),

	#[error("template function ${name} failed: {source}")]
	Function { name: String, source: Box<Error> },

	#[error("{0}")]
	Message(String),
}

pub type Result<T> = std::result::Result<T, Error>;

type Function = dyn Fn(&Value) -> Result<Value> + Send + Sync + 'static;

pub trait Context {
	fn get(&self, name: &str) -> Option<&Value>;
}

#[derive(Default)]
pub struct Templater {
	functions: HashMap<String, Box<Function>>,
}

impl Templater {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn function(mut self, name: impl Into<String>, function: impl Fn(&Value) -> Result<Value> + Send + Sync + 'static) -> Self {
		let name = name.into();
		assert!(!name.is_empty(), "template function name cannot be empty");
		assert!(!name.starts_with('$'), "template function name must not start with $");

		self.functions.insert(name, Box::new(function));
		self
	}

	pub fn render<C: Context + ?Sized>(&self, template: &Value, context: &C) -> Result<Value> {
		match template {
			Value::Null | Value::Bool(_) | Value::Number(_) => Ok(template.clone()),
			Value::String(string) => self.render_string(string, context),
			Value::Array(array) => array
				.iter()
				.map(|value| self.render(value, context))
				.collect::<Result<Vec<_>>>()
				.map(Value::Array),
			Value::Object(object) => self.render_object(object, context),
		}
	}

	fn render_string<C: Context + ?Sized>(&self, string: &str, context: &C) -> Result<Value> {
		match TemplateString::parse(string)? {
			TemplateString::Literal(value) => Ok(Value::String(value.to_owned())),
			TemplateString::Variable(name) => context.get(name).cloned().ok_or_else(|| Error::MissingVariable(name.to_owned())),
		}
	}

	fn render_object<C: Context + ?Sized>(&self, object: &Map<String, Value>, context: &C) -> Result<Value> {
		if let Some(call) = FunctionCall::parse(object)? {
			return self.render_function(call, context);
		}

		object
			.iter()
			.map(|(key, value)| {
				let key = ObjectKey::parse(key)?.rendered_key()?.to_owned();
				let value = self.render(value, context)?;

				Ok((key, value))
			})
			.collect::<Result<Map<_, _>>>()
			.map(Value::Object)
	}

	fn render_function<C: Context + ?Sized>(&self, call: FunctionCall<'_>, context: &C) -> Result<Value> {
		let function = self.functions.get(call.name).ok_or_else(|| Error::UnknownFunction(call.name.to_owned()))?;
		let args = self.render(call.args, context)?;

		function(&args).map_err(|source| Error::Function {
			name: call.name.to_owned(),
			source: Box::new(source),
		})
	}
}

impl Error {
	pub fn message(message: impl Into<String>) -> Self {
		Self::Message(message.into())
	}
}

impl Context for HashMap<String, Value> {
	fn get(&self, name: &str) -> Option<&Value> {
		HashMap::get(self, name)
	}
}

impl Context for Map<String, Value> {
	fn get(&self, name: &str) -> Option<&Value> {
		Map::get(self, name)
	}
}

impl Context for Value {
	fn get(&self, name: &str) -> Option<&Value> {
		Value::get(self, name)
	}
}

impl<T: Context + ?Sized> Context for &T {
	fn get(&self, name: &str) -> Option<&Value> {
		(*self).get(name)
	}
}

enum TemplateString<'a> {
	Literal(&'a str),
	Variable(&'a str),
}

impl<'a> TemplateString<'a> {
	fn parse(string: &'a str) -> Result<Self> {
		match DollarPrefix::parse(string) {
			DollarPrefix::Literal(value) | DollarPrefix::Escaped(value) => Ok(Self::Literal(value)),
			DollarPrefix::Reserved("") => Err(Error::EmptyVariableName),
			DollarPrefix::Reserved(name) => Ok(Self::Variable(name)),
		}
	}
}

enum ObjectKey<'a> {
	Literal(&'a str),
	Function(&'a str),
}

impl<'a> ObjectKey<'a> {
	fn parse(key: &'a str) -> Result<Self> {
		match DollarPrefix::parse(key) {
			DollarPrefix::Literal(value) | DollarPrefix::Escaped(value) => Ok(Self::Literal(value)),
			DollarPrefix::Reserved("") => Err(Error::EmptyFunctionName),
			DollarPrefix::Reserved(name) => Ok(Self::Function(name)),
		}
	}

	fn rendered_key(self) -> Result<&'a str> {
		match self {
			Self::Literal(key) => Ok(key),
			Self::Function(name) => Err(Error::FunctionObjectWithSiblings(name.to_owned())),
		}
	}
}

enum DollarPrefix<'a> {
	Literal(&'a str),
	Escaped(&'a str),
	Reserved(&'a str),
}

impl<'a> DollarPrefix<'a> {
	fn parse(value: &'a str) -> Self {
		let Some(rest) = value.strip_prefix('$') else {
			return Self::Literal(value);
		};

		if rest.starts_with('$') {
			Self::Escaped(rest)
		} else {
			Self::Reserved(rest)
		}
	}
}

struct FunctionCall<'a> {
	name: &'a str,
	args: &'a Value,
}

impl<'a> FunctionCall<'a> {
	fn parse(object: &'a Map<String, Value>) -> Result<Option<Self>> {
		for (key, args) in object {
			let ObjectKey::Function(name) = ObjectKey::parse(key)? else {
				continue;
			};

			if object.len() != 1 {
				return Err(Error::FunctionObjectWithSiblings(name.to_owned()));
			}

			return Ok(Some(Self { name, args }));
		}

		Ok(None)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;
	use test::{black_box, Bencher};

	fn templater() -> Templater {
		Templater::new()
	}

	fn context(value: Value) -> Map<String, Value> {
		value.as_object().unwrap().clone()
	}

	#[test]
	fn variable_string_renders_context_value() {
		let rendered = templater().render(&json!("$foo"), &json!({ "foo": 123 })).unwrap();
		assert_eq!(rendered, json!(123));
	}

	#[test]
	fn variable_string_renders_any_json_value() {
		let rendered = templater()
			.render(&json!({ "value": "$foo" }), &context(json!({ "foo": { "nested": ["yes", true] } })))
			.unwrap();

		assert_eq!(rendered, json!({ "value": { "nested": ["yes", true] } }));
	}

	#[test]
	fn nested_variables_render_recursively() {
		let rendered = templater().render(&json!({ "kek": "$foo" }), &context(json!({ "foo": 123 }))).unwrap();

		assert_eq!(rendered, json!({ "kek": 123 }));
	}

	#[test]
	fn dollars_escape_by_chopping_one_dollar() {
		let rendered = templater().render(&json!(["$$foo", "$$$foo", "$$$$foo", "$$$"]), &context(json!({}))).unwrap();

		assert_eq!(rendered, json!(["$foo", "$$foo", "$$$foo", "$$"]));
	}

	#[test]
	fn escaped_object_keys_chop_one_dollar() {
		let rendered = templater().render(&json!({ "$$schema": "x", "$$$schema": "y" }), &context(json!({}))).unwrap();

		assert_eq!(rendered, json!({ "$schema": "x", "$$schema": "y" }));
	}

	#[test]
	fn missing_variable_errors() {
		let error = templater().render(&json!("$foo"), &context(json!({}))).unwrap_err();
		assert!(matches!(error, Error::MissingVariable(name) if name == "foo"));
	}

	#[test]
	fn empty_variable_errors() {
		let error = templater().render(&json!("$"), &context(json!({}))).unwrap_err();
		assert!(matches!(error, Error::EmptyVariableName));
	}

	#[test]
	fn unknown_function_errors() {
		let error = templater().render(&json!({ "$nope": {} }), &context(json!({}))).unwrap_err();
		assert!(matches!(error, Error::UnknownFunction(name) if name == "nope"));
	}

	#[test]
	fn empty_function_name_errors() {
		let error = templater().render(&json!({ "$": {} }), &context(json!({}))).unwrap_err();
		assert!(matches!(error, Error::EmptyFunctionName));
	}

	#[test]
	#[should_panic(expected = "template function name cannot be empty")]
	fn function_names_cannot_be_empty() {
		let _ = templater().function("", |_| Ok(Value::Null));
	}

	#[test]
	#[should_panic(expected = "template function name must not start with $")]
	fn function_names_cannot_start_with_dollar() {
		let _ = templater().function("$map", |_| Ok(Value::Null));
	}

	#[test]
	fn function_objects_cannot_have_siblings() {
		let error = templater()
			.function("map", |_| Ok(Value::Null))
			.render(&json!({ "$map": {}, "kek": 1 }), &context(json!({})))
			.unwrap_err();

		assert!(matches!(error, Error::FunctionObjectWithSiblings(name) if name == "map"));
	}

	#[test]
	fn registered_function_receives_rendered_arguments() {
		let rendered = templater()
			.function("map", |args| {
				let input = args
					.get("input")
					.and_then(Value::as_f64)
					.ok_or_else(|| Error::message("$map.input must be a number"))?;
				let from = args
					.get("from")
					.and_then(Value::as_f64)
					.ok_or_else(|| Error::message("$map.from must be a number"))?;
				let to = args
					.get("to")
					.and_then(Value::as_f64)
					.ok_or_else(|| Error::message("$map.to must be a number"))?;

				Ok(json!(input.clamp(from.min(to), from.max(to))))
			})
			.render(
				&json!({ "kek": { "$map": { "input": "$foo", "from": 1, "to": 100 } } }),
				&context(json!({ "foo": 123 })),
			)
			.unwrap();

		assert_eq!(rendered, json!({ "kek": 100.0 }));
	}

	#[test]
	fn functions_compose_by_rendering_inner_functions_first() {
		let rendered = templater()
			.function("curve", |args| {
				let input = args
					.get("input")
					.and_then(Value::as_f64)
					.ok_or_else(|| Error::message("$curve.input must be a number"))?;

				Ok(json!({ "input": input * input }))
			})
			.function("map", |args| {
				let input = args
					.get("input")
					.and_then(Value::as_f64)
					.ok_or_else(|| Error::message("$map.input must be a number"))?;

				Ok(json!(input + 1.0))
			})
			.render(
				&json!({
					"$map": {
						"$curve": {
							"input": "$value"
						}
					}
				}),
				&context(json!({ "value": 3 })),
			)
			.unwrap();

		assert_eq!(rendered, json!(10.0));
	}

	#[test]
	fn function_errors_are_wrapped_with_function_name() {
		let error = templater()
			.function("bad", |_| Err(Error::message("nope")))
			.render(&json!({ "$bad": {} }), &context(json!({})))
			.unwrap_err();

		assert!(matches!(error, Error::Function { name, .. } if name == "bad"));
	}

	#[bench]
	fn deeply_nested_functions(bencher: &mut Bencher) {
		const DEPTH: usize = 1024;

		let mut templater = templater();
		let mut template = json!("$value");

		for index in 0..DEPTH {
			templater = templater.function(format!("f{index}"), |value| {
				let value = value.as_u64().ok_or_else(|| Error::message("expected an unsigned integer"))?;

				Ok(json!(value + 1))
			});
		}

		for index in 0..DEPTH {
			template = json!({ format!("$f{index}"): template });
		}

		let context = context(json!({ "value": 0 }));

		assert_eq!(templater.render(&template, &context).unwrap(), json!(DEPTH));

		bencher.iter(|| black_box(templater.render(black_box(&template), black_box(&context)).unwrap()));
	}
}
