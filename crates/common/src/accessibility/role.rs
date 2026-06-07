use objc2_core_foundation::{CFRetained, CFString};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Role {
	/// A role referenced by name, for roles not in the predefined list.
	Literal(String),

	Application,
	SystemWide,
	Window,
	Sheet,
	Drawer,
	GrowArea,
	Image,
	Unknown,
	Button,
	RadioButton,
	CheckBox,
	PopUpButton,
	MenuButton,
	TabGroup,
	Table,
	Column,
	Row,
	Outline,
	Browser,
	ScrollArea,
	ScrollBar,
	RadioGroup,
	List,
	Group,
	ValueIndicator,
	ComboBox,
	Slider,
	Incrementor,
	BusyIndicator,
	ProgressIndicator,
	RelevanceIndicator,
	Toolbar,
	DisclosureTriangle,
	TextField,
	TextArea,
	StaticText,
	Heading,
	MenuBar,
	MenuBarItem,
	Menu,
	MenuItem,
	SplitGroup,
	Splitter,
	ColorWell,
	TimeField,
	DateField,
	HelpTag,
	Matte,
	DockItem,
	Ruler,
	RulerMarker,
	Grid,
	LevelIndicator,
	Cell,
	LayoutArea,
	LayoutItem,
	Handle,
	Popover,

	// web
	ImageMap,
}

impl std::str::FromStr for Role {
	type Err = std::convert::Infallible;

	// TODO: Add a literal/escape syntax (see Attribute::FromStr) for the same
	// name-collision reason.
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"Application" | "application" | "AXApplication" => Role::Application,
			"SystemWide" | "systemWide" | "AXSystemWide" => Role::SystemWide,
			"Window" | "window" | "AXWindow" => Role::Window,
			"Sheet" | "sheet" | "AXSheet" => Role::Sheet,
			"Drawer" | "drawer" | "AXDrawer" => Role::Drawer,
			"GrowArea" | "growArea" | "AXGrowArea" => Role::GrowArea,
			"Image" | "image" | "AXImage" => Role::Image,
			"Unknown" | "unknown" | "AXUnknown" => Role::Unknown,
			"Button" | "button" | "AXButton" => Role::Button,
			"RadioButton" | "radioButton" | "AXRadioButton" => Role::RadioButton,
			"CheckBox" | "checkBox" | "AXCheckBox" => Role::CheckBox,
			"PopUpButton" | "popUpButton" | "AXPopUpButton" => Role::PopUpButton,
			"MenuButton" | "menuButton" | "AXMenuButton" => Role::MenuButton,
			"TabGroup" | "tabGroup" | "AXTabGroup" => Role::TabGroup,
			"Table" | "table" | "AXTable" => Role::Table,
			"Column" | "column" | "AXColumn" => Role::Column,
			"Row" | "row" | "AXRow" => Role::Row,
			"Outline" | "outline" | "AXOutline" => Role::Outline,
			"Browser" | "browser" | "AXBrowser" => Role::Browser,
			"ScrollArea" | "scrollArea" | "AXScrollArea" => Role::ScrollArea,
			"ScrollBar" | "scrollBar" | "AXScrollBar" => Role::ScrollBar,
			"RadioGroup" | "radioGroup" | "AXRadioGroup" => Role::RadioGroup,
			"List" | "list" | "AXList" => Role::List,
			"Group" | "group" | "AXGroup" => Role::Group,
			"ValueIndicator" | "valueIndicator" | "AXValueIndicator" => Role::ValueIndicator,
			"ComboBox" | "comboBox" | "AXComboBox" => Role::ComboBox,
			"Slider" | "slider" | "AXSlider" => Role::Slider,
			"Incrementor" | "incrementor" | "AXIncrementor" => Role::Incrementor,
			"BusyIndicator" | "busyIndicator" | "AXBusyIndicator" => Role::BusyIndicator,
			"ProgressIndicator" | "progressIndicator" | "AXProgressIndicator" => Role::ProgressIndicator,
			"RelevanceIndicator" | "relevanceIndicator" | "AXRelevanceIndicator" => Role::RelevanceIndicator,
			"Toolbar" | "toolbar" | "AXToolbar" => Role::Toolbar,
			"DisclosureTriangle" | "disclosureTriangle" | "AXDisclosureTriangle" => Role::DisclosureTriangle,
			"TextField" | "textField" | "AXTextField" => Role::TextField,
			"TextArea" | "textArea" | "AXTextArea" => Role::TextArea,
			"StaticText" | "staticText" | "AXStaticText" => Role::StaticText,
			"Heading" | "heading" | "AXHeading" => Role::Heading,
			"MenuBar" | "menuBar" | "AXMenuBar" => Role::MenuBar,
			"MenuBarItem" | "menuBarItem" | "AXMenuBarItem" => Role::MenuBarItem,
			"Menu" | "menu" | "AXMenu" => Role::Menu,
			"MenuItem" | "menuItem" | "AXMenuItem" => Role::MenuItem,
			"SplitGroup" | "splitGroup" | "AXSplitGroup" => Role::SplitGroup,
			"Splitter" | "splitter" | "AXSplitter" => Role::Splitter,
			"ColorWell" | "colorWell" | "AXColorWell" => Role::ColorWell,
			"TimeField" | "timeField" | "AXTimeField" => Role::TimeField,
			"DateField" | "dateField" | "AXDateField" => Role::DateField,
			"HelpTag" | "helpTag" | "AXHelpTag" => Role::HelpTag,
			"Matte" | "matte" | "AXMatte" => Role::Matte,
			"DockItem" | "dockItem" | "AXDockItem" => Role::DockItem,
			"Ruler" | "ruler" | "AXRuler" => Role::Ruler,
			"RulerMarker" | "rulerMarker" | "AXRulerMarker" => Role::RulerMarker,
			"Grid" | "grid" | "AXGrid" => Role::Grid,
			"LevelIndicator" | "levelIndicator" | "AXLevelIndicator" => Role::LevelIndicator,
			"Cell" | "cell" | "AXCell" => Role::Cell,
			"LayoutArea" | "layoutArea" | "AXLayoutArea" => Role::LayoutArea,
			"LayoutItem" | "layoutItem" | "AXLayoutItem" => Role::LayoutItem,
			"Handle" | "handle" | "AXHandle" => Role::Handle,
			"Popover" | "popover" | "AXPopover" => Role::Popover,
			"ImageMap" | "imageMap" | "AXImageMap" => Role::ImageMap,
			_ => Role::Literal(s.to_string()),
		})
	}
}

impl Role {
	#[allow(non_snake_case)]
	pub fn to_CFString(&self) -> CFRetained<CFString> {
		self.into()
	}
}

impl From<&Role> for CFRetained<CFString> {
	fn from(role: &Role) -> CFRetained<CFString> {
		if let Role::Literal(name) = role {
			return CFString::from_str(name);
		}

		CFString::from_static_str(match role {
			Role::Literal(_) => unreachable!(),
			Role::Application => "AXApplication",
			Role::SystemWide => "AXSystemWide",
			Role::Window => "AXWindow",
			Role::Sheet => "AXSheet",
			Role::Drawer => "AXDrawer",
			Role::GrowArea => "AXGrowArea",
			Role::Image => "AXImage",
			Role::Unknown => "AXUnknown",
			Role::Button => "AXButton",
			Role::RadioButton => "AXRadioButton",
			Role::CheckBox => "AXCheckBox",
			Role::PopUpButton => "AXPopUpButton",
			Role::MenuButton => "AXMenuButton",
			Role::TabGroup => "AXTabGroup",
			Role::Table => "AXTable",
			Role::Column => "AXColumn",
			Role::Row => "AXRow",
			Role::Outline => "AXOutline",
			Role::Browser => "AXBrowser",
			Role::ScrollArea => "AXScrollArea",
			Role::ScrollBar => "AXScrollBar",
			Role::RadioGroup => "AXRadioGroup",
			Role::List => "AXList",
			Role::Group => "AXGroup",
			Role::ValueIndicator => "AXValueIndicator",
			Role::ComboBox => "AXComboBox",
			Role::Slider => "AXSlider",
			Role::Incrementor => "AXIncrementor",
			Role::BusyIndicator => "AXBusyIndicator",
			Role::ProgressIndicator => "AXProgressIndicator",
			Role::RelevanceIndicator => "AXRelevanceIndicator",
			Role::Toolbar => "AXToolbar",
			Role::DisclosureTriangle => "AXDisclosureTriangle",
			Role::TextField => "AXTextField",
			Role::TextArea => "AXTextArea",
			Role::StaticText => "AXStaticText",
			Role::Heading => "AXHeading",
			Role::MenuBar => "AXMenuBar",
			Role::MenuBarItem => "AXMenuBarItem",
			Role::Menu => "AXMenu",
			Role::MenuItem => "AXMenuItem",
			Role::SplitGroup => "AXSplitGroup",
			Role::Splitter => "AXSplitter",
			Role::ColorWell => "AXColorWell",
			Role::TimeField => "AXTimeField",
			Role::DateField => "AXDateField",
			Role::HelpTag => "AXHelpTag",
			Role::Matte => "AXMatte",
			Role::DockItem => "AXDockItem",
			Role::Ruler => "AXRuler",
			Role::RulerMarker => "AXRulerMarker",
			Role::Grid => "AXGrid",
			Role::LevelIndicator => "AXLevelIndicator",
			Role::Cell => "AXCell",
			Role::LayoutArea => "AXLayoutArea",
			Role::LayoutItem => "AXLayoutItem",
			Role::Handle => "AXHandle",
			Role::Popover => "AXPopover",
			Role::ImageMap => "AXImageMap",
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_all_forms() {
		assert_eq!("button".parse::<Role>().unwrap(), Role::Button);
		assert_eq!("Button".parse::<Role>().unwrap(), Role::Button);
		assert_eq!("AXButton".parse::<Role>().unwrap(), Role::Button);

		assert_eq!("tabGroup".parse::<Role>().unwrap(), Role::TabGroup);
		assert_eq!("TabGroup".parse::<Role>().unwrap(), Role::TabGroup);
		assert_eq!("AXTabGroup".parse::<Role>().unwrap(), Role::TabGroup);
	}

	#[test]
	fn unknown_becomes_named() {
		assert_eq!("AXCustomRole".parse::<Role>().unwrap(), Role::Literal("AXCustomRole".into()));
		assert_eq!("myRole".parse::<Role>().unwrap(), Role::Literal("myRole".into()));
	}
}
