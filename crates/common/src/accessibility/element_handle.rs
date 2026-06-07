use objc2_application_services::AXUIElement;
use objc2_core_foundation::{CFRetained, Type};
use std::{
	fmt::Display,
	sync::{
		LazyLock,
		atomic::{AtomicU32, Ordering},
	},
};

use crate::main_thread::MainThread;

type ElementRefMap = papaya::HashMap<ElementHandle, MainThread<CFRetained<AXUIElement>>>;

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

/// Accessibility Elements retained so the frontend can use them.
///
/// Needed because otherwise the references would become invalid, so the frontend would have
/// to re-walk the tree every time it needs to access an element.
pub static ELEMENTS: LazyLock<ElementRefMap> = LazyLock::new(papaya::HashMap::new);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", transparent)]
pub struct ElementHandle(pub(super) u32);

impl From<&ElementHandle> for u32 {
	fn from(value: &ElementHandle) -> Self {
		value.0
	}
}

impl From<ElementHandle> for u32 {
	fn from(value: ElementHandle) -> Self {
		value.0
	}
}
impl From<u32> for ElementHandle {
	fn from(value: u32) -> Self {
		ElementHandle(value)
	}
}

impl Display for ElementHandle {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "e#{}", self.0)
	}
}

pub fn retain_element(element: impl AsRef<AXUIElement>) -> ElementHandle {
	let element = MainThread::new(element.as_ref().retain());
	let element_handle = ElementHandle(NEXT_ID.fetch_add(1, Ordering::Relaxed));

	log::info!("Retained element {element_handle:?} / {} ({element:?})", ELEMENTS.len());

	ELEMENTS.pin().insert(element_handle, element);

	element_handle
}

pub fn update(element_handle: ElementHandle, element: impl AsRef<AXUIElement>) -> ElementHandle {
	let element = MainThread::new(element.as_ref().retain());

	ELEMENTS.pin().insert(element_handle, element);

	element_handle
}

pub fn dispose_element(element_handle: ElementHandle) -> Option<()> {
	log::info!("Disposing {element_handle} / {}", ELEMENTS.len());

	ELEMENTS.pin().remove(&element_handle).map(|_| ())
}

impl ElementHandle {
	pub fn dispose(self) -> Option<()> {
		dispose_element(self)
	}

	pub fn inner(&self) -> Option<CFRetained<AXUIElement>> {
		Some(ELEMENTS.pin().get(self)?.unwrap().clone())
	}
}
