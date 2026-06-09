import { App, Element, type AccessibilityEvent, type FilterStep, type RevivedJsonValue } from "../../globals";
import type { Attribute, MatchString } from "../../proto";

export { App, Element, Function, Key, Modifier, Pack, ScrollWheel, Var, Vars, View, action } from "../../globals";
export type { AccessibilityEvent, FilterStep, JsonArray, JsonObject, JsonPrimitive, JsonValue, RevivedJsonValue } from "../../globals";

const literal = (v: string): MatchString => ({ literal: v });

// prettier-ignore
export const Role = Object.freeze({
	APPLICATION: literal("AXApplication"), SYSTEM_WIDE: literal("AXSystemWide"), WINDOW: literal("AXWindow"),
	SHEET: literal("AXSheet"), DRAWER: literal("AXDrawer"), GROW_AREA: literal("AXGrowArea"), IMAGE: literal("AXImage"),
	UNKNOWN: literal("AXUnknown"), BUTTON: literal("AXButton"), RADIO_BUTTON: literal("AXRadioButton"),
	CHECK_BOX: literal("AXCheckBox"), POP_UP_BUTTON: literal("AXPopUpButton"), MENU_BUTTON: literal("AXMenuButton"),
	TAB_GROUP: literal("AXTabGroup"), TABLE: literal("AXTable"), COLUMN: literal("AXColumn"), ROW: literal("AXRow"),
	OUTLINE: literal("AXOutline"), BROWSER: literal("AXBrowser"), SCROLL_AREA: literal("AXScrollArea"),
	SCROLL_BAR: literal("AXScrollBar"), RADIO_GROUP: literal("AXRadioGroup"), LIST: literal("AXList"),
	GROUP: literal("AXGroup"), VALUE_INDICATOR: literal("AXValueIndicator"), COMBO_BOX: literal("AXComboBox"),
	SLIDER: literal("AXSlider"), INCREMENTOR: literal("AXIncrementor"), BUSY_INDICATOR: literal("AXBusyIndicator"),
	PROGRESS_INDICATOR: literal("AXProgressIndicator"), RELEVANCE_INDICATOR: literal("AXRelevanceIndicator"),
	TOOLBAR: literal("AXToolbar"), DISCLOSURE_TRIANGLE: literal("AXDisclosureTriangle"),
	TEXT_FIELD: literal("AXTextField"), TEXT_AREA: literal("AXTextArea"), STATIC_TEXT: literal("AXStaticText"),
	HEADING: literal("AXHeading"), MENU_BAR: literal("AXMenuBar"), MENU_BAR_ITEM: literal("AXMenuBarItem"),
	MENU: literal("AXMenu"), MENU_ITEM: literal("AXMenuItem"), SPLIT_GROUP: literal("AXSplitGroup"),
	SPLITTER: literal("AXSplitter"), COLOR_WELL: literal("AXColorWell"), TIME_FIELD: literal("AXTimeField"),
	DATE_FIELD: literal("AXDateField"), HELP_TAG: literal("AXHelpTag"), MATTE: literal("AXMatte"),
	DOCK_ITEM: literal("AXDockItem"), RULER: literal("AXRuler"), RULER_MARKER: literal("AXRulerMarker"),
	GRID: literal("AXGrid"), LEVEL_INDICATOR: literal("AXLevelIndicator"), CELL: literal("AXCell"),
	LAYOUT_AREA: literal("AXLayoutArea"), LAYOUT_ITEM: literal("AXLayoutItem"), HANDLE: literal("AXHandle"),
	POPOVER: literal("AXPopover"), IMAGE_MAP: literal("AXImageMap"),
});
export type Role = (typeof Role)[keyof typeof Role];

// prettier-ignore
export const Subrole = Object.freeze({
	CLOSE_BUTTON: literal("AXCloseButton"), MINIMIZE_BUTTON: literal("AXMinimizeButton"),
	ZOOM_BUTTON: literal("AXZoomButton"), TOOLBAR_BUTTON: literal("AXToolbarButton"),
	FULL_SCREEN_BUTTON: literal("AXFullScreenButton"), SECURE_TEXT_FIELD: literal("AXSecureTextField"),
	TABLE_ROW: literal("AXTableRow"), OUTLINE_ROW: literal("AXOutlineRow"), UNKNOWN: literal("AXUnknown"),
	STANDARD_WINDOW: literal("AXStandardWindow"), DIALOG: literal("AXDialog"),
	SYSTEM_DIALOG: literal("AXSystemDialog"), FLOATING_WINDOW: literal("AXFloatingWindow"),
	SYSTEM_FLOATING_WINDOW: literal("AXSystemFloatingWindow"), DECORATIVE: literal("AXDecorative"),
	INCREMENT_ARROW: literal("AXIncrementArrow"), DECREMENT_ARROW: literal("AXDecrementArrow"),
	INCREMENT_PAGE: literal("AXIncrementPage"), DECREMENT_PAGE: literal("AXDecrementPage"),
	SORT_BUTTON: literal("AXSortButton"), SEARCH_FIELD: literal("AXSearchField"),
	TIMELINE: literal("AXTimeline"), RATING_INDICATOR: literal("AXRatingIndicator"),
	CONTENT_LIST: literal("AXContentList"), DEFINITION_LIST: literal("AXDefinitionList"),
	DESCRIPTION_LIST: literal("AXDescriptionList"), TOGGLE: literal("AXToggle"), SWITCH: literal("AXSwitch"),
	APPLICATION_DOCK_ITEM: literal("AXApplicationDockItem"), DOCUMENT_DOCK_ITEM: literal("AXDocumentDockItem"),
	FOLDER_DOCK_ITEM: literal("AXFolderDockItem"), MINIMIZED_WINDOW_DOCK_ITEM: literal("AXMinimizedWindowDockItem"),
	URL_DOCK_ITEM: literal("AXURLDockItem"), DOCK_EXTRA_DOCK_ITEM: literal("AXDockExtraDockItem"),
	TRASH_DOCK_ITEM: literal("AXTrashDockItem"), SEPARATOR_DOCK_ITEM: literal("AXSeparatorDockItem"),
	PROCESS_SWITCHER_LIST: literal("AXProcessSwitcherList"), TAB_PANEL: literal("AXTabPanel"),
});
export type Subrole = (typeof Subrole)[keyof typeof Subrole];

// prettier-ignore
export const Action = Object.freeze({
	PRESS: literal("AXPress"), INCREMENT: literal("AXIncrement"), DECREMENT: literal("AXDecrement"),
	CONFIRM: literal("AXConfirm"), CANCEL: literal("AXCancel"), SHOW_ALTERNATE_UI: literal("AXShowAlternateUI"),
	SHOW_DEFAULT_UI: literal("AXShowDefaultUI"), RAISE: literal("AXRaise"),
	SHOW_MENU: literal("AXShowMenu"), PICK: literal("AXPick"),
});
export type Action = (typeof Action)[keyof typeof Action];

// prettier-ignore
export const Notification = Object.freeze({
	MAIN_WINDOW_CHANGED: literal("AXMainWindowChanged"), FOCUSED_WINDOW_CHANGED: literal("AXFocusedWindowChanged"),
	FOCUSED_UI_ELEMENT_CHANGED: literal("AXFocusedUIElementChanged"),
	APPLICATION_ACTIVATED: literal("AXApplicationActivated"), APPLICATION_DEACTIVATED: literal("AXApplicationDeactivated"),
	APPLICATION_HIDDEN: literal("AXApplicationHidden"), APPLICATION_SHOWN: literal("AXApplicationShown"),
	WINDOW_CREATED: literal("AXWindowCreated"), WINDOW_MOVED: literal("AXWindowMoved"),
	WINDOW_RESIZED: literal("AXWindowResized"), WINDOW_MINIATURIZED: literal("AXWindowMiniaturized"),
	WINDOW_DEMINIATURIZED: literal("AXWindowDeminiaturized"), DRAWER_CREATED: literal("AXDrawerCreated"),
	SHEET_CREATED: literal("AXSheetCreated"), HELP_TAG_CREATED: literal("AXHelpTagCreated"),
	VALUE_CHANGED: literal("AXValueChanged"), UI_ELEMENT_DESTROYED: literal("AXUIElementDestroyed"),
	ELEMENT_BUSY_CHANGED: literal("AXElementBusyChanged"), MENU_OPENED: literal("AXMenuOpened"),
	MENU_CLOSED: literal("AXMenuClosed"), MENU_ITEM_SELECTED: literal("AXMenuItemSelected"),
	ROW_COUNT_CHANGED: literal("AXRowCountChanged"), ROW_EXPANDED: literal("AXRowExpanded"),
	ROW_COLLAPSED: literal("AXRowCollapsed"), SELECTED_CELLS_CHANGED: literal("AXSelectedCellsChanged"),
	UNITS_CHANGED: literal("AXUnitsChanged"), SELECTED_CHILDREN_MOVED: literal("AXSelectedChildrenMoved"),
	SELECTED_CHILDREN_CHANGED: literal("AXSelectedChildrenChanged"),
	RESIZED: literal("AXResized"), MOVED: literal("AXMoved"), CREATED: literal("AXCreated"),
	SELECTED_ROWS_CHANGED: literal("AXSelectedRowsChanged"), SELECTED_COLUMNS_CHANGED: literal("AXSelectedColumnsChanged"),
	SELECTED_TEXT_CHANGED: literal("AXSelectedTextChanged"), TITLE_CHANGED: literal("AXTitleChanged"),
	LAYOUT_CHANGED: literal("AXLayoutChanged"), ANNOUNCEMENT_REQUESTED: literal("AXAnnouncementRequested"),
	ACTIVE_ELEMENT_CHANGED: literal("AXActiveElementChanged"), CURRENT_STATE_CHANGED: literal("AXCurrentStateChanged"),
	EXPANDED_CHANGED: literal("AXExpandedChanged"), INVALID_STATUS_CHANGED: literal("AXInvalidStatusChanged"),
	LAYOUT_COMPLETE: literal("AXLayoutComplete"), LIVE_REGION_CHANGED: literal("AXLiveRegionChanged"),
	LIVE_REGION_CREATED: literal("AXLiveRegionCreated"), LOAD_COMPLETE: literal("AXLoadComplete"),
});
export type Notification = (typeof Notification)[keyof typeof Notification];

// prettier-ignore
export const Orientation = Object.freeze({
	HORIZONTAL: literal("AXHorizontalOrientation"), VERTICAL: literal("AXVerticalOrientation"),
	UNKNOWN: literal("AXUnknownOrientation"),
});
export type Orientation = (typeof Orientation)[keyof typeof Orientation];

// prettier-ignore
export const SortDirection = Object.freeze({
	ASCENDING: literal("AXAscendingSortDirection"), DESCENDING: literal("AXDescendingSortDirection"),
	UNKNOWN: literal("AXUnknownSortDirection"),
});
export type SortDirection = (typeof SortDirection)[keyof typeof SortDirection];

export async function app(bundleIdentifier: string) {
	return new AppDelegate(await App.init(bundleIdentifier));
}

export class AppDelegate {
	#app: App;
	#delegate: ElementDelegate;

	constructor(app: App) {
		this.#app = app;
		this.#delegate = new ElementDelegate(this, undefined, app.element);
	}

	// Walking zero steps lands on the app itself. Without this, an empty walk returns
	// null and the app-root delegate latches dead after one failed read.
	walk = ((...args) => (args.length > 0 ? this.#app.element!.then(el => el.walk(...args)) : this.#app.element!)) as Element["walk"];

	get key() {
		return this.#app.key;
	}
	get handle() {
		return this.#app.handle;
	}
	get element() {
		return this.#app.element;
	}

	on = ((...args) => this.#delegate.on(...args)) as ElementDelegate["on"];
	$ = (...path: FilterStep[]) => new ElementDelegate(this, path);

	// The host invokes these on the inner App (rpc `workspaceAppActivated` et al.),
	// so forward them through — otherwise a pack's `app.onactivate = …` lands on
	// the wrapper and is never called. macOS won't re-fire focus notifications when
	// you switch INTO an app, so packs use `onactivate` to re-evaluate focus.
	get onactivate() {
		return this.#app.onactivate;
	}
	set onactivate(fn) {
		this.#app.onactivate = fn;
	}
	get ondeactivate() {
		return this.#app.ondeactivate;
	}
	set ondeactivate(fn) {
		this.#app.ondeactivate = fn;
	}
	get onterminate() {
		return this.#app.onterminate;
	}
	set onterminate(fn) {
		this.#app.onterminate = fn;
	}

	get elementBusy() {
		return this.#delegate.attribute.elementBusy;
	}
	get focusedApplication() {
		return this.#delegate.attribute.focusedApplication;
	}
	get isApplicationRunning() {
		return this.#delegate.attribute.isApplicationRunning;
	}
	get shownMenuUIElement() {
		return this.#delegate.attribute.shownMenuUIElement;
	}
	get labelValue() {
		return this.#delegate.attribute.labelValue;
	}
	get labelUIElements() {
		return this.#delegate.attribute.labelUIElements;
	}
	get url() {
		return this.#delegate.attribute.url;
	}
	get columnTitle() {
		return this.#delegate.attribute.columnTitle;
	}
	get incrementButton() {
		return this.#delegate.attribute.incrementButton;
	}
	get decrementButton() {
		return this.#delegate.attribute.decrementButton;
	}
	get incrementor() {
		return this.#delegate.attribute.incrementor;
	}
	get document() {
		return this.#delegate.attribute.document;
	}
	get previousContents() {
		return this.#delegate.attribute.previousContents;
	}
	get nextContents() {
		return this.#delegate.attribute.nextContents;
	}
	get contents() {
		return this.#delegate.attribute.contents;
	}
	get splitters() {
		return this.#delegate.attribute.splitters;
	}
	get selected() {
		return this.#delegate.attribute.selected;
	}
	get expanded() {
		return this.#delegate.attribute.expanded;
	}
	get filename() {
		return this.#delegate.attribute.filename;
	}
	get overflowButton() {
		return this.#delegate.attribute.overflowButton;
	}
	get tabs() {
		return this.#delegate.attribute.tabs;
	}
	get edited() {
		return this.#delegate.attribute.edited;
	}
	get header() {
		return this.#delegate.attribute.header;
	}
	get orientation() {
		return this.#delegate.attribute.orientation;
	}
	get verticalScrollBar() {
		return this.#delegate.attribute.verticalScrollBar;
	}
	get horizontalScrollBar() {
		return this.#delegate.attribute.horizontalScrollBar;
	}
	get markerTypeDescription() {
		return this.#delegate.attribute.markerTypeDescription;
	}
	get markerType() {
		return this.#delegate.attribute.markerType;
	}
	get unitDescription() {
		return this.#delegate.attribute.unitDescription;
	}
	get units() {
		return this.#delegate.attribute.units;
	}
	get markerUIElements() {
		return this.#delegate.attribute.markerUIElements;
	}
	get matteContentUIElement() {
		return this.#delegate.attribute.matteContentUIElement;
	}
	get matteHole() {
		return this.#delegate.attribute.matteHole;
	}
	get disclosedByRow() {
		return this.#delegate.attribute.disclosedByRow;
	}
	get disclosedRows() {
		return this.#delegate.attribute.disclosedRows;
	}
	get disclosing() {
		return this.#delegate.attribute.disclosing;
	}
	get index() {
		return this.#delegate.attribute.index;
	}
	get columnHeaderUIElements() {
		return this.#delegate.attribute.columnHeaderUIElements;
	}
	get sortDirection() {
		return this.#delegate.attribute.sortDirection;
	}
	get selectedColumns() {
		return this.#delegate.attribute.selectedColumns;
	}
	get visibleColumns() {
		return this.#delegate.attribute.visibleColumns;
	}
	get columns() {
		return this.#delegate.attribute.columns;
	}
	get selectedRows() {
		return this.#delegate.attribute.selectedRows;
	}
	get visibleRows() {
		return this.#delegate.attribute.visibleRows;
	}
	get rows() {
		return this.#delegate.attribute.rows;
	}
	get yearField() {
		return this.#delegate.attribute.yearField;
	}
	get monthField() {
		return this.#delegate.attribute.monthField;
	}
	get dayField() {
		return this.#delegate.attribute.dayField;
	}
	get ampmfield() {
		return this.#delegate.attribute.ampmfield;
	}
	get secondField() {
		return this.#delegate.attribute.secondField;
	}
	get minuteField() {
		return this.#delegate.attribute.minuteField;
	}
	get hourField() {
		return this.#delegate.attribute.hourField;
	}
	get extrasMenuBar() {
		return this.#delegate.attribute.extrasMenuBar;
	}
	get focusedUIElement() {
		return this.#delegate.attribute.focusedUIElement;
	}
	get focusedWindow() {
		return this.#delegate.attribute.focusedWindow;
	}
	get mainWindow() {
		return this.#delegate.attribute.mainWindow;
	}
	get hidden() {
		return this.#delegate.attribute.hidden;
	}
	get frontmost() {
		return this.#delegate.attribute.frontmost;
	}
	get windows() {
		return this.#delegate.attribute.windows;
	}
	get menuBar() {
		return this.#delegate.attribute.menuBar;
	}
	get menuItemPrimaryUIElement() {
		return this.#delegate.attribute.menuItemPrimaryUIElement;
	}
	get menuItemMarkChar() {
		return this.#delegate.attribute.menuItemMarkChar;
	}
	get menuItemCmdModifiers() {
		return this.#delegate.attribute.menuItemCmdModifiers;
	}
	get menuItemCmdGlyph() {
		return this.#delegate.attribute.menuItemCmdGlyph;
	}
	get menuItemCmdVirtualKey() {
		return this.#delegate.attribute.menuItemCmdVirtualKey;
	}
	get menuItemCmdChar() {
		return this.#delegate.attribute.menuItemCmdChar;
	}
	get cancelButton() {
		return this.#delegate.attribute.cancelButton;
	}
	get defaultButton() {
		return this.#delegate.attribute.defaultButton;
	}
	get modal() {
		return this.#delegate.attribute.modal;
	}
	get growArea() {
		return this.#delegate.attribute.growArea;
	}
	get proxy() {
		return this.#delegate.attribute.proxy;
	}
	get toolbarButton() {
		return this.#delegate.attribute.toolbarButton;
	}
	get minimizeButton() {
		return this.#delegate.attribute.minimizeButton;
	}
	get zoomButton() {
		return this.#delegate.attribute.zoomButton;
	}
	get closeButton() {
		return this.#delegate.attribute.closeButton;
	}
	get minimized() {
		return this.#delegate.attribute.minimized;
	}
	get main() {
		return this.#delegate.attribute.main;
	}
	get sharedCharacterRange() {
		return this.#delegate.attribute.sharedCharacterRange;
	}
	get sharedTextUIElements() {
		return this.#delegate.attribute.sharedTextUIElements;
	}
	get numberOfCharacters() {
		return this.#delegate.attribute.numberOfCharacters;
	}
	get visibleCharacterRange() {
		return this.#delegate.attribute.visibleCharacterRange;
	}
	get selectedTextRanges() {
		return this.#delegate.attribute.selectedTextRanges;
	}
	get selectedTextRange() {
		return this.#delegate.attribute.selectedTextRange;
	}
	get selectedText() {
		return this.#delegate.attribute.selectedText;
	}
	get placeholderValue() {
		return this.#delegate.attribute.placeholderValue;
	}
	get allowedValues() {
		return this.#delegate.attribute.allowedValues;
	}
	get valueWraps() {
		return this.#delegate.attribute.valueWraps;
	}
	get valueIncrement() {
		return this.#delegate.attribute.valueIncrement;
	}
	get maxValue() {
		return this.#delegate.attribute.maxValue;
	}
	get minValue() {
		return this.#delegate.attribute.minValue;
	}
	get valueDescription() {
		return this.#delegate.attribute.valueDescription;
	}
	get value() {
		return this.#delegate.attribute.value;
	}
	get size() {
		return this.#delegate.attribute.size;
	}
	get position() {
		return this.#delegate.attribute.position;
	}
	get focused() {
		return this.#delegate.attribute.focused;
	}
	get enabled() {
		return this.#delegate.attribute.enabled;
	}
	get sharedFocusElements() {
		return this.#delegate.attribute.sharedFocusElements;
	}
	get linkedUIElements() {
		return this.#delegate.attribute.linkedUIElements;
	}
	get servesAsTitleForUIElements() {
		return this.#delegate.attribute.servesAsTitleForUIElements;
	}
	get titleUIElement() {
		return this.#delegate.attribute.titleUIElement;
	}
	get topLevelUIElement() {
		return this.#delegate.attribute.topLevelUIElement;
	}
	get window() {
		return this.#delegate.attribute.window;
	}
	get visibleChildren() {
		return this.#delegate.attribute.visibleChildren;
	}
	get selectedChildren() {
		return this.#delegate.attribute.selectedChildren;
	}
	get children() {
		return this.#delegate.attribute.children;
	}
	get parent() {
		return this.#delegate.attribute.parent;
	}
	get help() {
		return this.#delegate.attribute.help;
	}
	get description() {
		return this.#delegate.attribute.description;
	}
	get title() {
		return this.#delegate.attribute.title;
	}
	get roleDescription() {
		return this.#delegate.attribute.roleDescription;
	}
	get subrole() {
		return this.#delegate.attribute.subrole;
	}
	get role() {
		return this.#delegate.attribute.role;
	}
	get identifier() {
		return this.#delegate.attribute.identifier;
	}
	get alternateUIVisible() {
		return this.#delegate.attribute.alternateUIVisible;
	}

	// ---------------------------------------------------------------------------------------------------------------------

	pick = () => this.#delegate.action.AXPick?.();
	showMenu = () => this.#delegate.action.AXShowMenu?.();
	raise = () => this.#delegate.action.AXRaise?.();
	showDefaultUI = () => this.#delegate.action.AXShowDefaultUI?.();
	showAlternateUI = () => this.#delegate.action.AXShowAlternateUI?.();
	cancel = () => this.#delegate.action.AXCancel?.();
	confirm = () => this.#delegate.action.AXConfirm?.();
	decrement = () => this.#delegate.action.AXDecrement?.();
	increment = () => this.#delegate.action.AXIncrement?.();
	press = () => this.#delegate.action.AXPress?.();
}

type DefaultNotificationInfos = {
	windowCreated: undefined;
	applicationShown: undefined;
	applicationHidden: undefined;
	applicationDeactivated: undefined;
	applicationActivated: undefined;
	focusedUIElementChanged: undefined;
	focusedWindowChanged: undefined;
	mainWindowChanged: undefined;
	loadComplete: undefined;
	liveRegionCreated: undefined;
	liveRegionChanged: undefined;
	layoutComplete: undefined;
	invalidStatusChanged: undefined;
	expandedChanged: undefined;
	currentStateChanged: undefined;
	activeElementChanged: undefined;
	layoutChanged: undefined;
	titleChanged: undefined;
	selectedTextChanged: undefined;
	selectedColumnsChanged: undefined;
	selectedRowsChanged: undefined;
	created: undefined;
	moved: undefined;
	resized: undefined;
	selectedChildrenChanged: undefined;
	selectedChildrenMoved: undefined;
	unitsChanged: undefined;
	selectedCellsChanged: undefined;
	rowCollapsed: undefined;
	rowExpanded: undefined;
	rowCountChanged: undefined;
	menuItemSelected: undefined;
	menuClosed: undefined;
	menuOpened: undefined;
	elementBusyChanged: undefined;
	UIElementDestroyed: undefined;
	valueChanged: undefined;
	helpTagCreated: undefined;
	sheetCreated: undefined;
	drawerCreated: undefined;
	windowDeminiaturized: undefined;
	windowMiniaturized: undefined;
	windowResized: undefined;
	windowMoved: undefined;

	announcementRequested: Record<string, any>;
};

export class ElementDelegate {
	#root: ElementDelegate | AppDelegate;
	#pathFromRoot?: FilterStep[];
	#element?: Promise<Element>;

	constructor(root: ElementDelegate | AppDelegate, pathFromRoot?: FilterStep[], element?: Element | Promise<Element>) {
		this.#root = root;
		if (pathFromRoot) this.#pathFromRoot = pathFromRoot;
		if (element) this.#element = element instanceof Promise ? element : Promise.resolve(element);
	}

	static transformNotificationName(notificationName: string) {
		// prettier-ignore
		switch (notificationName) {
			case "announcementRequested": return "AXAnnouncementRequested";
			case "windowMoved": return "AXWindowMoved";
			case "windowResized": return "AXWindowResized";
			case "windowMiniaturized": return "AXWindowMiniaturized";
			case "windowDeminiaturized": return "AXWindowDeminiaturized";
			case "drawerCreated": return "AXDrawerCreated";
			case "sheetCreated": return "AXSheetCreated";
			case "helpTagCreated": return "AXHelpTagCreated";
			case "valueChanged": return "AXValueChanged";
			case "UIElementDestroyed": return "AXUIElementDestroyed";
			case "elementBusyChanged": return "AXElementBusyChanged";
			case "menuOpened": return "AXMenuOpened";
			case "menuClosed": return "AXMenuClosed";
			case "menuItemSelected": return "AXMenuItemSelected";
			case "rowCountChanged": return "AXRowCountChanged";
			case "rowExpanded": return "AXRowExpanded";
			case "rowCollapsed": return "AXRowCollapsed";
			case "selectedCellsChanged": return "AXSelectedCellsChanged";
			case "unitsChanged": return "AXUnitsChanged";
			case "selectedChildrenMoved": return "AXSelectedChildrenMoved";
			case "selectedChildrenChanged": return "AXSelectedChildrenChanged";
			case "resized": return "AXResized";
			case "moved": return "AXMoved";
			case "created": return "AXCreated";
			case "selectedRowsChanged": return "AXSelectedRowsChanged";
			case "selectedColumnsChanged": return "AXSelectedColumnsChanged";
			case "selectedTextChanged": return "AXSelectedTextChanged";
			case "titleChanged": return "AXTitleChanged";
			case "layoutChanged": return "AXLayoutChanged";
			case "activeElementChanged": return "AXActiveElementChanged";
			case "currentStateChanged": return "AXCurrentStateChanged";
			case "expandedChanged": return "AXExpandedChanged";
			case "invalidStatusChanged": return "AXInvalidStatusChanged";
			case "layoutComplete": return "AXLayoutComplete";
			case "liveRegionChanged": return "AXLiveRegionChanged";
			case "liveRegionCreated": return "AXLiveRegionCreated";
			case "loadComplete": return "AXLoadComplete";
			case "mainWindowChanged": return "AXMainWindowChanged";
			case "focusedWindowChanged": return "AXFocusedWindowChanged";
			case "focusedUIElementChanged": return "AXFocusedUIElementChanged";
			case "applicationActivated": return "AXApplicationActivated";
			case "applicationDeactivated": return "AXApplicationDeactivated";
			case "applicationHidden": return "AXApplicationHidden";
			case "applicationShown": return "AXApplicationShown";
			case "windowCreated": return "AXWindowCreated";
			default: return notificationName;
		}
	}

	static transformCallback(callback: (event: AccessibilityEvent<RevivedJsonValue>) => void | Promise<void>) {
		return (event: AccessibilityEvent<RevivedJsonValue>) => {
			const info = event.info;

			if (typeof info === "object" && info !== null) {
				for (const key in info) {
					if (key.startsWith("AX") && key.endsWith("Key")) {
						const data = info as Record<string, unknown>;
						// AXPriorityKey -> priority
						data[key[2]!.toLowerCase() + key.slice(3, -3)] = data[key];
						//   ^ lowercase first letter (Priority -> priority)
					}
				}
			}

			return callback(event);
		};
	}

	get path() {
		const pathFromRoot = this.#pathFromRoot;
		if (!pathFromRoot) return null;

		if (!(this.#root instanceof ElementDelegate)) return pathFromRoot.slice();

		const rootPath = this.#root.path;
		if (!rootPath) return null;

		// Makes sense, right?
		//
		//   root's path + path to ourselves from root
		// = absolute path to ourselves
		return rootPath.concat(pathFromRoot);
	}

	get app() {
		let current = this.#root;
		while (current instanceof ElementDelegate) current = current.#root;
		return current;
	}

	get element() {
		this.#element ??= this.app.walk(...(this.path ?? [])).then(el => el ?? Promise.reject(new Error("Element not found")));
		return this.#element;
	}

	$ = (...path: FilterStep[]) => new ElementDelegate(this, path);

	invalidate = () => (this.#element = undefined);

	on(
		notificationName: keyof DefaultNotificationInfos | (string & {}),
		callback: (event: AccessibilityEvent<RevivedJsonValue>) => void | Promise<void>,
	): () => Promise<void> {
		const unregister = this.element.then(el =>
			el?.on(ElementDelegate.transformNotificationName(notificationName), ElementDelegate.transformCallback(callback)),
		);
		return async () => (await unregister)?.();
	}

	// On failure, drop the cached element so the next access re-walks the query,
	// then rethrow — the caller still has to see this call fail.
	attribute = new Proxy(this, {
		get: (target, prop: string) =>
			target.element
				.then(el => el.attribute[prop as Extract<Attribute, string>])
				.catch(e => {
					target.invalidate();
					throw e;
				}),
	}) as any as Record<Extract<Attribute, string>, Promise<RevivedJsonValue>>;

	action = new Proxy(this, {
		get: (target, prop: string) => () =>
			target.element
				.then(el => el.action[prop]!())
				.catch(e => {
					target.invalidate();
					throw e;
				}),
	}) as any as Record<string, () => Promise<void>>;

	// ---------------------------------------------------------------------------------------------------------------------

	get elementBusy() {
		return this.attribute.elementBusy;
	}
	get focusedApplication() {
		return this.attribute.focusedApplication;
	}
	get isApplicationRunning() {
		return this.attribute.isApplicationRunning;
	}
	get shownMenuUIElement() {
		return this.attribute.shownMenuUIElement;
	}
	get labelValue() {
		return this.attribute.labelValue;
	}
	get labelUIElements() {
		return this.attribute.labelUIElements;
	}
	get url() {
		return this.attribute.url;
	}
	get columnTitle() {
		return this.attribute.columnTitle;
	}
	get incrementButton() {
		return this.attribute.incrementButton;
	}
	get decrementButton() {
		return this.attribute.decrementButton;
	}
	get incrementor() {
		return this.attribute.incrementor;
	}
	get document() {
		return this.attribute.document;
	}
	get previousContents() {
		return this.attribute.previousContents;
	}
	get nextContents() {
		return this.attribute.nextContents;
	}
	get contents() {
		return this.attribute.contents;
	}
	get splitters() {
		return this.attribute.splitters;
	}
	get selected() {
		return this.attribute.selected;
	}
	get expanded() {
		return this.attribute.expanded;
	}
	get filename() {
		return this.attribute.filename;
	}
	get overflowButton() {
		return this.attribute.overflowButton;
	}
	get tabs() {
		return this.attribute.tabs;
	}
	get edited() {
		return this.attribute.edited;
	}
	get header() {
		return this.attribute.header;
	}
	get orientation() {
		return this.attribute.orientation;
	}
	get verticalScrollBar() {
		return this.attribute.verticalScrollBar;
	}
	get horizontalScrollBar() {
		return this.attribute.horizontalScrollBar;
	}
	get markerTypeDescription() {
		return this.attribute.markerTypeDescription;
	}
	get markerType() {
		return this.attribute.markerType;
	}
	get unitDescription() {
		return this.attribute.unitDescription;
	}
	get units() {
		return this.attribute.units;
	}
	get markerUIElements() {
		return this.attribute.markerUIElements;
	}
	get matteContentUIElement() {
		return this.attribute.matteContentUIElement;
	}
	get matteHole() {
		return this.attribute.matteHole;
	}
	get disclosedByRow() {
		return this.attribute.disclosedByRow;
	}
	get disclosedRows() {
		return this.attribute.disclosedRows;
	}
	get disclosing() {
		return this.attribute.disclosing;
	}
	get index() {
		return this.attribute.index;
	}
	get columnHeaderUIElements() {
		return this.attribute.columnHeaderUIElements;
	}
	get sortDirection() {
		return this.attribute.sortDirection;
	}
	get selectedColumns() {
		return this.attribute.selectedColumns;
	}
	get visibleColumns() {
		return this.attribute.visibleColumns;
	}
	get columns() {
		return this.attribute.columns;
	}
	get selectedRows() {
		return this.attribute.selectedRows;
	}
	get visibleRows() {
		return this.attribute.visibleRows;
	}
	get rows() {
		return this.attribute.rows;
	}
	get yearField() {
		return this.attribute.yearField;
	}
	get monthField() {
		return this.attribute.monthField;
	}
	get dayField() {
		return this.attribute.dayField;
	}
	get ampmfield() {
		return this.attribute.ampmfield;
	}
	get secondField() {
		return this.attribute.secondField;
	}
	get minuteField() {
		return this.attribute.minuteField;
	}
	get hourField() {
		return this.attribute.hourField;
	}
	get extrasMenuBar() {
		return this.attribute.extrasMenuBar;
	}
	get focusedUIElement() {
		return this.attribute.focusedUIElement;
	}
	get focusedWindow() {
		return this.attribute.focusedWindow;
	}
	get mainWindow() {
		return this.attribute.mainWindow;
	}
	get hidden() {
		return this.attribute.hidden;
	}
	get frontmost() {
		return this.attribute.frontmost;
	}
	get windows() {
		return this.attribute.windows;
	}
	get menuBar() {
		return this.attribute.menuBar;
	}
	get menuItemPrimaryUIElement() {
		return this.attribute.menuItemPrimaryUIElement;
	}
	get menuItemMarkChar() {
		return this.attribute.menuItemMarkChar;
	}
	get menuItemCmdModifiers() {
		return this.attribute.menuItemCmdModifiers;
	}
	get menuItemCmdGlyph() {
		return this.attribute.menuItemCmdGlyph;
	}
	get menuItemCmdVirtualKey() {
		return this.attribute.menuItemCmdVirtualKey;
	}
	get menuItemCmdChar() {
		return this.attribute.menuItemCmdChar;
	}
	get cancelButton() {
		return this.attribute.cancelButton;
	}
	get defaultButton() {
		return this.attribute.defaultButton;
	}
	get modal() {
		return this.attribute.modal;
	}
	get growArea() {
		return this.attribute.growArea;
	}
	get proxy() {
		return this.attribute.proxy;
	}
	get toolbarButton() {
		return this.attribute.toolbarButton;
	}
	get minimizeButton() {
		return this.attribute.minimizeButton;
	}
	get zoomButton() {
		return this.attribute.zoomButton;
	}
	get closeButton() {
		return this.attribute.closeButton;
	}
	get minimized() {
		return this.attribute.minimized;
	}
	get main() {
		return this.attribute.main;
	}
	get sharedCharacterRange() {
		return this.attribute.sharedCharacterRange;
	}
	get sharedTextUIElements() {
		return this.attribute.sharedTextUIElements;
	}
	get numberOfCharacters() {
		return this.attribute.numberOfCharacters;
	}
	get visibleCharacterRange() {
		return this.attribute.visibleCharacterRange;
	}
	get selectedTextRanges() {
		return this.attribute.selectedTextRanges;
	}
	get selectedTextRange() {
		return this.attribute.selectedTextRange;
	}
	get selectedText() {
		return this.attribute.selectedText;
	}
	get placeholderValue() {
		return this.attribute.placeholderValue;
	}
	get allowedValues() {
		return this.attribute.allowedValues;
	}
	get valueWraps() {
		return this.attribute.valueWraps;
	}
	get valueIncrement() {
		return this.attribute.valueIncrement;
	}
	get maxValue() {
		return this.attribute.maxValue;
	}
	get minValue() {
		return this.attribute.minValue;
	}
	get valueDescription() {
		return this.attribute.valueDescription;
	}
	get value() {
		return this.attribute.value;
	}
	get size() {
		return this.attribute.size;
	}
	get position() {
		return this.attribute.position;
	}
	get focused() {
		return this.attribute.focused;
	}
	get enabled() {
		return this.attribute.enabled;
	}
	get sharedFocusElements() {
		return this.attribute.sharedFocusElements;
	}
	get linkedUIElements() {
		return this.attribute.linkedUIElements;
	}
	get servesAsTitleForUIElements() {
		return this.attribute.servesAsTitleForUIElements;
	}
	get titleUIElement() {
		return this.attribute.titleUIElement;
	}
	get topLevelUIElement() {
		return this.attribute.topLevelUIElement;
	}
	get window() {
		return this.attribute.window;
	}
	get visibleChildren() {
		return this.attribute.visibleChildren;
	}
	get selectedChildren() {
		return this.attribute.selectedChildren;
	}
	get children() {
		return this.attribute.children;
	}
	get parent() {
		return this.attribute.parent;
	}
	get help() {
		return this.attribute.help;
	}
	get description() {
		return this.attribute.description;
	}
	get title() {
		return this.attribute.title;
	}
	get roleDescription() {
		return this.attribute.roleDescription;
	}
	get subrole() {
		return this.attribute.subrole;
	}
	get role() {
		return this.attribute.role;
	}
	get identifier() {
		return this.attribute.identifier;
	}
	get alternateUIVisible() {
		return this.attribute.alternateUIVisible;
	}

	// ---------------------------------------------------------------------------------------------------------------------

	pick = () => this.action.AXPick?.();
	showMenu = () => this.action.AXShowMenu?.();
	raise = () => this.action.AXRaise?.();
	showDefaultUI = () => this.action.AXShowDefaultUI?.();
	showAlternateUI = () => this.action.AXShowAlternateUI?.();
	cancel = () => this.action.AXCancel?.();
	confirm = () => this.action.AXConfirm?.();
	decrement = () => this.action.AXDecrement?.();
	increment = () => this.action.AXIncrement?.();
	press = () => this.action.AXPress?.();
}

/**
 * Navigate through the menubar and press the final menu item.
 * Steps are menu titles, supports glob patterns (e.g., "Export Audio/Video*").
 */
export async function menubar(app: AppDelegate, ...steps: string[]) {
	if (steps.length === 0) return;

	// Start from the menu bar
	let current = app.$({ role: Role.MENU_BAR });

	for (let i = 0; i < steps.length; i++) {
		const step = steps[i]!;
		const isLast = i === steps.length - 1;

		// Find menu item with matching title
		current = current.$({ title: step });

		// If not last, we need to navigate into the opened menu
		if (!isLast) {
			// After pressing a menu bar item, the menu opens as a child
			// Navigate to the next level
			current = current.$({ role: Role.MENU });
		} else {
			return current.press();
		}
	}
}
