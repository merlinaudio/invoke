import { createElement } from "react";
import type { AppHandle, Attribute, ElementHandle, Filter, MatchString } from "./proto";
import { flight, registerAction, registerView, serializeView } from "./flight";

// Opaque pack-protocol handles — libinvoke's proto carries these inline as `number`.
type VarHandle = number;
type NotificationRegistrationHandle = number;

// `Filters` is the object form of the AX `Filter` union (used to build `FilterStep`).
type UnionToIntersection<U> = (U extends unknown ? (arg: U) => void : never) extends (arg: infer I) => void ? I : never;
type StripNever<T> = T extends any ? { [K in keyof T as T[K] extends never | undefined ? never : K]: T[K] } : never;
type Filters = UnionToIntersection<StripNever<Exclude<Filter, string>>>;
import * as rpc from "./rpc";

export type PackHandle = {
	publisherDomain: string;
	packName: string;
};
export type FunctionHandle = number;

(Symbol as any).dispose ??= Symbol("Symbol.dispose");
(Symbol as any).asyncDispose ??= Symbol("Symbol.asyncDispose");

export type JsonObject = { [Key in string]: JsonValue };
export type JsonArray = JsonValue[] | readonly JsonValue[];
export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | JsonObject | JsonArray;

export type RevivedJsonObject = { [Key in string]: RevivedJsonValue };
export type RevivedJsonArray = RevivedJsonValue[] | readonly RevivedJsonValue[];
export type RevivedJsonValue =
	| JsonPrimitive
	| RevivedJsonObject
	| RevivedJsonArray
	// Special cases here are revived at the RPC boundary.
	| Element;

export type AccessibilityEvent<Info = RevivedJsonObject> = {
	name: string;
	element: Element | null;
	info: Info;
};

/**
 * Register a react server action.
 *
 * @example
 * let i = 0;
 * const increment = action(() => i += 1);
 * View.init(() => <Button action={increment}>Count: {i}</Button>)
 */
export const action = registerAction;

/**
 * RSC container. Always owned — currently by functions, later by packs or
 * other things. Owner carries the view's ID. No view, no UI.
 *
 *     View = new View(() => <Function name="zoom" />)
 *            Doesn't render until pulled. Owner just holds the ID.
 *            Host pulls payload from worker on demand.
 *            1st render, 100th render — same pull.
 *
 * Functions without an explicit view get a default:
 * `new View(() => <Function name={name} />)`.
 *
 * Search is component-driven. Components register themselves:
 *
 *     <Function> mounts   → registry.set("zoom", fuzzy, exact)
 *     <Function> unmounts → registry.delete("zoom")  ← automatic via ViewSearchProvider
 *
 * ViewSearchProvider wraps each view's tree. Tracks what its children
 * registered. Cleans up on unmount — action re-render, pack removal,
 * whatever. Components register, React lifecycle unregisters.
 *
 * All views stay mounted. Search hides with display:none, reorders with
 * CSS order. Can't search for something that unregistered itself.
 */
export class View {
	#id: number;
	#component: () => any;

	constructor(component: () => any) {
		this.#component = component;
		this.#id = registerView(this);
	}

	get id() {
		return this.#id;
	}

	render() {
		return serializeView(this.#component());
	}
}

export class Modifier {
	static CapsLock = 1 << 1;
	static Shift = 1 << 2;
	static Control = 1 << 3;
	static Option = 1 << 4;
	static Command = 1 << 5;
	static Help = 1 << 6;
	static SecondaryFn = 1 << 7;
	static Numpad = 1 << 8;
}

export class Function {
	#handle: FunctionHandle;

	// Errors propagate to the transport, which reports them to the host — as an
	// ERROR response when the run expects one, logged otherwise.
	run: (payload: unknown) => unknown | Promise<unknown>;
	end?: () => void | Promise<void>;

	static async init(name: string, run: (payload: unknown) => unknown | Promise<unknown>): Promise<Function>;
	static async init(name: string, run: (payload: unknown) => unknown | Promise<unknown>, end: () => void | Promise<void>): Promise<Function>;
	static async init(name: string, run: (payload: unknown) => unknown | Promise<unknown>, view: View): Promise<Function>;
	static async init(name: string, run: (payload: unknown) => unknown | Promise<unknown>, end: () => void | Promise<void>, view: View): Promise<Function>;
	static async init(
		name: string,
		run: (payload: unknown) => unknown | Promise<unknown>,
		end: (() => void | Promise<void>) | undefined,
		view: View | undefined,
	): Promise<Function>;
	static async init(...args: any[]) {
		let name: string, run: (payload: unknown) => unknown | Promise<unknown>, end: (() => void | Promise<void>) | undefined, view: View | undefined;

		// Pop View from end if present
		if (args[args.length - 1] instanceof View) view = args.pop();

		[name, run, end] = args;

		if (typeof name !== "string") throw new Error(`First argument must be the function name. Got: ${name} (${typeof name})`);

		// Default view: just a <Function>
		view ??= new View(() => createElement(flight.modules["invoke/ui"]!.Function, { name }));

		const functionHandle = await rpc.defineFunction(name, view.id);

		if (functionHandle == null) throw new Error(`Failed to declare function "${name}"`);

		return new Function(functionHandle, run, end);
	}

	constructor(handle: FunctionHandle, run: (payload: unknown) => unknown | Promise<unknown>, end?: () => void | Promise<void>) {
		this.#handle = handle;
		this.run = run;
		this.end = end;
		rpc.registerFunction(handle, this);
	}

	get handle() {
		return this.#handle;
	}
}

type ModifierStr =
	| "caps"
	| "shift"
	| "control"
	| "option"
	| "command"
	| "help"
	| "secondaryFn"
	| "numpad"
	| "capsLock"
	| "ctrl"
	| "alternate"
	| "opt"
	| "alt"
	| "cmd"
	| "help"
	| "secondaryFn"
	| "numpad";

// prettier-ignore
type KeyName =
	| "f1" | "f2" | "f3" | "f4" | "f5" | "f6" | "f7" | "f8" | "f9" | "f10"
	| "f11" | "f12" | "f13" | "f14" | "f15" | "f16" | "f17" | "f18" | "f19" | "f20"
	| "escape" | "delete" | "tab" | "capsLock" | "return" | "space"
	| "leftArrow" | "rightArrow" | "upArrow" | "downArrow"
	| "pageUp" | "pageDown" | "home" | "end"
	| "leftShift" | "rightShift" | "leftControl" | "rightControl"
	| "leftOption" | "rightOption" | "leftCommand" | "rightCommand"
	| "globe"
	| "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m"
	| "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z"
	| "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "0"
	| "grave" | "leftBracket" | "rightBracket" | "backslash"
	| "semicolon" | "quote" | "minus" | "equal" | "comma" | "period" | "slash" | "section"
	| "numpad0" | "numpad1" | "numpad2" | "numpad3" | "numpad4"
	| "numpad5" | "numpad6" | "numpad7" | "numpad8" | "numpad9"
	| "numpadMultiply" | "numpadAdd" | "numpadSubtract" | "numpadDivide"
	| "numpadEnter" | "numpadDecimal" | "numpadClear" | "numpadComma"
	| "volumeUp" | "volumeDown" | "mute" | "yen" | "underscore"
	| "eisu" | "kana" | "menu" | "help" | "forwardDelete" | "power"
	| "missionControl" | "enterPowerbook";

type KeyStr = KeyName | ModifierStr;

export class Key {
	static #getModifierFlag(key: string, _modifiers: number) {
		switch (key) {
			case "caps":
			case "capsLock":
			case "alphaShift":
				return Modifier.CapsLock;
			case "shift":
				return Modifier.Shift;
			case "control":
			case "ctrl":
				return Modifier.Control;
			case "option":
			case "alternate":
			case "opt":
			case "alt":
				return Modifier.Option;
			case "command":
			case "cmd":
				return Modifier.Command;
			case "help":
				return Modifier.Help;
			case "secondaryFn":
				return Modifier.SecondaryFn;
			case "numpad":
				return Modifier.Numpad;
		}
	}

	#key;
	#modifiers = 0;
	#application?: AppHandle;

	set application(appHandle: AppHandle) {
		this.#application = appHandle;
	}

	constructor(...args: (KeyStr | (string & {}))[]) {
		if (args.length === 1) args = args[0]!.split("+");

		let mainKey;
		let modifiers = 0;
		for (let i = 0; i < args.length; i++) {
			const key = args[i];
			if (key === "") continue;
			if (mainKey === undefined && i === args.length - 1) {
				// Modifiers only; set last modifier as main key (because we need to have one)
				mainKey = key;
			}
			const flag = Key.#getModifierFlag(key!, modifiers);
			if (flag === undefined) mainKey = key;
			else modifiers |= flag; // e.g. modifiers = modifiers | CapsLock
		}

		this.#key = mainKey;
		this.#modifiers = modifiers;
	}

	down = async (application?: AppHandle) => {
		const appHandle = this.#application ?? application;
		if (appHandle == null) {
			throw new Error("Key is not associated with an application, and no application was provided as an argument");
		}
		return rpc.keyboardKeyDown(appHandle, this.#key!, this.#modifiers);
	};

	up = async (application?: AppHandle) => {
		const appHandle = this.#application ?? application;
		if (appHandle == null) {
			throw new Error("Key is not associated with an application, and no application was provided as an argument");
		}
		return rpc.keyboardKeyUp(appHandle, this.#key!, this.#modifiers);
	};

	press = async (application?: AppHandle) => {
		const appHandle = this.#application ?? application;
		if (appHandle == null) {
			throw new Error("Key is not associated with an application, and no application was provided as an argument");
		}
		return rpc.keyboardKeyPress(appHandle, this.#key!, this.#modifiers);
	};
}

export class ScrollWheel {
	#application?: AppHandle;

	set application(appHandle: AppHandle) {
		this.#application = appHandle;
	}

	// delta = lines. positive = natural direction (down/right), negative = up/left.
	y = async (delta: number, application?: AppHandle) => {
		const appHandle = this.#application ?? application;
		if (appHandle == null) throw new Error("No application — set .application or pass one");
		return rpc.scrollWheelY(appHandle, delta);
	};

	x = async (delta: number, application?: AppHandle) => {
		const appHandle = this.#application ?? application;
		if (appHandle == null) throw new Error("No application — set .application or pass one");
		return rpc.scrollWheelX(appHandle, delta);
	};
}

export class Var {
	#name: string;
	#handle?: VarHandle;
	#value = false;

	get value() {
		return this.#value;
	}

	set value(b: boolean) {
		if (this.#handle == null) {
			console.error("Var not declared, cannot set value", { name: this.#name });
			return;
		}

		// This async setter is fine because setting a var should NEVER fail. If it does, we got problems.
		let old = this.#value;
		this.#value = b;
		rpc.setVar(this.#handle, b).catch(e => {
			console.error("Failed to set var value", { name: this.#name, value: b, error: e });
			this.#value = old;
		});
	}

	constructor(name: string, init = false) {
		this.#name = name;

		// This async constructor is fine because setting a var should NEVER fail. If it does, we got problems.
		rpc.declareVar(name).then(handle => {
			if (handle == null) return console.error("Failed to declare var", { handle });

			this.#handle = handle;
			this.value = init;
		});
	}
}

export type Vars = {
	new <Init extends Record<string, boolean>>(init: Init): Record<keyof Init, boolean>;
};

export function Vars<Init extends Record<string, boolean>>(init: Init): Record<keyof Init, boolean> {
	const vars = new Map<string, Var>();

	for (const name in init) vars.set(name, new Var(name, init[name]!));

	return new Proxy(vars, {
		get(target, prop) {
			if (typeof prop !== "string") return (target as any)[prop];
			return target.get(prop as any)!.value;
		},
		set(target, prop, value) {
			if (typeof prop !== "string") {
				throw new Error(
					`Attempted to set Vars.${String(prop)} to ${value}, but key ${String(prop)} is of type ${typeof prop}. Vars only supports string keys.k`,
				);
			}

			target.get(prop as any)!.value = value;

			return true;
		},
	}) as any as typeof init;
}

// MatchString fields accept raw strings for ergonomics — transformStep wraps them as { glob: value }.
type FilterStepFields = { [K in keyof Omit<Filters, "has">]?: Filters[K] extends MatchString ? string | MatchString : Filters[K] };
export type FilterStep = Extract<Filter, string> | (FilterStepFields & { has?: FilterStep });

export class Element {
	#handle: ElementHandle;

	static dispose = (handle: ElementHandle) => rpc.disposeElement(handle);

	static #finalizationRegistry = new FinalizationRegistry<ElementHandle>(Element.dispose);

	static transformStep(step: FilterStep): Filter[] {
		if (typeof step === "string") return [step satisfies Filter];
		return Object.entries(step).map(([key, value]) => {
			let v;
			if (typeof value === "string") v = { glob: value };
			else if (typeof value === "boolean") v = value;
			else if (typeof value === "object" && value !== null && ("glob" in value || "literal" in value)) v = value;
			else v = Element.transformStep(value!);
			return { [key]: v } as unknown as Filter;
		});
	}

	static formatPath(path: FilterStep[]): string {
		return path.map(step => (typeof step === "string" ? step : JSON.stringify(step))).join(" -> ");
	}

	constructor(handle: ElementHandle) {
		if (typeof handle !== "number") throw new Error("Element handle must be a number");

		this.#handle = handle;

		// When `this` gets garbage collected, dispose the element handle in Rust.
		// If already disposed, does nothing - each handle is pretty much unique (technically it can loop around after `u32::MAX` handles)
		// and Rust will skip disposing.
		Element.#finalizationRegistry.register(this, this.#handle);
	}

	get handle() {
		return this.#handle;
	}

	walk = async (...path: FilterStep[]) => {
		const filterPath = path.map(Element.transformStep);

		// The first argument, this.#handle.value, is the existing handle (if any).
		// If it's provided/not null, rust will update that handle instead of creating a new one.
		//
		// ...because what a handle actually points to should be fluid. The underlying element may change at any time.
		//
		// It's the same for AppHandle: the underlying process and PID may change, e.g. after quitting and relaunching the app,
		// but the AppHandle stays the same. We swap out the underlying NSRunningApplication for the new one.
		const foundElementHandle = await rpc.walkElement(this.#handle, filterPath);
		if (!foundElementHandle) return null;

		return new Element(foundElementHandle);
	};

	on = (notificationName: string, callback: (event: AccessibilityEvent) => void | Promise<void>): (() => Promise<void>) => {
		let registrationHandle: NotificationRegistrationHandle | null = null;

		const fetchHandle = async () => {
			const handle = await rpc.observeElementNotification(this.#handle, notificationName);
			registrationHandle = handle;
			rpc.registerAccessibilityCallback(handle, callback as (event: unknown) => any);
		};

		fetchHandle().catch(e => {
			console.error(`[Element.on] Error registering AX notification`, notificationName, e);
		});

		return async () => {
			if (registrationHandle === null) return;
			await rpc.unobserveElementNotification(registrationHandle);
			rpc.unregisterAccessibilityCallback(registrationHandle);
		};
	};

	getAttribute = <T = RevivedJsonValue>(attr: string) => rpc.getElementAttribute<T>(this.#handle, attr as Attribute);
	setAttribute = (attr: string, value: string | number | boolean) => rpc.setElementAttribute(this.#handle, attr as Attribute, value);
	runAction = (action: string) => rpc.performElementAction(this.#handle, action);

	attribute = new Proxy(this, {
		get: (target, prop) => (typeof prop === "string" ? target.getAttribute(prop) : undefined),
	}) as any as Record<Extract<Attribute, string>, Promise<RevivedJsonValue>>;

	action = new Proxy(this, {
		get: (target, prop) => (typeof prop === "string" ? () => target.runAction(prop) : undefined),
	}) as any as Record<string, () => Promise<void>>;

	// /**
	//  * Listen for notifications on the element
	//  */
	// on = new Proxy(
	// 	{
	// 		get: (target, prop) => {
	// 			if (typeof prop !== "string") return target[prop as keyof typeof target];

	// 			switch (prop) {
	// 				case "uiElementDestroyed":
	// 					prop = "AXUIElementDestroyed";
	// 					break;
	// 				default:
	// 					prop = `AX${prop[0]!.toUpperCase()}${prop.slice(1)}`;
	// 					break;
	// 			}

	// 			return (callback: (event: any) => void) => target(prop, callback); // target = #listen
	// 		},
	// 	}
	// ) as any as {
	// 	[K in keyof DefaultNotificationInfos]: (
	// 		callback: (event: AccessibilityEvent<K extends keyof DefaultNotificationInfos ? DefaultNotificationInfos[K] : undefined>) => any
	// 	) => void;
	// };

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

	[Symbol.asyncDispose] = () => Element.dispose(this.#handle);
	[Symbol.dispose] = () => this[Symbol.asyncDispose]();
}

export class Pack {
	#handle: PackHandle;

	static register(publisherDomain: string, packName: string): Promise<PackHandle> {
		return Promise.resolve({ publisherDomain, packName });
	}

	static async init(publisherDomain: string, packName: string) {
		return new Pack(await this.register(publisherDomain, packName));
	}

	constructor(handle: PackHandle) {
		this.#handle = handle;
	}

	get handle() {
		return this.#handle;
	}

	run(functionName: string, payload?: unknown): Promise<RevivedJsonValue | undefined> {
		return rpc.runFunction(this.#handle, functionName, payload);
	}
}

export class App {
	#handle: AppHandle;
	#element?: Promise<Element>;

	onactivate?: () => void | Promise<void>;
	ondeactivate?: () => void | Promise<void>;
	onterminate?: () => void | Promise<void>;

	get element() {
		return this.#element;
	}

	static register(bundleIdentifier: string): Promise<AppHandle> {
		return rpc.registerApp(bundleIdentifier);
	}

	static async init(bundleIdentifier: string) {
		return new App(await this.register(bundleIdentifier));
	}

	constructor(handle: AppHandle) {
		this.#handle = handle;
		rpc.registerAppInstance(handle, this);

		this.#element = rpc.registerPendingAppElement(handle, elementHandle => new Element(elementHandle));

		// eagerly try — if app is already active, resolve immediately
		// instead of waiting for workspaceAppActivated which won't fire
		rpc.resolvePendingAppElement(handle).catch(() => {});
	}

	get handle() {
		return this.#handle;
	}

	key = Object.assign(
		(...args: ConstructorParameters<typeof Key>) => {
			const key = new Key(...args);
			key.application = this.#handle;
			return key;
		},
		{
			down: (...args: ConstructorParameters<typeof Key>) => new Key(...args).down(this.#handle),
			up: (...args: ConstructorParameters<typeof Key>) => new Key(...args).up(this.#handle),
			press: (...args: ConstructorParameters<typeof Key>) => new Key(...args).press(this.#handle),
		},
	);

	scroll = Object.assign(
		() => {
			const sw = new ScrollWheel();
			sw.application = this.#handle;
			return sw;
		},
		{
			y: (delta: number) => new ScrollWheel().y(delta, this.#handle),
			x: (delta: number) => new ScrollWheel().x(delta, this.#handle),
		},
	);

	[Symbol.asyncDispose] = () => {
		rpc.unregisterAppInstance(this.#handle);
	};
	[Symbol.dispose] = () => this[Symbol.asyncDispose]();
}

export type API = {
	Pack: typeof Pack;
	Function: typeof Function;
	App: typeof App;
	Element: typeof Element;
	Key: typeof Key;
	ScrollWheel: typeof ScrollWheel;
	Modifier: typeof Modifier;
	Var: typeof Var;
	Vars: typeof Vars;
	View: typeof View;
	action: typeof action;
};
