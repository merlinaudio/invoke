use objc2_core_foundation::{CFRetained, CFString};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(rename_all = "camelCase", from = "String")
)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Attribute {
	/// An attribute referenced by name, for attributes not in the predefined list.
	Literal(String),

	Identifier,

	// informational attributes
	Role,
	Subrole,
	RoleDescription,
	Title,
	Description,
	Help,

	// hierarchy or relationship attributes
	Parent,
	Children,
	SelectedChildren,
	VisibleChildren,
	Window,
	TopLevelUIElement,
	TitleUIElement,
	ServesAsTitleForUIElements,
	LinkedUIElements,
	SharedFocusElements,

	// visual state attributes
	Enabled,
	Focused,
	Position,
	Size,

	// value attributes
	Value,
	ValueDescription,
	MinValue,
	MaxValue,
	ValueIncrement,
	ValueWraps,
	AllowedValues,
	PlaceholderValue,

	// text-specific attributes
	SelectedText,
	SelectedTextRange,
	SelectedTextRanges,
	VisibleCharacterRange,
	NumberOfCharacters,
	SharedTextUIElements,
	SharedCharacterRange,

	// window, sheet, or drawer-specific attributes
	Main,
	Minimized,
	CloseButton,
	ZoomButton,
	MinimizeButton,
	ToolbarButton,
	Proxy,
	GrowArea,
	Modal,
	DefaultButton,
	CancelButton,

	// menu or menu item-specific attributes
	MenuItemCmdChar,
	MenuItemCmdVirtualKey,
	MenuItemCmdGlyph,
	MenuItemCmdModifiers,
	MenuItemMarkChar,
	MenuItemPrimaryUIElement,

	// application element-specific attributes
	MenuBar,
	Windows,
	Frontmost,
	Hidden,
	MainWindow,
	FocusedWindow,
	FocusedUIElement,
	ExtrasMenuBar,

	// date/time-specific attributes
	HourField,
	MinuteField,
	SecondField,
	#[serde(rename = "ampmField")]
	AMPMField,
	DayField,
	MonthField,
	YearField,

	// table, outline, or browser-specific attributes
	Rows,
	VisibleRows,
	SelectedRows,
	Columns,
	VisibleColumns,
	SelectedColumns,
	SortDirection,
	ColumnHeaderUIElements,
	Index,
	Disclosing,
	DisclosedRows,
	DisclosedByRow,

	// matte-specific attributes
	MatteHole,
	MatteContentUIElement,

	// ruler-specific attributes
	MarkerUIElements,
	Units,
	UnitDescription,
	MarkerType,
	MarkerTypeDescription,

	// miscellaneous or role-specific attributes
	HorizontalScrollBar,
	VerticalScrollBar,
	Orientation,
	Header,
	Edited,
	Tabs,
	OverflowButton,
	Filename,
	Expanded,
	Selected,
	Splitters,
	Contents,
	NextContents,
	PreviousContents,
	Document,
	Incrementor,
	DecrementButton,
	IncrementButton,
	ColumnTitle,
	#[serde(rename = "url")]
	URL,
	LabelUIElements,
	LabelValue,
	ShownMenuUIElement,
	IsApplicationRunning,
	FocusedApplication,
	ElementBusy,
	AlternateUIVisible,
}

impl std::str::FromStr for Attribute {
	type Err = std::convert::Infallible;

	// TODO: Add a literal/escape syntax (e.g. `="Title"` or `raw:Title`) so users can
	// force Literal("Title") when an app creates a custom attribute whose name collides
	// with a built-in. Right now, "Title" always resolves to Attribute::Title (AXTitle),
	// so there's no way to refer to a custom attribute literally named "Title".
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"Identifier" | "identifier" | "AXIdentifier" => Attribute::Identifier,
			"Role" | "role" | "AXRole" => Attribute::Role,
			"Subrole" | "subrole" | "AXSubrole" => Attribute::Subrole,
			"RoleDescription" | "roleDescription" | "AXRoleDescription" => Attribute::RoleDescription,
			"Title" | "title" | "AXTitle" => Attribute::Title,
			"Description" | "description" | "AXDescription" => Attribute::Description,
			"Help" | "help" | "AXHelp" => Attribute::Help,
			"Parent" | "parent" | "AXParent" => Attribute::Parent,
			"Children" | "children" | "AXChildren" => Attribute::Children,
			"SelectedChildren" | "selectedChildren" | "AXSelectedChildren" => Attribute::SelectedChildren,
			"VisibleChildren" | "visibleChildren" | "AXVisibleChildren" => Attribute::VisibleChildren,
			"Window" | "window" | "AXWindow" => Attribute::Window,
			"TopLevelUIElement" | "topLevelUIElement" | "AXTopLevelUIElement" => Attribute::TopLevelUIElement,
			"TitleUIElement" | "titleUIElement" | "AXTitleUIElement" => Attribute::TitleUIElement,
			"ServesAsTitleForUIElements" | "servesAsTitleForUIElements" | "AXServesAsTitleForUIElements" => Attribute::ServesAsTitleForUIElements,
			"LinkedUIElements" | "linkedUIElements" | "AXLinkedUIElements" => Attribute::LinkedUIElements,
			"SharedFocusElements" | "sharedFocusElements" | "AXSharedFocusElements" => Attribute::SharedFocusElements,
			"Enabled" | "enabled" | "AXEnabled" => Attribute::Enabled,
			"Focused" | "focused" | "AXFocused" => Attribute::Focused,
			"Position" | "position" | "AXPosition" => Attribute::Position,
			"Size" | "size" | "AXSize" => Attribute::Size,
			"Value" | "value" | "AXValue" => Attribute::Value,
			"ValueDescription" | "valueDescription" | "AXValueDescription" => Attribute::ValueDescription,
			"MinValue" | "minValue" | "AXMinValue" => Attribute::MinValue,
			"MaxValue" | "maxValue" | "AXMaxValue" => Attribute::MaxValue,
			"ValueIncrement" | "valueIncrement" | "AXValueIncrement" => Attribute::ValueIncrement,
			"ValueWraps" | "valueWraps" | "AXValueWraps" => Attribute::ValueWraps,
			"AllowedValues" | "allowedValues" | "AXAllowedValues" => Attribute::AllowedValues,
			"PlaceholderValue" | "placeholderValue" | "AXPlaceholderValue" => Attribute::PlaceholderValue,
			"SelectedText" | "selectedText" | "AXSelectedText" => Attribute::SelectedText,
			"SelectedTextRange" | "selectedTextRange" | "AXSelectedTextRange" => Attribute::SelectedTextRange,
			"SelectedTextRanges" | "selectedTextRanges" | "AXSelectedTextRanges" => Attribute::SelectedTextRanges,
			"VisibleCharacterRange" | "visibleCharacterRange" | "AXVisibleCharacterRange" => Attribute::VisibleCharacterRange,
			"NumberOfCharacters" | "numberOfCharacters" | "AXNumberOfCharacters" => Attribute::NumberOfCharacters,
			"SharedTextUIElements" | "sharedTextUIElements" | "AXSharedTextUIElements" => Attribute::SharedTextUIElements,
			"SharedCharacterRange" | "sharedCharacterRange" | "AXSharedCharacterRange" => Attribute::SharedCharacterRange,
			"Main" | "main" | "AXMain" => Attribute::Main,
			"Minimized" | "minimized" | "AXMinimized" => Attribute::Minimized,
			"CloseButton" | "closeButton" | "AXCloseButton" => Attribute::CloseButton,
			"ZoomButton" | "zoomButton" | "AXZoomButton" => Attribute::ZoomButton,
			"MinimizeButton" | "minimizeButton" | "AXMinimizeButton" => Attribute::MinimizeButton,
			"ToolbarButton" | "toolbarButton" | "AXToolbarButton" => Attribute::ToolbarButton,
			"Proxy" | "proxy" | "AXProxy" => Attribute::Proxy,
			"GrowArea" | "growArea" | "AXGrowArea" => Attribute::GrowArea,
			"Modal" | "modal" | "AXModal" => Attribute::Modal,
			"DefaultButton" | "defaultButton" | "AXDefaultButton" => Attribute::DefaultButton,
			"CancelButton" | "cancelButton" | "AXCancelButton" => Attribute::CancelButton,
			"MenuItemCmdChar" | "menuItemCmdChar" | "AXMenuItemCmdChar" => Attribute::MenuItemCmdChar,
			"MenuItemCmdVirtualKey" | "menuItemCmdVirtualKey" | "AXMenuItemCmdVirtualKey" => Attribute::MenuItemCmdVirtualKey,
			"MenuItemCmdGlyph" | "menuItemCmdGlyph" | "AXMenuItemCmdGlyph" => Attribute::MenuItemCmdGlyph,
			"MenuItemCmdModifiers" | "menuItemCmdModifiers" | "AXMenuItemCmdModifiers" => Attribute::MenuItemCmdModifiers,
			"MenuItemMarkChar" | "menuItemMarkChar" | "AXMenuItemMarkChar" => Attribute::MenuItemMarkChar,
			"MenuItemPrimaryUIElement" | "menuItemPrimaryUIElement" | "AXMenuItemPrimaryUIElement" => Attribute::MenuItemPrimaryUIElement,
			"MenuBar" | "menuBar" | "AXMenuBar" => Attribute::MenuBar,
			"Windows" | "windows" | "AXWindows" => Attribute::Windows,
			"Frontmost" | "frontmost" | "AXFrontmost" => Attribute::Frontmost,
			"Hidden" | "hidden" | "AXHidden" => Attribute::Hidden,
			"MainWindow" | "mainWindow" | "AXMainWindow" => Attribute::MainWindow,
			"FocusedWindow" | "focusedWindow" | "AXFocusedWindow" => Attribute::FocusedWindow,
			"FocusedUIElement" | "focusedUIElement" | "AXFocusedUIElement" => Attribute::FocusedUIElement,
			"ExtrasMenuBar" | "extrasMenuBar" | "AXExtrasMenuBar" => Attribute::ExtrasMenuBar,
			"HourField" | "hourField" | "AXHourField" => Attribute::HourField,
			"MinuteField" | "minuteField" | "AXMinuteField" => Attribute::MinuteField,
			"SecondField" | "secondField" | "AXSecondField" => Attribute::SecondField,
			"AMPMField" | "aMPMField" | "AXAMPMField" => Attribute::AMPMField,
			"DayField" | "dayField" | "AXDayField" => Attribute::DayField,
			"MonthField" | "monthField" | "AXMonthField" => Attribute::MonthField,
			"YearField" | "yearField" | "AXYearField" => Attribute::YearField,
			"Rows" | "rows" | "AXRows" => Attribute::Rows,
			"VisibleRows" | "visibleRows" | "AXVisibleRows" => Attribute::VisibleRows,
			"SelectedRows" | "selectedRows" | "AXSelectedRows" => Attribute::SelectedRows,
			"Columns" | "columns" | "AXColumns" => Attribute::Columns,
			"VisibleColumns" | "visibleColumns" | "AXVisibleColumns" => Attribute::VisibleColumns,
			"SelectedColumns" | "selectedColumns" | "AXSelectedColumns" => Attribute::SelectedColumns,
			"SortDirection" | "sortDirection" | "AXSortDirection" => Attribute::SortDirection,
			"ColumnHeaderUIElements" | "columnHeaderUIElements" | "AXColumnHeaderUIElements" => Attribute::ColumnHeaderUIElements,
			"Index" | "index" | "AXIndex" => Attribute::Index,
			"Disclosing" | "disclosing" | "AXDisclosing" => Attribute::Disclosing,
			"DisclosedRows" | "disclosedRows" | "AXDisclosedRows" => Attribute::DisclosedRows,
			"DisclosedByRow" | "disclosedByRow" | "AXDisclosedByRow" => Attribute::DisclosedByRow,
			"MatteHole" | "matteHole" | "AXMatteHole" => Attribute::MatteHole,
			"MatteContentUIElement" | "matteContentUIElement" | "AXMatteContentUIElement" => Attribute::MatteContentUIElement,
			"MarkerUIElements" | "markerUIElements" | "AXMarkerUIElements" => Attribute::MarkerUIElements,
			"Units" | "units" | "AXUnits" => Attribute::Units,
			"UnitDescription" | "unitDescription" | "AXUnitDescription" => Attribute::UnitDescription,
			"MarkerType" | "markerType" | "AXMarkerType" => Attribute::MarkerType,
			"MarkerTypeDescription" | "markerTypeDescription" | "AXMarkerTypeDescription" => Attribute::MarkerTypeDescription,
			"HorizontalScrollBar" | "horizontalScrollBar" | "AXHorizontalScrollBar" => Attribute::HorizontalScrollBar,
			"VerticalScrollBar" | "verticalScrollBar" | "AXVerticalScrollBar" => Attribute::VerticalScrollBar,
			"Orientation" | "orientation" | "AXOrientation" => Attribute::Orientation,
			"Header" | "header" | "AXHeader" => Attribute::Header,
			"Edited" | "edited" | "AXEdited" => Attribute::Edited,
			"Tabs" | "tabs" | "AXTabs" => Attribute::Tabs,
			"OverflowButton" | "overflowButton" | "AXOverflowButton" => Attribute::OverflowButton,
			"Filename" | "filename" | "AXFilename" => Attribute::Filename,
			"Expanded" | "expanded" | "AXExpanded" => Attribute::Expanded,
			"Selected" | "selected" | "AXSelected" => Attribute::Selected,
			"Splitters" | "splitters" | "AXSplitters" => Attribute::Splitters,
			"Contents" | "contents" | "AXContents" => Attribute::Contents,
			"NextContents" | "nextContents" | "AXNextContents" => Attribute::NextContents,
			"PreviousContents" | "previousContents" | "AXPreviousContents" => Attribute::PreviousContents,
			"Document" | "document" | "AXDocument" => Attribute::Document,
			"Incrementor" | "incrementor" | "AXIncrementor" => Attribute::Incrementor,
			"DecrementButton" | "decrementButton" | "AXDecrementButton" => Attribute::DecrementButton,
			"IncrementButton" | "incrementButton" | "AXIncrementButton" => Attribute::IncrementButton,
			"ColumnTitle" | "columnTitle" | "AXColumnTitle" => Attribute::ColumnTitle,
			"URL" | "uRL" | "url" | "AXURL" => Attribute::URL,
			"LabelUIElements" | "labelUIElements" | "AXLabelUIElements" => Attribute::LabelUIElements,
			"LabelValue" | "labelValue" | "AXLabelValue" => Attribute::LabelValue,
			"ShownMenuUIElement" | "shownMenuUIElement" | "AXShownMenuUIElement" => Attribute::ShownMenuUIElement,
			"IsApplicationRunning" | "isApplicationRunning" | "AXIsApplicationRunning" => Attribute::IsApplicationRunning,
			"FocusedApplication" | "focusedApplication" | "AXFocusedApplication" => Attribute::FocusedApplication,
			"ElementBusy" | "elementBusy" | "AXElementBusy" => Attribute::ElementBusy,
			"AlternateUIVisible" | "alternateUIVisible" | "AXAlternateUIVisible" => Attribute::AlternateUIVisible,
			_ => Attribute::Literal(s.to_string()),
		})
	}
}

impl Attribute {
	#[allow(non_snake_case)]
	pub fn to_CFString(&self) -> CFRetained<CFString> {
		self.into()
	}
}

// impl Into<CFRetained<CFString>> for Attribute {
impl From<&Attribute> for CFRetained<CFString> {
	fn from(attr: &Attribute) -> CFRetained<CFString> {
		use Attribute::*;

		if let Literal(name) = attr {
			return CFString::from_str(name);
		}

		CFString::from_static_str(match attr {
			Literal(_) => unreachable!(),
			Identifier => "AXIdentifier",
			AlternateUIVisible => "AXAlternateUIVisible",
			ElementBusy => "AXElementBusy",
			FocusedApplication => "AXFocusedApplication",
			IsApplicationRunning => "AXIsApplicationRunning",
			ShownMenuUIElement => "AXShownMenuUIElement",
			LabelValue => "AXLabelValue",
			LabelUIElements => "AXLabelUIElements",
			URL => "AXURL",
			ColumnTitle => "AXColumnTitle",
			IncrementButton => "AXIncrementButton",
			DecrementButton => "AXDecrementButton",
			Incrementor => "AXIncrementor",
			Document => "AXDocument",
			PreviousContents => "AXPreviousContents",
			NextContents => "AXNextContents",
			Contents => "AXContents",
			Splitters => "AXSplitters",
			Selected => "AXSelected",
			Expanded => "AXExpanded",
			Filename => "AXFilename",
			OverflowButton => "AXOverflowButton",
			Tabs => "AXTabs",
			Edited => "AXEdited",
			Header => "AXHeader",
			Orientation => "AXOrientation",
			VerticalScrollBar => "AXVerticalScrollBar",
			HorizontalScrollBar => "AXHorizontalScrollBar",
			MarkerTypeDescription => "AXMarkerTypeDescription",
			MarkerType => "AXMarkerType",
			UnitDescription => "AXUnitDescription",
			Units => "AXUnits",
			MarkerUIElements => "AXMarkerUIElements",
			MatteContentUIElement => "AXMatteContentUIElement",
			MatteHole => "AXMatteHole",
			DisclosedByRow => "AXDisclosedByRow",
			DisclosedRows => "AXDisclosedRows",
			Disclosing => "AXDisclosing",
			Index => "AXIndex",
			ColumnHeaderUIElements => "AXColumnHeaderUIElements",
			SortDirection => "AXSortDirection",
			SelectedColumns => "AXSelectedColumns",
			VisibleColumns => "AXVisibleColumns",
			Columns => "AXColumns",
			SelectedRows => "AXSelectedRows",
			VisibleRows => "AXVisibleRows",
			Rows => "AXRows",
			YearField => "AXYearField",
			MonthField => "AXMonthField",
			DayField => "AXDayField",
			AMPMField => "AXAMPMField",
			SecondField => "AXSecondField",
			MinuteField => "AXMinuteField",
			HourField => "AXHourField",
			ExtrasMenuBar => "AXExtrasMenuBar",
			FocusedUIElement => "AXFocusedUIElement",
			FocusedWindow => "AXFocusedWindow",
			MainWindow => "AXMainWindow",
			Hidden => "AXHidden",
			Frontmost => "AXFrontmost",
			Windows => "AXWindows",
			MenuBar => "AXMenuBar",
			MenuItemPrimaryUIElement => "AXMenuItemPrimaryUIElement",
			MenuItemMarkChar => "AXMenuItemMarkChar",
			MenuItemCmdModifiers => "AXMenuItemCmdModifiers",
			MenuItemCmdGlyph => "AXMenuItemCmdGlyph",
			MenuItemCmdVirtualKey => "AXMenuItemCmdVirtualKey",
			MenuItemCmdChar => "AXMenuItemCmdChar",
			CancelButton => "AXCancelButton",
			DefaultButton => "AXDefaultButton",
			Modal => "AXModal",
			GrowArea => "AXGrowArea",
			Proxy => "AXProxy",
			ToolbarButton => "AXToolbarButton",
			MinimizeButton => "AXMinimizeButton",
			ZoomButton => "AXZoomButton",
			CloseButton => "AXCloseButton",
			Minimized => "AXMinimized",
			Main => "AXMain",
			SharedCharacterRange => "AXSharedCharacterRange",
			SharedTextUIElements => "AXSharedTextUIElements",
			NumberOfCharacters => "AXNumberOfCharacters",
			VisibleCharacterRange => "AXVisibleCharacterRange",
			SelectedTextRanges => "AXSelectedTextRanges",
			SelectedTextRange => "AXSelectedTextRange",
			SelectedText => "AXSelectedText",
			AllowedValues => "AXAllowedValues",
			ValueWraps => "AXValueWraps",
			ValueIncrement => "AXValueIncrement",
			MaxValue => "AXMaxValue",
			MinValue => "AXMinValue",
			ValueDescription => "AXValueDescription",
			Value => "AXValue",
			PlaceholderValue => "AXPlaceholderValue",
			Size => "AXSize",
			Position => "AXPosition",
			Focused => "AXFocused",
			Enabled => "AXEnabled",
			SharedFocusElements => "AXSharedFocusElements",
			LinkedUIElements => "AXLinkedUIElements",
			ServesAsTitleForUIElements => "AXServesAsTitleForUIElements",
			TitleUIElement => "AXTitleUIElement",
			TopLevelUIElement => "AXTopLevelUIElement",
			Window => "AXWindow",
			VisibleChildren => "AXVisibleChildren",
			SelectedChildren => "AXSelectedChildren",
			Children => "AXChildren",
			Parent => "AXParent",
			Help => "AXHelp",
			Description => "AXDescription",
			Title => "AXTitle",
			RoleDescription => "AXRoleDescription",
			Subrole => "AXSubrole",
			Role => "AXRole",
		})
	}
}

impl From<String> for Attribute {
	fn from(s: String) -> Self {
		s.parse().unwrap()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_all_forms() {
		// camelCase
		assert_eq!("title".parse::<Attribute>().unwrap(), Attribute::Title);
		// PascalCase
		assert_eq!("Title".parse::<Attribute>().unwrap(), Attribute::Title);
		// AX-prefixed
		assert_eq!("AXTitle".parse::<Attribute>().unwrap(), Attribute::Title);

		// Multi-word: camelCase
		assert_eq!("roleDescription".parse::<Attribute>().unwrap(), Attribute::RoleDescription);
		// Multi-word: PascalCase
		assert_eq!("RoleDescription".parse::<Attribute>().unwrap(), Attribute::RoleDescription);
		// Multi-word: AX-prefixed
		assert_eq!("AXRoleDescription".parse::<Attribute>().unwrap(), Attribute::RoleDescription);

		// All-caps: URL
		assert_eq!("url".parse::<Attribute>().unwrap(), Attribute::URL);
		assert_eq!("URL".parse::<Attribute>().unwrap(), Attribute::URL);
		assert_eq!("AXURL".parse::<Attribute>().unwrap(), Attribute::URL);
	}

	#[test]
	fn unknown_becomes_named() {
		assert_eq!("AXCustomThing".parse::<Attribute>().unwrap(), Attribute::Literal("AXCustomThing".into()));
		assert_eq!("myWeirdAttr".parse::<Attribute>().unwrap(), Attribute::Literal("myWeirdAttr".into()));
		assert_eq!("".parse::<Attribute>().unwrap(), Attribute::Literal("".into()));
	}
}
