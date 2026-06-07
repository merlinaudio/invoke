use objc2_application_services::{AXCopyMultipleAttributeOptions, AXError, AXUIElement};
use objc2_core_foundation::{CFArray, CFEqual, CFRetained, CFString, CFType, Type};
use std::ptr::{NonNull, null};
use thiserror::Error;

use crate::accessibility::{
	self, Value,
	action::Action,
	attribute::Attribute,
	element_handle::{ElementHandle, retain_element},
	filter::FilterStep,
	orientation::Orientation,
	role::Role,
	sort_direction::SortDirection,
	subrole::Subrole,
};

trait ToResult {
	fn to_result(self) -> Result<(), Error>;
}

impl ToResult for AXError {
	fn to_result(self) -> Result<(), Error> {
		if self != AXError::Success { Err(Error::AX(self.0)) } else { Ok(()) }
	}
}

#[derive(Debug, Error)]
pub enum Error {
	#[error("AXError: {0}")]
	AX(i32),

	#[error("OS failed to copy data to pointer, or we couldn't read copied data from pointer")]
	PointerError,

	#[error("Invalid process ID, e.g. negative")]
	ProcessIdInvalid,

	#[error("Process ID valid but no associated process found")]
	ProcessNotFound,

	#[error("Attribute not found")]
	AttributeNotFound,

	#[error("Type conversion error: {0}")]
	TypeConversion(#[from] accessibility::value::Error),

	#[error("Element handle did not have an inner element")]
	ElementHandleEmpty,
}

/// Represents an element in the macOS Accessibility API.
#[derive(Debug, Clone)]
pub struct Element {
	ui_element: CFRetained<AXUIElement>,

	/// Cache of attributes that have been fetched before.
	attribute_cache: Vec<(Attribute, Value)>,
}

impl From<CFRetained<AXUIElement>> for Element {
	fn from(ui_element: CFRetained<AXUIElement>) -> Self {
		Self {
			ui_element,
			attribute_cache: Default::default(),
		}
	}
}

impl<'a> From<&'a Element> for &'a AXUIElement {
	fn from(element: &'a Element) -> Self {
		&element.ui_element
	}
}

impl TryFrom<ElementHandle> for Element {
	type Error = Error;

	fn try_from(element: ElementHandle) -> Result<Self, Self::Error> {
		Ok(Self {
			ui_element: element.inner().ok_or(Error::ElementHandleEmpty)?,
			attribute_cache: Default::default(),
		})
	}
}

impl AsRef<AXUIElement> for Element {
	fn as_ref(&self) -> &AXUIElement {
		&self.ui_element
	}
}

impl PartialEq for Element {
	fn eq(&self, other: &Self) -> bool {
		self.ui_element == other.ui_element
	}
}

impl Drop for Element {
	fn drop(&mut self) {
		log::trace!("Dropping {:?}", self.ui_element);
	}
}

impl Element {
	pub fn new(underlying_axuielement: &AXUIElement) -> Self {
		Self {
			ui_element: underlying_axuielement.retain(),
			attribute_cache: Default::default(),
		}
	}

	/// Checks if two elements are the same underlying AXUIElement.
	pub fn is_same_element(&self, other: &Element) -> bool {
		CFEqual(Some(&self.ui_element), Some(&other.ui_element))
	}

	pub fn retain(&self) -> ElementHandle {
		retain_element(&self.ui_element)
	}

	pub fn new_application(pid: u32) -> Result<Self, Error> {
		let Ok(pid) = pid.try_into() else {
			return Err(Error::ProcessIdInvalid);
		};

		let element = unsafe { AXUIElement::new_application(pid) };

		Ok(Self {
			ui_element: element.retain(),
			attribute_cache: Default::default(),
		})
	}

	pub fn new_systemwide() -> Result<Self, Error> {
		let element = unsafe { AXUIElement::new_system_wide() };
		Ok(Self {
			ui_element: element.retain(),
			attribute_cache: Default::default(),
		})
	}

	/// Finds an Element in an AXUIElement tree.
	///
	/// ## Explanation
	///
	/// To be more specific, this function recursively steps into an element's children based on a list of filter sets.
	///
	/// Let's break it down with some pseudocode:
	///
	/// ```ignore
	/// // We want to find the "Browser" pane in Ableton Live.
	/// let ableton_pid = 12345;
	///
	/// let ableton = Element::new_application(ableton_pid)?;
	///
	/// let browser_axgroup: Option<Element> = ableton.walk(vec![
	///     // Go into            window                       not a dialog
	///     HashSet(Filter::Role("AXWindow"), Filter::Subrole("AXStandardWindow")),
	///
	///     // then into          a "group" element         titled "browser"   that contains at least one Input element
	///     HashSet(Filter::Role("AXGroup"), Filter::Title("Browser"), Filter::AnyChildrenMatch(HashSet(Filter::Role("AXInput")))
	/// ])?;
	/// ```
	///
	/// What's returned by `ableton.walk()` in the example above is the last element in the filter path (if there was any).
	///
	/// ## Filters are subtractive (`AND`)
	///
	/// Filters are treated as a "list of requirements".
	///
	/// The first child that matches all filters is chosen. That means if there's an Input titled "FooBarBaz" **without an Identifier**, but you specify this:
	///
	/// ```ignore
	/// vec![HashSet(Filter::Role("AXInput"), Filter::Title("FooBarBaz"), Filter::Identifier("MainWindow.Browser.Search"))]
	/// ```
	///
	/// ...it will be **skipped** because of the Identifier filter. The element did not have an Identifier, but you specified one.
	///
	/// ## This function matches very eagerly
	///
	/// Keep in mind that the **first** element that matches all filters is chosen in each step.
	///
	/// If there are three windows, for example, and you just specify
	///
	/// ```ignore
	/// vec![HashSet(Filter::Role("AXWindow")), HashSet(Filter::Role("AXTextField"), Filter::LabelValue("Track Name"))]
	/// ```
	///
	/// the first window will always be chosen, even if it doesn't have a TextField labeled "Track Name".\
	/// (Returning `None` in that case.)
	pub fn walk(&self, path: &[FilterStep]) -> Result<Option<Element>, Error> {
		// Subtractive (`AND`) list of filters for the current step.
		// Recursive calls (see below) only happen if there are filters left.
		// However, packs may hand us an empty path directly, in which case the semantically
		// correct thing is to return `None`.
		let Some((current_step_filters, rest)) = path.split_first() else {
			return Ok(None);
		};

		for child in self.children()? {
			let mut child = Element::new(&child);

			if current_step_filters.iter().all(|filter| filter.matches(&mut child).unwrap_or(false)) {
				// This child matches all the filters!

				if rest.is_empty() {
					// No filters left! This is the element we were looking for.
					return Ok(Some(child));
				} else {
					// Step into the element and continue walking
					return child.walk(rest);
				}
			}
		}

		// No child matched the filters
		log::debug!("Walk got until element={self:#?} filters={current_step_filters:#?}");

		Ok(None)
	}

	pub fn perform_action(&self, action: &Action) -> Result<(), Error> {
		unsafe { self.ui_element.perform_action(&action.to_CFString()) }.to_result()
	}

	pub fn available_actions(&self) -> Result<Vec<Action>, Error> {
		let names = copy_cfarray(&self.ui_element, |el, ptr| unsafe { AXUIElement::copy_action_names(el, ptr) })?;
		Ok(unsafe { names.iter_unchecked().map(|s: &CFString| s.to_string().parse().unwrap()).collect() })
	}

	pub fn set_string_attribute(&self, attribute: &Attribute, value: &str) -> Result<(), Error> {
		self.set_attribute(attribute, &CFString::from_str(value))
	}

	pub fn set_attribute(&self, attribute: &Attribute, value: &CFType) -> Result<(), Error> {
		unsafe { self.ui_element.set_attribute_value(&attribute.to_CFString(), value) }.to_result()
	}

	/// Gets a single attribute value. Returns a cached value if available.
	pub fn attribute(&mut self, attribute: Attribute) -> Result<&Value, Error> {
		if let Ok(i) = self.attribute_cache.binary_search_by(|(a, _)| a.cmp(&attribute)) {
			return Ok(&self.attribute_cache[i].1);
		}

		let mut raw = null::<CFType>();
		let out = NonNull::new(&mut raw).unwrap();
		unsafe { self.ui_element.copy_attribute_value(&attribute.to_CFString(), out) }.to_result()?;
		let value = Value::try_from(unsafe { raw.as_ref() }.ok_or(Error::PointerError)?)?;

		let insert_index = match self.attribute_cache.binary_search_by(|(a, _)| a.cmp(&attribute)) {
			Ok(i) | Err(i) => i,
		};
		self.attribute_cache.insert(insert_index, (attribute, value));
		Ok(&self.attribute_cache[insert_index].1)
	}

	/// Returns this element's role as a typed enum.
	pub fn role(&mut self) -> Result<Role, Error> {
		let s: &str = self.attribute(Attribute::Role)?.try_into()?;
		Ok(s.parse().unwrap())
	}

	/// Returns this element's subrole as a typed enum.
	pub fn subrole(&mut self) -> Result<Subrole, Error> {
		let s: &str = self.attribute(Attribute::Subrole)?.try_into()?;
		Ok(s.parse().unwrap())
	}

	/// Returns this element's orientation as a typed enum.
	pub fn orientation(&mut self) -> Result<Orientation, Error> {
		let s: &str = self.attribute(Attribute::Orientation)?.try_into()?;
		Ok(s.parse().unwrap())
	}

	/// Returns this element's sort direction as a typed enum.
	pub fn sort_direction(&mut self) -> Result<SortDirection, Error> {
		let s: &str = self.attribute(Attribute::SortDirection)?.try_into()?;
		Ok(s.parse().unwrap())
	}

	/// Returns the names of all attributes this element supports.
	pub fn available_attributes(&self) -> Result<Vec<String>, Error> {
		let names: CFRetained<CFArray<CFString>> = copy_cfarray(&self.ui_element, |el, ptr| unsafe { AXUIElement::copy_attribute_names(el, ptr) })?;
		Ok(unsafe { names.iter_unchecked().map(|s: &CFString| s.to_string()).collect() })
	}

	/// Batch-reads multiple attributes in one Mach IPC round-trip.
	/// Unsupported or empty attributes come back as `None` instead of erroring.
	pub fn attributes(&self, attrs: &[Attribute]) -> Vec<(Attribute, Option<Value>)> {
		let names = attrs.iter().map(Into::into).collect::<Vec<CFRetained<CFString>>>();
		let name_refs = names.iter().map(|s| s.as_ref()).collect::<Vec<&CFString>>();
		let names_array = CFArray::<CFString>::from_objects(&name_refs);

		let values = copy_cfarray::<CFType>(&self.ui_element, |el, out| unsafe {
			el.copy_multiple_attribute_values(names_array.as_ref(), AXCopyMultipleAttributeOptions(0), out)
		});
		let values = values.as_ref().map(|a| unsafe { a.to_vec_unchecked() }).unwrap_or_default();

		attrs
			.iter()
			.enumerate()
			.map(|(i, attr)| (attr.clone(), values.get(i).and_then(|cf| Value::try_from(*cf).ok())))
			.collect()
	}

	pub fn children(&self) -> Result<Vec<CFRetained<AXUIElement>>, Error> {
		log::trace!("Getting children: {:#?}", self.ui_element);

		let children: CFRetained<CFArray<AXUIElement>> = copy_cfarray(&self.ui_element, |el, out| unsafe {
			AXUIElement::copy_attribute_values(el, &Attribute::Children.to_CFString(), 0, 1024, out)
		})?;

		Ok(children.to_vec())
	}
}

/// Calls an AXUIElement function that fills a CFArray out-pointer, handling the pointer dance.
pub fn copy_cfarray<T: Type>(el: &AXUIElement, copy_fn: impl Fn(&AXUIElement, NonNull<*const CFArray>) -> AXError) -> Result<CFRetained<CFArray<T>>, Error> {
	let mut raw = null::<CFArray>();
	let out = NonNull::new(&mut raw).unwrap();
	copy_fn(el, out).to_result()?;
	Ok(unsafe { CFRetained::cast_unchecked::<CFArray<T>>(raw.as_ref().ok_or(Error::PointerError)?.retain()) })
}
