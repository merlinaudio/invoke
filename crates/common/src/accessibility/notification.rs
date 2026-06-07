use objc2_core_foundation::{CFRetained, CFString};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(try_from = "String", into = "String"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Notification {
	/// A notification referenced by name, for notifications not in the predefined list.
	Literal(String),

	MainWindowChanged,
	FocusedWindowChanged,
	FocusedUIElementChanged,
	ApplicationActivated,
	ApplicationDeactivated,
	ApplicationHidden,
	ApplicationShown,
	WindowCreated,
	WindowMoved,
	WindowResized,
	WindowMiniaturized,
	WindowDeminiaturized,
	DrawerCreated,
	SheetCreated,
	HelpTagCreated,
	ValueChanged,
	UIElementDestroyed,
	ElementBusyChanged,
	MenuOpened,
	MenuClosed,
	MenuItemSelected,
	RowCountChanged,
	RowExpanded,
	RowCollapsed,
	SelectedCellsChanged,
	UnitsChanged,
	SelectedChildrenMoved,
	SelectedChildrenChanged,
	Resized,
	Moved,
	Created,
	SelectedRowsChanged,
	SelectedColumnsChanged,
	SelectedTextChanged,
	TitleChanged,
	LayoutChanged,
	AnnouncementRequested,

	// web
	ActiveElementChanged,
	CurrentStateChanged,
	ExpandedChanged,
	InvalidStatusChanged,
	LayoutComplete,
	LiveRegionChanged,
	LiveRegionCreated,
	LoadComplete,
}

impl std::str::FromStr for Notification {
	type Err = std::convert::Infallible;

	// TODO: Add a literal/escape syntax (see Attribute::FromStr) for the same
	// name-collision reason.
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"MainWindowChanged" | "mainWindowChanged" | "AXMainWindowChanged" => Notification::MainWindowChanged,
			"FocusedWindowChanged" | "focusedWindowChanged" | "AXFocusedWindowChanged" => Notification::FocusedWindowChanged,
			"FocusedUIElementChanged" | "focusedUIElementChanged" | "AXFocusedUIElementChanged" => Notification::FocusedUIElementChanged,
			"ApplicationActivated" | "applicationActivated" | "AXApplicationActivated" => Notification::ApplicationActivated,
			"ApplicationDeactivated" | "applicationDeactivated" | "AXApplicationDeactivated" => Notification::ApplicationDeactivated,
			"ApplicationHidden" | "applicationHidden" | "AXApplicationHidden" => Notification::ApplicationHidden,
			"ApplicationShown" | "applicationShown" | "AXApplicationShown" => Notification::ApplicationShown,
			"WindowCreated" | "windowCreated" | "AXWindowCreated" => Notification::WindowCreated,
			"WindowMoved" | "windowMoved" | "AXWindowMoved" => Notification::WindowMoved,
			"WindowResized" | "windowResized" | "AXWindowResized" => Notification::WindowResized,
			"WindowMiniaturized" | "windowMiniaturized" | "AXWindowMiniaturized" => Notification::WindowMiniaturized,
			"WindowDeminiaturized" | "windowDeminiaturized" | "AXWindowDeminiaturized" => Notification::WindowDeminiaturized,
			"DrawerCreated" | "drawerCreated" | "AXDrawerCreated" => Notification::DrawerCreated,
			"SheetCreated" | "sheetCreated" | "AXSheetCreated" => Notification::SheetCreated,
			"HelpTagCreated" | "helpTagCreated" | "AXHelpTagCreated" => Notification::HelpTagCreated,
			"ValueChanged" | "valueChanged" | "AXValueChanged" => Notification::ValueChanged,
			"UIElementDestroyed" | "uIElementDestroyed" | "uiElementDestroyed" | "AXUIElementDestroyed" => Notification::UIElementDestroyed,
			"ElementBusyChanged" | "elementBusyChanged" | "AXElementBusyChanged" => Notification::ElementBusyChanged,
			"MenuOpened" | "menuOpened" | "AXMenuOpened" => Notification::MenuOpened,
			"MenuClosed" | "menuClosed" | "AXMenuClosed" => Notification::MenuClosed,
			"MenuItemSelected" | "menuItemSelected" | "AXMenuItemSelected" => Notification::MenuItemSelected,
			"RowCountChanged" | "rowCountChanged" | "AXRowCountChanged" => Notification::RowCountChanged,
			"RowExpanded" | "rowExpanded" | "AXRowExpanded" => Notification::RowExpanded,
			"RowCollapsed" | "rowCollapsed" | "AXRowCollapsed" => Notification::RowCollapsed,
			"SelectedCellsChanged" | "selectedCellsChanged" | "AXSelectedCellsChanged" => Notification::SelectedCellsChanged,
			"UnitsChanged" | "unitsChanged" | "AXUnitsChanged" => Notification::UnitsChanged,
			"SelectedChildrenMoved" | "selectedChildrenMoved" | "AXSelectedChildrenMoved" => Notification::SelectedChildrenMoved,
			"SelectedChildrenChanged" | "selectedChildrenChanged" | "AXSelectedChildrenChanged" => Notification::SelectedChildrenChanged,
			"Resized" | "resized" | "AXResized" => Notification::Resized,
			"Moved" | "moved" | "AXMoved" => Notification::Moved,
			"Created" | "created" | "AXCreated" => Notification::Created,
			"SelectedRowsChanged" | "selectedRowsChanged" | "AXSelectedRowsChanged" => Notification::SelectedRowsChanged,
			"SelectedColumnsChanged" | "selectedColumnsChanged" | "AXSelectedColumnsChanged" => Notification::SelectedColumnsChanged,
			"SelectedTextChanged" | "selectedTextChanged" | "AXSelectedTextChanged" => Notification::SelectedTextChanged,
			"TitleChanged" | "titleChanged" | "AXTitleChanged" => Notification::TitleChanged,
			"LayoutChanged" | "layoutChanged" | "AXLayoutChanged" => Notification::LayoutChanged,
			"AnnouncementRequested" | "announcementRequested" | "AXAnnouncementRequested" => Notification::AnnouncementRequested,
			"ActiveElementChanged" | "activeElementChanged" | "AXActiveElementChanged" => Notification::ActiveElementChanged,
			"CurrentStateChanged" | "currentStateChanged" | "AXCurrentStateChanged" => Notification::CurrentStateChanged,
			"ExpandedChanged" | "expandedChanged" | "AXExpandedChanged" => Notification::ExpandedChanged,
			"InvalidStatusChanged" | "invalidStatusChanged" | "AXInvalidStatusChanged" => Notification::InvalidStatusChanged,
			"LayoutComplete" | "layoutComplete" | "AXLayoutComplete" => Notification::LayoutComplete,
			"LiveRegionChanged" | "liveRegionChanged" | "AXLiveRegionChanged" => Notification::LiveRegionChanged,
			"LiveRegionCreated" | "liveRegionCreated" | "AXLiveRegionCreated" => Notification::LiveRegionCreated,
			"LoadComplete" | "loadComplete" | "AXLoadComplete" => Notification::LoadComplete,
			_ => Notification::Literal(s.to_string()),
		})
	}
}

impl From<String> for Notification {
	fn from(value: String) -> Self {
		value.parse().unwrap()
	}
}

impl From<Notification> for String {
	fn from(value: Notification) -> Self {
		value.to_CFString().to_string()
	}
}

impl Notification {
	#[allow(non_snake_case)]
	pub fn to_CFString(&self) -> CFRetained<CFString> {
		self.into()
	}
}

impl From<&Notification> for CFRetained<CFString> {
	fn from(notif: &Notification) -> CFRetained<CFString> {
		if let Notification::Literal(name) = notif {
			return CFString::from_str(name);
		}

		CFString::from_static_str(match notif {
			Notification::Literal(_) => unreachable!(),
			Notification::MainWindowChanged => "AXMainWindowChanged",
			Notification::FocusedWindowChanged => "AXFocusedWindowChanged",
			Notification::FocusedUIElementChanged => "AXFocusedUIElementChanged",
			Notification::ApplicationActivated => "AXApplicationActivated",
			Notification::ApplicationDeactivated => "AXApplicationDeactivated",
			Notification::ApplicationHidden => "AXApplicationHidden",
			Notification::ApplicationShown => "AXApplicationShown",
			Notification::WindowCreated => "AXWindowCreated",
			Notification::WindowMoved => "AXWindowMoved",
			Notification::WindowResized => "AXWindowResized",
			Notification::WindowMiniaturized => "AXWindowMiniaturized",
			Notification::WindowDeminiaturized => "AXWindowDeminiaturized",
			Notification::DrawerCreated => "AXDrawerCreated",
			Notification::SheetCreated => "AXSheetCreated",
			Notification::HelpTagCreated => "AXHelpTagCreated",
			Notification::ValueChanged => "AXValueChanged",
			Notification::UIElementDestroyed => "AXUIElementDestroyed",
			Notification::ElementBusyChanged => "AXElementBusyChanged",
			Notification::MenuOpened => "AXMenuOpened",
			Notification::MenuClosed => "AXMenuClosed",
			Notification::MenuItemSelected => "AXMenuItemSelected",
			Notification::RowCountChanged => "AXRowCountChanged",
			Notification::RowExpanded => "AXRowExpanded",
			Notification::RowCollapsed => "AXRowCollapsed",
			Notification::SelectedCellsChanged => "AXSelectedCellsChanged",
			Notification::UnitsChanged => "AXUnitsChanged",
			Notification::SelectedChildrenMoved => "AXSelectedChildrenMoved",
			Notification::SelectedChildrenChanged => "AXSelectedChildrenChanged",
			Notification::Resized => "AXResized",
			Notification::Moved => "AXMoved",
			Notification::Created => "AXCreated",
			Notification::SelectedRowsChanged => "AXSelectedRowsChanged",
			Notification::SelectedColumnsChanged => "AXSelectedColumnsChanged",
			Notification::SelectedTextChanged => "AXSelectedTextChanged",
			Notification::TitleChanged => "AXTitleChanged",
			Notification::LayoutChanged => "AXLayoutChanged",
			Notification::AnnouncementRequested => "AXAnnouncementRequested",
			Notification::ActiveElementChanged => "AXActiveElementChanged",
			Notification::CurrentStateChanged => "AXCurrentStateChanged",
			Notification::ExpandedChanged => "AXExpandedChanged",
			Notification::InvalidStatusChanged => "AXInvalidStatusChanged",
			Notification::LayoutComplete => "AXLayoutComplete",
			Notification::LiveRegionChanged => "AXLiveRegionChanged",
			Notification::LiveRegionCreated => "AXLiveRegionCreated",
			Notification::LoadComplete => "AXLoadComplete",
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_all_forms() {
		assert_eq!("valueChanged".parse::<Notification>().unwrap(), Notification::ValueChanged);
		assert_eq!("ValueChanged".parse::<Notification>().unwrap(), Notification::ValueChanged);
		assert_eq!("AXValueChanged".parse::<Notification>().unwrap(), Notification::ValueChanged);

		assert_eq!(
			"focusedUIElementChanged".parse::<Notification>().unwrap(),
			Notification::FocusedUIElementChanged
		);
		assert_eq!(
			"FocusedUIElementChanged".parse::<Notification>().unwrap(),
			Notification::FocusedUIElementChanged
		);
		assert_eq!(
			"AXFocusedUIElementChanged".parse::<Notification>().unwrap(),
			Notification::FocusedUIElementChanged
		);
	}

	#[test]
	fn unknown_becomes_named() {
		assert_eq!(
			"AXCustomNotification".parse::<Notification>().unwrap(),
			Notification::Literal("AXCustomNotification".into())
		);
		assert_eq!("myNotif".parse::<Notification>().unwrap(), Notification::Literal("myNotif".into()));
	}
}
