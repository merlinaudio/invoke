//! A module for converting Core Foundation types to native Rust types.
//!
//! The whole `Native` thing is a hangover from older code. Feel free to replace. Do not use `Native` directly anymore.

#![allow(deprecated)]

use std::{
	collections::HashMap,
	ops::{Deref, DerefMut},
	sync::LazyLock,
};

use objc2_application_services::AXUIElement;
use objc2_core_foundation::{CFArray, CFBoolean, CFDictionary, CFGetTypeID, CFNumber, CFRetained, CFString, CFType, CFTypeID, ConcreteType, Type};

#[cfg(feature = "serde")]
use serde_json::Value as JsonValue;
use thiserror::Error;

#[cfg(feature = "accessibility")]
use crate::accessibility::element::Element;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Type mismatch")]
	TypeMismatch,
}

/// A "bridge" type to quickly convert between `CFType`s and native Rust types like `String`, `bool`, ... (and other related types like `serde_json::Value`)
///
/// For example:
///
/// ```ignore
/// let some_cf_string = CFString::from_str("Hello, world!");
///
/// let string = Native::<String>::try_from(&some_cf_string); // <-- A perfectly normal String!
/// ```
#[deprecated(note = "Use `Value` instead")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Native<T>(pub T);

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	String(String),
	Bool(bool),
	Number(f64),
	Integer(i32),
	Float(f32),
	Array(Vec<Value>),
	Dictionary(HashMap<String, Value>),

	#[cfg(feature = "accessibility")]
	Element(Element),
}

#[cfg(feature = "serde")]
impl Value {
	/// Convert to JSON. `on_element` lets callers decide how to serialize Element values —
	/// the host retains them as handles (`{"#e": id}`), the CLI snapshots their attributes.
	pub fn to_json(&self, on_element: &impl Fn(&Element) -> serde_json::Value) -> serde_json::Value {
		match self {
			Value::String(s) => serde_json::Value::String(s.clone()),
			Value::Bool(b) => serde_json::Value::Bool(*b),
			Value::Number(n) => serde_json::json!(*n),
			Value::Integer(i) => serde_json::json!(*i),
			Value::Float(f) => serde_json::json!(*f),
			Value::Array(a) => serde_json::Value::Array(a.iter().map(|v| v.to_json(on_element)).collect()),
			Value::Dictionary(d) => serde_json::Value::Object(d.iter().map(|(k, v)| (k.clone(), v.to_json(on_element))).collect()),
			#[cfg(feature = "accessibility")]
			Value::Element(el) => on_element(el),
		}
	}
}

#[cfg(feature = "serde")]
impl From<&Value> for serde_json::Value {
	fn from(v: &Value) -> Self {
		v.to_json(&|_| serde_json::Value::Null)
	}
}

static STRING_TYPEID: LazyLock<CFTypeID> = LazyLock::new(CFString::type_id);
static BOOL_TYPEID: LazyLock<CFTypeID> = LazyLock::new(CFBoolean::type_id);
static NUMBER_TYPEID: LazyLock<CFTypeID> = LazyLock::new(CFNumber::type_id);
static INTEGER_TYPEID: LazyLock<CFTypeID> = LazyLock::new(CFNumber::type_id);
static FLOAT_TYPEID: LazyLock<CFTypeID> = LazyLock::new(CFNumber::type_id);
static ARRAY_TYPEID: LazyLock<CFTypeID> = LazyLock::new(CFArray::type_id);
static DICTIONARY_TYPEID: LazyLock<CFTypeID> = LazyLock::new(CFDictionary::type_id);
static AXUIELEMENT_TYPEID: LazyLock<CFTypeID> = LazyLock::new(AXUIElement::type_id);

impl TryFrom<&CFType> for Value {
	type Error = Error;

	/// Basically the same as CFType::downcast_ref and then mapping to `Value`.
	fn try_from(value: &CFType) -> Result<Self, Self::Error> {
		// Get type name/typeid of CFType:
		let type_id = CFGetTypeID(Some(value));

		Ok(match type_id {
			n if n == *STRING_TYPEID => Value::String(Native::try_from(value)?.0),
			n if n == *BOOL_TYPEID => Value::Bool(Native::try_from(value)?.0),
			n if n == *NUMBER_TYPEID => Value::Number(Native::try_from(value)?.0),
			n if n == *INTEGER_TYPEID => Value::Integer(Native::try_from(value)?.0),
			n if n == *FLOAT_TYPEID => Value::Float(Native::try_from(value)?.0),
			n if n == *ARRAY_TYPEID => {
				let array: &CFArray = value.downcast_ref().ok_or(Error::TypeMismatch)?;
				let len = array.len();
				let mut items = Vec::with_capacity(len);
				for i in 0..len {
					// get() retains the element, safe for immutable arrays.
					// items are untyped (Opaque) so we go through get_unchecked → CFType pointer cast.
					let item_ptr = unsafe { array.as_opaque().value_at_index(i as _) };
					if let Some(cf) = unsafe { item_ptr.cast::<CFType>().as_ref() }
						&& let Ok(v) = Value::try_from(cf)
					{
						items.push(v);
					}
				}
				Value::Array(items)
			}
			n if n == *DICTIONARY_TYPEID => Value::Dictionary(cfdictionary_to_hashmap(value.downcast_ref().ok_or(Error::TypeMismatch)?)),
			n if n == *AXUIELEMENT_TYPEID => {
				// Cast value to AXUIElement
				let ptr: *const CFType = value;
				let ptr: *const AXUIElement = ptr.cast();

				// as_ref_unchecked is safe because `value` is &CFType, so it's guaranteed not to be null
				let ui_element: &AXUIElement = unsafe { ptr.as_ref_unchecked() };

				Value::Element(Element::new(ui_element))
			}

			_ => return Err(Error::TypeMismatch),
		})
	}
}

impl PartialEq<String> for &Value {
	fn eq(&self, other: &String) -> bool {
		match self {
			&Value::String(value) => value == other,
			_ => false,
		}
	}
}

impl PartialEq<bool> for &Value {
	fn eq(&self, other: &bool) -> bool {
		match self {
			&Value::Bool(value) => value == other,
			_ => false,
		}
	}
}

impl PartialEq<f64> for &Value {
	fn eq(&self, other: &f64) -> bool {
		match self {
			&Value::Number(value) => value == other,
			_ => false,
		}
	}
}

impl PartialEq<i32> for &Value {
	fn eq(&self, other: &i32) -> bool {
		match self {
			&Value::Integer(value) => value == other,
			_ => false,
		}
	}
}

impl PartialEq<f32> for &Value {
	fn eq(&self, other: &f32) -> bool {
		match self {
			&Value::Float(value) => value == other,
			_ => false,
		}
	}
}

// ---------------------------------------------------------------------------------------------------------------------

impl PartialEq<String> for Value {
	fn eq(&self, other: &String) -> bool {
		match self {
			Value::String(value) => value == other,
			_ => false,
		}
	}
}

impl PartialEq<bool> for Value {
	fn eq(&self, other: &bool) -> bool {
		match self {
			Value::Bool(value) => value == other,
			_ => false,
		}
	}
}

impl PartialEq<f64> for Value {
	fn eq(&self, other: &f64) -> bool {
		match self {
			Value::Number(value) => value == other,
			_ => false,
		}
	}
}

impl PartialEq<i32> for Value {
	fn eq(&self, other: &i32) -> bool {
		match self {
			Value::Integer(value) => value == other,
			_ => false,
		}
	}
}

impl PartialEq<f32> for Value {
	fn eq(&self, other: &f32) -> bool {
		match self {
			Value::Float(value) => value == other,
			_ => false,
		}
	}
}

// ---------------------------------------------------------------------------------------------------------------------

impl TryFrom<Value> for String {
	type Error = Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::String(value) => Ok(value),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl TryFrom<Value> for bool {
	type Error = Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Bool(value) => Ok(value),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl TryFrom<Value> for f64 {
	type Error = Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Number(value) => Ok(value),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl TryFrom<Value> for i32 {
	type Error = Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Integer(value) => Ok(value),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl TryFrom<Value> for f32 {
	type Error = Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Float(value) => Ok(value),
			_ => Err(Error::TypeMismatch),
		}
	}
}

// ---------------------------------------------------------------------------------------------------------------------

impl<'a> TryFrom<&'a Value> for &'a str {
	type Error = Error;

	fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
		match value {
			Value::String(value) => Ok(value.as_str()),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl<'a> TryFrom<&'a Value> for &'a bool {
	type Error = Error;

	fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
		match value {
			Value::Bool(value) => Ok(value),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl<'a> TryFrom<&'a Value> for &'a f64 {
	type Error = Error;

	fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
		match value {
			Value::Number(value) => Ok(value),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl<'a> TryFrom<&'a Value> for &'a i32 {
	type Error = Error;

	fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
		match value {
			Value::Integer(value) => Ok(value),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl<'a> TryFrom<&'a Value> for &'a f32 {
	type Error = Error;

	fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
		match value {
			Value::Float(value) => Ok(value),
			_ => Err(Error::TypeMismatch),
		}
	}
}

// ---------------------------------------------------------------------------------------------------------------------

impl TryFrom<Value> for Native<String> {
	type Error = Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::String(value) => Ok(Native(value)),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl TryFrom<Value> for Native<bool> {
	type Error = Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Bool(value) => Ok(Native(value)),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl TryFrom<Value> for Native<f64> {
	type Error = Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Number(value) => Ok(Native(value)),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl TryFrom<Value> for Native<i32> {
	type Error = Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Integer(value) => Ok(Native(value)),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl TryFrom<Value> for Native<f32> {
	type Error = Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::Float(value) => Ok(Native(value)),
			_ => Err(Error::TypeMismatch),
		}
	}
}

// ---------------------------------------------------------------------------------------------------------------------

impl<'a> TryFrom<&'a Value> for Native<&'a str> {
	type Error = Error;

	fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
		match value {
			Value::String(value) => Ok(Native(value.as_str())),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl<'a> TryFrom<&'a Value> for Native<&'a bool> {
	type Error = Error;

	fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
		match value {
			Value::Bool(value) => Ok(Native(value)),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl<'a> TryFrom<&'a Value> for Native<&'a f64> {
	type Error = Error;

	fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
		match value {
			Value::Number(value) => Ok(Native(value)),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl<'a> TryFrom<&'a Value> for Native<&'a i32> {
	type Error = Error;

	fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
		match value {
			Value::Integer(value) => Ok(Native(value)),
			_ => Err(Error::TypeMismatch),
		}
	}
}

impl<'a> TryFrom<&'a Value> for Native<&'a f32> {
	type Error = Error;

	fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
		match value {
			Value::Float(value) => Ok(Native(value)),
			_ => Err(Error::TypeMismatch),
		}
	}
}

// ---------------------------------------------------------------------------------------------------------------------

impl<T> Deref for Native<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> DerefMut for Native<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<T> AsRef<T> for Native<T> {
	fn as_ref(&self) -> &T {
		&self.0
	}
}

impl AsRef<str> for Native<&str> {
	fn as_ref(&self) -> &str {
		self.0
	}
}

// ---------------------------------------------------------------------------------------------------------------------

impl TryFrom<&CFType> for Native<String> {
	type Error = Error;

	fn try_from(value: &CFType) -> Result<Self, Self::Error> {
		let Some(value) = value.downcast_ref::<CFString>() else {
			return Err(Error::TypeMismatch);
		};

		Ok(Self(value.to_string()))
	}
}

impl TryFrom<&CFType> for Native<bool> {
	type Error = Error;

	fn try_from(value: &CFType) -> Result<Self, Self::Error> {
		let Some(value) = value.downcast_ref::<CFBoolean>() else {
			return Err(Error::TypeMismatch);
		};

		Ok(Self(value.as_bool()))
	}
}

impl TryFrom<&CFType> for Native<f64> {
	type Error = Error;

	fn try_from(value: &CFType) -> Result<Self, Self::Error> {
		let Some(value) = value.downcast_ref::<CFNumber>() else {
			return Err(Error::TypeMismatch);
		};

		Ok(Self(value.as_f64().ok_or(Error::TypeMismatch)?))
	}
}

impl TryFrom<&CFNumber> for Native<f64> {
	type Error = Error;

	fn try_from(value: &CFNumber) -> Result<Self, Self::Error> {
		let value = try {
			let value = value.downcast_ref::<CFNumber>()?;
			value.as_f64()?
		};

		let Some(value) = value else {
			return Err(Error::TypeMismatch);
		};

		Ok(Self(value))
	}
}

impl TryFrom<&CFType> for Native<i64> {
	type Error = Error;

	fn try_from(value: &CFType) -> Result<Self, Self::Error> {
		let Some(value) = value.downcast_ref::<CFNumber>() else {
			return Err(Error::TypeMismatch);
		};

		let value = try { value.as_i64()? };

		let Some(value) = value else {
			return Err(Error::TypeMismatch);
		};

		Ok(Self(value))
	}
}

impl TryFrom<&CFType> for Native<f32> {
	type Error = Error;

	fn try_from(value: &CFType) -> Result<Self, Self::Error> {
		let Some(value) = value.downcast_ref::<CFNumber>() else {
			return Err(Error::TypeMismatch);
		};

		let value = try { value.as_f32()? };

		let Some(value) = value else {
			return Err(Error::TypeMismatch);
		};

		Ok(Self(value))
	}
}

impl TryFrom<&CFType> for Native<i32> {
	type Error = Error;

	fn try_from(value: &CFType) -> Result<Self, Self::Error> {
		let Some(value) = value.downcast_ref::<CFNumber>() else {
			return Err(Error::TypeMismatch);
		};

		let value = try { value.as_i32()? };

		let Some(value) = value else {
			return Err(Error::TypeMismatch);
		};

		Ok(Self(value))
	}
}

// ---------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "serde")]
impl From<Native<String>> for JsonValue {
	fn from(native: Native<String>) -> Self {
		JsonValue::String(native.0)
	}
}

#[cfg(feature = "serde")]
impl From<Native<bool>> for JsonValue {
	fn from(native: Native<bool>) -> Self {
		JsonValue::Bool(native.0)
	}
}

#[cfg(feature = "serde")]
impl From<Native<f64>> for JsonValue {
	fn from(native: Native<f64>) -> Self {
		JsonValue::Number(serde_json::Number::from_f64(native.0).unwrap_or(serde_json::Number::from(0)))
	}
}

#[cfg(feature = "serde")]
impl From<Native<i64>> for JsonValue {
	fn from(native: Native<i64>) -> Self {
		JsonValue::Number(serde_json::Number::from(native.0))
	}
}

#[cfg(feature = "serde")]
impl From<Native<f32>> for JsonValue {
	fn from(native: Native<f32>) -> Self {
		JsonValue::Number(serde_json::Number::from_f64(native.0 as f64).unwrap_or(serde_json::Number::from(0)))
	}
}

#[cfg(feature = "serde")]
impl From<Native<i32>> for JsonValue {
	fn from(native: Native<i32>) -> Self {
		JsonValue::Number(serde_json::Number::from(native.0))
	}
}

// ---------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "serde")]
pub fn cftype_to_json_value(cftype: &CFType) -> Option<JsonValue> {
	if let Ok(native) = Native::<String>::try_from(cftype) {
		return Some(JsonValue::from(native));
	}

	if let Ok(native) = Native::<bool>::try_from(cftype) {
		return Some(JsonValue::from(native));
	}

	if let Ok(native) = Native::<i64>::try_from(cftype) {
		return Some(JsonValue::from(native));
	}

	if let Ok(native) = Native::<i32>::try_from(cftype) {
		return Some(JsonValue::from(native));
	}

	if let Ok(native) = Native::<f64>::try_from(cftype) {
		return Some(JsonValue::from(native));
	}

	if let Ok(native) = Native::<f32>::try_from(cftype) {
		return Some(JsonValue::from(native));
	}

	None
}

// ---------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "serde")]
pub fn cfdictionary_to_hashmap(cfdictionary: &CFDictionary) -> HashMap<String, Value> {
	let cfdictionary = unsafe { CFRetained::cast_unchecked::<CFDictionary<CFType, CFType>>(cfdictionary.retain()) };
	let (keys, values) = cfdictionary.to_vecs();

	let mut hash_map = HashMap::with_capacity(cfdictionary.len());
	for (i, key) in keys.iter().enumerate() {
		try {
			let key = Native::try_from(key.as_ref())
				.inspect_err(|e| log::warn!("Unable to convert CFDictionary key {key:?} to string, skipping key. Error: {e:?}"))
				.ok()?
				.0;

			let value = values.get(i).expect("keys and values should have the same number of items").as_ref();
			let value = Value::try_from(value)
				.inspect_err(|e| log::warn!("Unable to convert CFDictionary value {value:?} to value, skipping key {key:?}. Error: {e:?}"))
				.ok()?;

			hash_map.insert(key, value);
		};
	}

	hash_map
}

#[cfg(test)]
mod tests {
	use std::{f32::consts::PI as PI_F32, f64::consts::PI as PI_F64};

	use super::*;
	use objc2_core_foundation::{CFBoolean, CFNumber, CFString, CFType};

	fn as_cftype<T: Type>(v: &T) -> &CFType {
		// every CF type can be viewed as CFType
		let ptr: *const T = v;
		unsafe { &*ptr.cast::<CFType>() }
	}

	#[test]
	fn string() {
		let cf = CFString::from_str("hello");
		let val = Value::try_from(as_cftype(&*cf)).unwrap();
		assert_eq!(val, Value::String("hello".into()));
	}

	#[test]
	fn bool_true() {
		let val = Value::try_from(as_cftype(CFBoolean::new(true))).unwrap();
		assert_eq!(val, Value::Bool(true));
	}

	#[test]
	fn bool_false() {
		let val = Value::try_from(as_cftype(CFBoolean::new(false))).unwrap();
		assert_eq!(val, Value::Bool(false));
	}

	#[test]
	fn number_f64() {
		let cf = CFNumber::new_f64(PI_F64);
		let val = Value::try_from(as_cftype(&*cf)).unwrap();
		assert_eq!(val, Value::Number(PI_F64));
	}

	#[test]
	fn number_i32() {
		let cf = CFNumber::new_i32(42);
		let val = Value::try_from(as_cftype(&*cf)).unwrap();
		// CFNumber type id is the same for all numeric types, so it'll
		// hit the first matching arm (Number/f64). that's fine — the
		// value is losslessly representable.
		assert_eq!(val, Value::Number(42.0));
	}

	#[test]
	fn number_f32() {
		let cf = CFNumber::new_f32(PI_F32);
		let val = Value::try_from(as_cftype(&*cf)).unwrap();
		assert_eq!(val, Value::Number(PI_F32 as f64));
	}

	#[test]
	fn array_of_every_type() {
		let s = CFString::from_str("item");
		let n = CFNumber::new_f64(99.0);
		let b = CFBoolean::new(true);

		let arr = CFArray::<CFType>::from_objects(&[s.as_ref(), n.as_ref(), b.as_ref()]);

		let val = Value::try_from(as_cftype(&*arr)).unwrap();
		assert_eq!(val, Value::Array(vec![Value::String("item".into()), Value::Number(99.0), Value::Bool(true),]));
	}

	#[test]
	fn dictionary_of_every_type() {
		let k1 = CFString::from_str("str_key");
		let v1 = CFString::from_str("str_val");
		let k2 = CFString::from_str("num_key");
		let v2 = CFNumber::new_f64(7.0);
		let k3 = CFString::from_str("bool_key");
		let v3 = CFBoolean::new(false);

		let dict = CFDictionary::<CFType, CFType>::from_slices(&[k1.as_ref(), k2.as_ref(), k3.as_ref()], &[v1.as_ref(), v2.as_ref(), v3.as_ref()]);

		let val = Value::try_from(as_cftype(&*dict)).unwrap();
		match val {
			Value::Dictionary(map) => {
				assert_eq!(map.get("str_key"), Some(&Value::String("str_val".into())));
				assert_eq!(map.get("num_key"), Some(&Value::Number(7.0)));
				assert_eq!(map.get("bool_key"), Some(&Value::Bool(false)));
				assert_eq!(map.len(), 3);
			}
			other => panic!("expected Dictionary, got {other:?}"),
		}
	}
}
