use objc2_core_foundation::{CFRetained, CFString};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Subrole {
	/// A subrole referenced by name, for subroles not in the predefined list.
	Literal(String),

	// core
	CloseButton,
	MinimizeButton,
	ZoomButton,
	ToolbarButton,
	FullScreenButton,
	SecureTextField,
	TableRow,
	OutlineRow,
	Unknown,
	StandardWindow,
	Dialog,
	SystemDialog,
	FloatingWindow,
	SystemFloatingWindow,
	Decorative,
	IncrementArrow,
	DecrementArrow,
	IncrementPage,
	DecrementPage,
	SortButton,
	SearchField,
	Timeline,
	RatingIndicator,
	ContentList,
	DefinitionList,
	DescriptionList,
	Toggle,
	Switch,
	ApplicationDockItem,
	DocumentDockItem,
	FolderDockItem,
	MinimizedWindowDockItem,
	URLDockItem,
	DockExtraDockItem,
	TrashDockItem,
	SeparatorDockItem,
	ProcessSwitcherList,

	// web
	ApplicationAlertDialog,
	ApplicationAlert,
	ApplicationDialog,
	ApplicationGroup,
	ApplicationLog,
	ApplicationMarquee,
	ApplicationStatus,
	ApplicationTimer,
	Audio,
	CodeStyleGroup,
	Definition,
	DeleteStyleGroup,
	Details,
	DocumentArticle,
	DocumentMath,
	DocumentNote,
	EmptyGroup,
	Fieldset,
	FileUploadButton,
	InsertStyleGroup,
	LandmarkBanner,
	LandmarkComplementary,
	LandmarkContentInfo,
	LandmarkMain,
	LandmarkNavigation,
	LandmarkRegion,
	LandmarkSearch,
	MathFenceOperator,
	MathFenced,
	MathFraction,
	MathIdentifier,
	MathMultiscript,
	MathNumber,
	MathOperator,
	MathRoot,
	MathRow,
	MathSeparatorOperator,
	MathSquareRoot,
	MathSubscriptSuperscript,
	MathTableCell,
	MathTableRow,
	MathTable,
	MathText,
	MathUnderOver,
	Meter,
	RubyInline,
	RubyText,
	SubscriptStyleGroup,
	Summary,
	SuperscriptStyleGroup,
	TabPanel,
	Term,
	TimeGroup,
	UserInterfaceTooltip,
	Video,
	WebApplication,
}

impl std::str::FromStr for Subrole {
	type Err = std::convert::Infallible;

	// TODO: Add a literal/escape syntax (see Attribute::FromStr) for the same
	// name-collision reason.
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			// core
			"CloseButton" | "closeButton" | "AXCloseButton" => Subrole::CloseButton,
			"MinimizeButton" | "minimizeButton" | "AXMinimizeButton" => Subrole::MinimizeButton,
			"ZoomButton" | "zoomButton" | "AXZoomButton" => Subrole::ZoomButton,
			"ToolbarButton" | "toolbarButton" | "AXToolbarButton" => Subrole::ToolbarButton,
			"FullScreenButton" | "fullScreenButton" | "AXFullScreenButton" => Subrole::FullScreenButton,
			"SecureTextField" | "secureTextField" | "AXSecureTextField" => Subrole::SecureTextField,
			"TableRow" | "tableRow" | "AXTableRow" => Subrole::TableRow,
			"OutlineRow" | "outlineRow" | "AXOutlineRow" => Subrole::OutlineRow,
			"Unknown" | "unknown" | "AXUnknown" => Subrole::Unknown,
			"StandardWindow" | "standardWindow" | "AXStandardWindow" => Subrole::StandardWindow,
			"Dialog" | "dialog" | "AXDialog" => Subrole::Dialog,
			"SystemDialog" | "systemDialog" | "AXSystemDialog" => Subrole::SystemDialog,
			"FloatingWindow" | "floatingWindow" | "AXFloatingWindow" => Subrole::FloatingWindow,
			"SystemFloatingWindow" | "systemFloatingWindow" | "AXSystemFloatingWindow" => Subrole::SystemFloatingWindow,
			"Decorative" | "decorative" | "AXDecorative" => Subrole::Decorative,
			"IncrementArrow" | "incrementArrow" | "AXIncrementArrow" => Subrole::IncrementArrow,
			"DecrementArrow" | "decrementArrow" | "AXDecrementArrow" => Subrole::DecrementArrow,
			"IncrementPage" | "incrementPage" | "AXIncrementPage" => Subrole::IncrementPage,
			"DecrementPage" | "decrementPage" | "AXDecrementPage" => Subrole::DecrementPage,
			"SortButton" | "sortButton" | "AXSortButton" => Subrole::SortButton,
			"SearchField" | "searchField" | "AXSearchField" => Subrole::SearchField,
			"Timeline" | "timeline" | "AXTimeline" => Subrole::Timeline,
			"RatingIndicator" | "ratingIndicator" | "AXRatingIndicator" => Subrole::RatingIndicator,
			"ContentList" | "contentList" | "AXContentList" => Subrole::ContentList,
			"DefinitionList" | "definitionList" | "AXDefinitionList" => Subrole::DefinitionList,
			"DescriptionList" | "descriptionList" | "AXDescriptionList" => Subrole::DescriptionList,
			"Toggle" | "toggle" | "AXToggle" => Subrole::Toggle,
			"Switch" | "switch" | "AXSwitch" => Subrole::Switch,
			"ApplicationDockItem" | "applicationDockItem" | "AXApplicationDockItem" => Subrole::ApplicationDockItem,
			"DocumentDockItem" | "documentDockItem" | "AXDocumentDockItem" => Subrole::DocumentDockItem,
			"FolderDockItem" | "folderDockItem" | "AXFolderDockItem" => Subrole::FolderDockItem,
			"MinimizedWindowDockItem" | "minimizedWindowDockItem" | "AXMinimizedWindowDockItem" => Subrole::MinimizedWindowDockItem,
			"URLDockItem" | "uRLDockItem" | "urlDockItem" | "AXURLDockItem" => Subrole::URLDockItem,
			"DockExtraDockItem" | "dockExtraDockItem" | "AXDockExtraDockItem" => Subrole::DockExtraDockItem,
			"TrashDockItem" | "trashDockItem" | "AXTrashDockItem" => Subrole::TrashDockItem,
			"SeparatorDockItem" | "separatorDockItem" | "AXSeparatorDockItem" => Subrole::SeparatorDockItem,
			"ProcessSwitcherList" | "processSwitcherList" | "AXProcessSwitcherList" => Subrole::ProcessSwitcherList,

			// web
			"ApplicationAlertDialog" | "applicationAlertDialog" | "AXApplicationAlertDialog" => Subrole::ApplicationAlertDialog,
			"ApplicationAlert" | "applicationAlert" | "AXApplicationAlert" => Subrole::ApplicationAlert,
			"ApplicationDialog" | "applicationDialog" | "AXApplicationDialog" => Subrole::ApplicationDialog,
			"ApplicationGroup" | "applicationGroup" | "AXApplicationGroup" => Subrole::ApplicationGroup,
			"ApplicationLog" | "applicationLog" | "AXApplicationLog" => Subrole::ApplicationLog,
			"ApplicationMarquee" | "applicationMarquee" | "AXApplicationMarquee" => Subrole::ApplicationMarquee,
			"ApplicationStatus" | "applicationStatus" | "AXApplicationStatus" => Subrole::ApplicationStatus,
			"ApplicationTimer" | "applicationTimer" | "AXApplicationTimer" => Subrole::ApplicationTimer,
			"Audio" | "audio" | "AXAudio" => Subrole::Audio,
			"CodeStyleGroup" | "codeStyleGroup" | "AXCodeStyleGroup" => Subrole::CodeStyleGroup,
			"Definition" | "definition" | "AXDefinition" => Subrole::Definition,
			"DeleteStyleGroup" | "deleteStyleGroup" | "AXDeleteStyleGroup" => Subrole::DeleteStyleGroup,
			"Details" | "details" | "AXDetails" => Subrole::Details,
			"DocumentArticle" | "documentArticle" | "AXDocumentArticle" => Subrole::DocumentArticle,
			"DocumentMath" | "documentMath" | "AXDocumentMath" => Subrole::DocumentMath,
			"DocumentNote" | "documentNote" | "AXDocumentNote" => Subrole::DocumentNote,
			"EmptyGroup" | "emptyGroup" | "AXEmptyGroup" => Subrole::EmptyGroup,
			"Fieldset" | "fieldset" | "AXFieldset" => Subrole::Fieldset,
			"FileUploadButton" | "fileUploadButton" | "AXFileUploadButton" => Subrole::FileUploadButton,
			"InsertStyleGroup" | "insertStyleGroup" | "AXInsertStyleGroup" => Subrole::InsertStyleGroup,
			"LandmarkBanner" | "landmarkBanner" | "AXLandmarkBanner" => Subrole::LandmarkBanner,
			"LandmarkComplementary" | "landmarkComplementary" | "AXLandmarkComplementary" => Subrole::LandmarkComplementary,
			"LandmarkContentInfo" | "landmarkContentInfo" | "AXLandmarkContentInfo" => Subrole::LandmarkContentInfo,
			"LandmarkMain" | "landmarkMain" | "AXLandmarkMain" => Subrole::LandmarkMain,
			"LandmarkNavigation" | "landmarkNavigation" | "AXLandmarkNavigation" => Subrole::LandmarkNavigation,
			"LandmarkRegion" | "landmarkRegion" | "AXLandmarkRegion" => Subrole::LandmarkRegion,
			"LandmarkSearch" | "landmarkSearch" | "AXLandmarkSearch" => Subrole::LandmarkSearch,
			"MathFenceOperator" | "mathFenceOperator" | "AXMathFenceOperator" => Subrole::MathFenceOperator,
			"MathFenced" | "mathFenced" | "AXMathFenced" => Subrole::MathFenced,
			"MathFraction" | "mathFraction" | "AXMathFraction" => Subrole::MathFraction,
			"MathIdentifier" | "mathIdentifier" | "AXMathIdentifier" => Subrole::MathIdentifier,
			"MathMultiscript" | "mathMultiscript" | "AXMathMultiscript" => Subrole::MathMultiscript,
			"MathNumber" | "mathNumber" | "AXMathNumber" => Subrole::MathNumber,
			"MathOperator" | "mathOperator" | "AXMathOperator" => Subrole::MathOperator,
			"MathRoot" | "mathRoot" | "AXMathRoot" => Subrole::MathRoot,
			"MathRow" | "mathRow" | "AXMathRow" => Subrole::MathRow,
			"MathSeparatorOperator" | "mathSeparatorOperator" | "AXMathSeparatorOperator" => Subrole::MathSeparatorOperator,
			"MathSquareRoot" | "mathSquareRoot" | "AXMathSquareRoot" => Subrole::MathSquareRoot,
			"MathSubscriptSuperscript" | "mathSubscriptSuperscript" | "AXMathSubscriptSuperscript" => Subrole::MathSubscriptSuperscript,
			"MathTableCell" | "mathTableCell" | "AXMathTableCell" => Subrole::MathTableCell,
			"MathTableRow" | "mathTableRow" | "AXMathTableRow" => Subrole::MathTableRow,
			"MathTable" | "mathTable" | "AXMathTable" => Subrole::MathTable,
			"MathText" | "mathText" | "AXMathText" => Subrole::MathText,
			"MathUnderOver" | "mathUnderOver" | "AXMathUnderOver" => Subrole::MathUnderOver,
			"Meter" | "meter" | "AXMeter" => Subrole::Meter,
			"RubyInline" | "rubyInline" | "AXRubyInline" => Subrole::RubyInline,
			"RubyText" | "rubyText" | "AXRubyText" => Subrole::RubyText,
			"SubscriptStyleGroup" | "subscriptStyleGroup" | "AXSubscriptStyleGroup" => Subrole::SubscriptStyleGroup,
			"Summary" | "summary" | "AXSummary" => Subrole::Summary,
			"SuperscriptStyleGroup" | "superscriptStyleGroup" | "AXSuperscriptStyleGroup" => Subrole::SuperscriptStyleGroup,
			"TabPanel" | "tabPanel" | "AXTabPanel" => Subrole::TabPanel,
			"Term" | "term" | "AXTerm" => Subrole::Term,
			"TimeGroup" | "timeGroup" | "AXTimeGroup" => Subrole::TimeGroup,
			"UserInterfaceTooltip" | "userInterfaceTooltip" | "AXUserInterfaceTooltip" => Subrole::UserInterfaceTooltip,
			"Video" | "video" | "AXVideo" => Subrole::Video,
			"WebApplication" | "webApplication" | "AXWebApplication" => Subrole::WebApplication,
			_ => Subrole::Literal(s.to_string()),
		})
	}
}

impl Subrole {
	#[allow(non_snake_case)]
	pub fn to_CFString(&self) -> CFRetained<CFString> {
		self.into()
	}
}

impl From<&Subrole> for CFRetained<CFString> {
	fn from(subrole: &Subrole) -> CFRetained<CFString> {
		if let Subrole::Literal(name) = subrole {
			return CFString::from_str(name);
		}

		CFString::from_static_str(match subrole {
			Subrole::Literal(_) => unreachable!(),
			Subrole::CloseButton => "AXCloseButton",
			Subrole::MinimizeButton => "AXMinimizeButton",
			Subrole::ZoomButton => "AXZoomButton",
			Subrole::ToolbarButton => "AXToolbarButton",
			Subrole::FullScreenButton => "AXFullScreenButton",
			Subrole::SecureTextField => "AXSecureTextField",
			Subrole::TableRow => "AXTableRow",
			Subrole::OutlineRow => "AXOutlineRow",
			Subrole::Unknown => "AXUnknown",
			Subrole::StandardWindow => "AXStandardWindow",
			Subrole::Dialog => "AXDialog",
			Subrole::SystemDialog => "AXSystemDialog",
			Subrole::FloatingWindow => "AXFloatingWindow",
			Subrole::SystemFloatingWindow => "AXSystemFloatingWindow",
			Subrole::Decorative => "AXDecorative",
			Subrole::IncrementArrow => "AXIncrementArrow",
			Subrole::DecrementArrow => "AXDecrementArrow",
			Subrole::IncrementPage => "AXIncrementPage",
			Subrole::DecrementPage => "AXDecrementPage",
			Subrole::SortButton => "AXSortButton",
			Subrole::SearchField => "AXSearchField",
			Subrole::Timeline => "AXTimeline",
			Subrole::RatingIndicator => "AXRatingIndicator",
			Subrole::ContentList => "AXContentList",
			Subrole::DefinitionList => "AXDefinitionList",
			Subrole::DescriptionList => "AXDescriptionList",
			Subrole::Toggle => "AXToggle",
			Subrole::Switch => "AXSwitch",
			Subrole::ApplicationDockItem => "AXApplicationDockItem",
			Subrole::DocumentDockItem => "AXDocumentDockItem",
			Subrole::FolderDockItem => "AXFolderDockItem",
			Subrole::MinimizedWindowDockItem => "AXMinimizedWindowDockItem",
			Subrole::URLDockItem => "AXURLDockItem",
			Subrole::DockExtraDockItem => "AXDockExtraDockItem",
			Subrole::TrashDockItem => "AXTrashDockItem",
			Subrole::SeparatorDockItem => "AXSeparatorDockItem",
			Subrole::ProcessSwitcherList => "AXProcessSwitcherList",
			Subrole::ApplicationAlertDialog => "AXApplicationAlertDialog",
			Subrole::ApplicationAlert => "AXApplicationAlert",
			Subrole::ApplicationDialog => "AXApplicationDialog",
			Subrole::ApplicationGroup => "AXApplicationGroup",
			Subrole::ApplicationLog => "AXApplicationLog",
			Subrole::ApplicationMarquee => "AXApplicationMarquee",
			Subrole::ApplicationStatus => "AXApplicationStatus",
			Subrole::ApplicationTimer => "AXApplicationTimer",
			Subrole::Audio => "AXAudio",
			Subrole::CodeStyleGroup => "AXCodeStyleGroup",
			Subrole::Definition => "AXDefinition",
			Subrole::DeleteStyleGroup => "AXDeleteStyleGroup",
			Subrole::Details => "AXDetails",
			Subrole::DocumentArticle => "AXDocumentArticle",
			Subrole::DocumentMath => "AXDocumentMath",
			Subrole::DocumentNote => "AXDocumentNote",
			Subrole::EmptyGroup => "AXEmptyGroup",
			Subrole::Fieldset => "AXFieldset",
			Subrole::FileUploadButton => "AXFileUploadButton",
			Subrole::InsertStyleGroup => "AXInsertStyleGroup",
			Subrole::LandmarkBanner => "AXLandmarkBanner",
			Subrole::LandmarkComplementary => "AXLandmarkComplementary",
			Subrole::LandmarkContentInfo => "AXLandmarkContentInfo",
			Subrole::LandmarkMain => "AXLandmarkMain",
			Subrole::LandmarkNavigation => "AXLandmarkNavigation",
			Subrole::LandmarkRegion => "AXLandmarkRegion",
			Subrole::LandmarkSearch => "AXLandmarkSearch",
			Subrole::MathFenceOperator => "AXMathFenceOperator",
			Subrole::MathFenced => "AXMathFenced",
			Subrole::MathFraction => "AXMathFraction",
			Subrole::MathIdentifier => "AXMathIdentifier",
			Subrole::MathMultiscript => "AXMathMultiscript",
			Subrole::MathNumber => "AXMathNumber",
			Subrole::MathOperator => "AXMathOperator",
			Subrole::MathRoot => "AXMathRoot",
			Subrole::MathRow => "AXMathRow",
			Subrole::MathSeparatorOperator => "AXMathSeparatorOperator",
			Subrole::MathSquareRoot => "AXMathSquareRoot",
			Subrole::MathSubscriptSuperscript => "AXMathSubscriptSuperscript",
			Subrole::MathTableCell => "AXMathTableCell",
			Subrole::MathTableRow => "AXMathTableRow",
			Subrole::MathTable => "AXMathTable",
			Subrole::MathText => "AXMathText",
			Subrole::MathUnderOver => "AXMathUnderOver",
			Subrole::Meter => "AXMeter",
			Subrole::RubyInline => "AXRubyInline",
			Subrole::RubyText => "AXRubyText",
			Subrole::SubscriptStyleGroup => "AXSubscriptStyleGroup",
			Subrole::Summary => "AXSummary",
			Subrole::SuperscriptStyleGroup => "AXSuperscriptStyleGroup",
			Subrole::TabPanel => "AXTabPanel",
			Subrole::Term => "AXTerm",
			Subrole::TimeGroup => "AXTimeGroup",
			Subrole::UserInterfaceTooltip => "AXUserInterfaceTooltip",
			Subrole::Video => "AXVideo",
			Subrole::WebApplication => "AXWebApplication",
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_all_forms() {
		assert_eq!("dialog".parse::<Subrole>().unwrap(), Subrole::Dialog);
		assert_eq!("Dialog".parse::<Subrole>().unwrap(), Subrole::Dialog);
		assert_eq!("AXDialog".parse::<Subrole>().unwrap(), Subrole::Dialog);

		assert_eq!("closeButton".parse::<Subrole>().unwrap(), Subrole::CloseButton);
		assert_eq!("CloseButton".parse::<Subrole>().unwrap(), Subrole::CloseButton);
		assert_eq!("AXCloseButton".parse::<Subrole>().unwrap(), Subrole::CloseButton);

		assert_eq!("landmarkBanner".parse::<Subrole>().unwrap(), Subrole::LandmarkBanner);
		assert_eq!("LandmarkBanner".parse::<Subrole>().unwrap(), Subrole::LandmarkBanner);
		assert_eq!("AXLandmarkBanner".parse::<Subrole>().unwrap(), Subrole::LandmarkBanner);
	}

	#[test]
	fn url_dock_item_alias() {
		assert_eq!("urlDockItem".parse::<Subrole>().unwrap(), Subrole::URLDockItem);
		assert_eq!("URLDockItem".parse::<Subrole>().unwrap(), Subrole::URLDockItem);
		assert_eq!("AXURLDockItem".parse::<Subrole>().unwrap(), Subrole::URLDockItem);
	}

	#[test]
	fn unknown_becomes_named() {
		assert_eq!("AXCustomSubrole".parse::<Subrole>().unwrap(), Subrole::Literal("AXCustomSubrole".into()));
		assert_eq!("mySubrole".parse::<Subrole>().unwrap(), Subrole::Literal("mySubrole".into()));
	}
}
