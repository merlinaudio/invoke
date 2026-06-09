import { route, router } from "@merlin.audio/router";
import type { AppHandle, Attribute, ElementHandle } from "./proto";

// Opaque pack-protocol handles — libinvoke's proto carries these inline as `number`.
type VarHandle = number;
type NotificationRegistrationHandle = number;
import { start, type Call } from "./transport";
import type { HostHandlers } from "./proto";
import { id, renderRegisteredView, runRegisteredAction } from "./flight";
import { Element, type PackHandle } from "./globals";

// `route` with a single handler types its input as `undefined`. The wire hands
// each handler a named-args object, so route through an identity validator.
const handler = <In, Out>(fn: (args: In) => Out) => route((args: unknown) => args as In, fn);

// The pack runtime's RPC layer. The handler set below answers inbound requests
// from the host (`runFunction`, `renderView`, …); the `host` proxy and the
// positional wrappers below make outbound calls (`declareVar`, `walkElement`, …).
// `transport.ts` owns the socket and the wire; this module owns the contract —
// method names and arg shapes match `proto.rs` (`HostHandlers`/`PackHandlers`),
// with the outbound shapes type-checked against the generated `proto.d.ts`.

// The host marks elements on the wire as `{"#e": handle}`. `serialize` writes
// that marker for an outbound `Element`; `reviver` — handed to the transport's
// `JSON.parse` — rebuilds `Element`s from it on every inbound line.

function serialize(value: unknown) {
	return JSON.stringify(value, (_key, value) => {
		if (value && typeof value === "object" && "handle" in value && typeof value.handle === "number") return { "#e": value.handle };
		return value;
	});
}

const reviver = (_key: string, value: any) => (value && typeof value === "object" && "#e" in value ? new Element(value["#e"]) : value);

function deserialize(value: unknown) {
	if (typeof value !== "string") throw new TypeError("Expected serialized string");
	return JSON.parse(value, reviver);
}

type FunctionHandle = number;

type FunctionInstance = {
	run(payload: unknown): unknown | Promise<unknown>;
	end(): void | Promise<void>;
};

type AppInstance = {
	onactivate?: () => void | Promise<void>;
	ondeactivate?: () => void | Promise<void>;
	onterminate?: () => void | Promise<void>;
};

type Deferred<T> = {
	resolve: (value: T) => void;
	reject: (error: any) => void;
	promise: Promise<T>;
};

const functions = new Map<FunctionHandle, FunctionInstance>();
const accessibilityCallbacks = new Map<number, (event: unknown) => any>();
const apps = new Map<AppHandle, AppInstance>();
const pendingAppElements = new Map<AppHandle, Deferred<ElementHandle>>();

export function registerFunction(handle: FunctionHandle, fn: FunctionInstance) {
	functions.set(handle, fn);
}

export function registerAccessibilityCallback(handle: NotificationRegistrationHandle, callback: (event: unknown) => any) {
	accessibilityCallbacks.set(handle, callback);
}

export function unregisterAccessibilityCallback(handle: NotificationRegistrationHandle) {
	accessibilityCallbacks.delete(handle);
}

export function registerAppInstance(handle: AppHandle, app: AppInstance) {
	apps.set(handle, app);
}

export function unregisterAppInstance(handle: AppHandle) {
	apps.delete(handle);
}

export function registerPendingAppElement<T>(appHandle: AppHandle, revive: (handle: ElementHandle) => T) {
	const pending = Promise.withResolvers<ElementHandle>();
	pendingAppElements.set(appHandle, pending);
	return pending.promise.then(revive);
}

export async function resolvePendingAppElement(appHandle: AppHandle) {
	const pending = pendingAppElements.get(appHandle);
	if (pending == null) return console.debug(`[resolvePendingAppElement] no pending app element for app ${appHandle}`);

	const element = await getAppElement(appHandle);
	if (element == null) throw new Error(`Failed to get app element for app ${appHandle}`);

	pending.resolve(element);
	pendingAppElements.delete(appHandle);
}

// ---------------------------------------------------------------------------------------------------------------------
// Inbound — requests the host sends us (`PackHandlers` in proto.rs).
// ---------------------------------------------------------------------------------------------------------------------

const handlers = router().routes({
	runFunction: handler(async ({ function: functionHandle, payload }: { function: number; payload: unknown }) => {
		const instance = functions.get(functionHandle);
		if (!instance) {
			console.warn(`[pack] runFunction: no function found for ${functionHandle}`);
			return undefined;
		}
		return serialize(await instance.run(deserialize(payload)));
	}),

	endFunction: handler(({ function: functionHandle }: { function: number }) => {
		const instance = functions.get(functionHandle);
		if (!instance) return console.warn(`[pack] endFunction: no function found for ${functionHandle}`);
		instance.end();
	}),

	accessibilityNotification: handler(async ({ notification, event }: { notification: number; event: unknown }) => {
		try {
			await accessibilityCallbacks.get(notification)?.(event);
		} catch (e) {
			console.warn("AX callback error", e);
		}
	}),

	workspaceAppActivated: handler(async ({ app }: { app: number }) => {
		console.log(`App ${app} activated`);

		resolvePendingAppElement(app).catch(e => console.error(`[workspaceAppActivated] Error resolving pending app element for app ${app}`, e));

		await apps.get(app)?.onactivate?.();
	}),

	workspaceAppDeactivated: handler(async ({ app }: { app: number }) => {
		console.log(`App ${app} deactivated`);

		Bun.gc(true);

		resolvePendingAppElement(app).catch(e => console.error(`[workspaceAppDeactivated] Error resolving pending app element for app ${app}`, e));

		await apps.get(app)?.ondeactivate?.();
	}),

	workspaceAppTerminated: handler(async ({ app }: { app: number }) => {
		console.log(`App ${app} terminated`);
		await apps.get(app)?.onterminate?.();
	}),

	renderView: handler(({ view }: { view: number }) => renderRegisteredView(view)),

	runViewAction: handler(({ actionId, args, view }: { actionId: string; args: unknown; view: number }) => runRegisteredAction(actionId, args, view)),
});

// ---------------------------------------------------------------------------------------------------------------------
// Transport — set once by `connect()`; the outbound functions below late-bind
// through it. `id` (the pack's random self-id) is logged on connect.
// ---------------------------------------------------------------------------------------------------------------------

let call: Call;

export async function connect(socketPath: string, onClose: () => void) {
	call = await start(socketPath, handlers, reviver, onClose);
	console.log(`[pack] connected: ${id}`);
}

// Typed proxy over the host contract — `host.setVar({ var, value })`, mirroring
// the renderer's `api.*`. Method→args shapes come from libinvoke's generated
// `proto.d.ts` (`HostHandlers`), so a wrong arg name/type is a compile error. The
// wire encodes requests but not responses, so the result is opaque (`unknown`);
// the positional wrappers below cast it to the concrete handle types.
type Defined<T> = { [K in keyof T as T[K] extends undefined ? never : K]: T[K] };
type UnionToIntersection<U> = (U extends unknown ? (k: U) => void : never) extends (k: infer I) => void ? I : never;
type Methods = UnionToIntersection<HostHandlers extends infer M ? (M extends unknown ? Defined<M> : never) : never>;
export type Host = { [M in keyof Methods]: (args: Methods[M]) => Promise<unknown> };

export const host = new Proxy({}, { get: (_t, method: string) => (args: unknown) => call(method, args) }) as Host;

// ---------------------------------------------------------------------------------------------------------------------
// Outbound — calls the pack makes to the host (`HostHandlers` in proto.rs).
// The pack's public API (globals.ts) is positional; the wire is named-args, so
// each function marshals its positional arguments into the named-arg object.
// ---------------------------------------------------------------------------------------------------------------------

export const registerApp = (bundleIdentifier: string) => call("registerApp", { bundleIdentifier }) as Promise<AppHandle>;

// Tell the host the pack finished its initial load and function registration.
export const ready = () => call("ready", {}) as Promise<void>;

export const defineFunction = (functionName: string, view: number) =>
	call("defineFunction", { functionName, view }) as Promise<FunctionHandle | null>;

export const declareVar = (name: string) => call("declareVar", { name }) as Promise<VarHandle | null>;
export const setVar = (varHandle: VarHandle, value: boolean) => call("setVar", { var: varHandle, value }) as Promise<void>;

export const keyboardKeyDown = (app: AppHandle, key: string, modifiers: number) => call("keyboardKeyDown", { app, key, modifiers }) as Promise<void>;
export const keyboardKeyUp = (app: AppHandle, key: string, modifiers: number) => call("keyboardKeyUp", { app, key, modifiers }) as Promise<void>;
export const keyboardKeyPress = (app: AppHandle, key: string, modifiers: number) =>
	call("keyboardKeyPress", { app, key, modifiers }) as Promise<void>;

export const scrollWheelY = (app: AppHandle, delta: number) => call("scrollWheelY", { app, delta }) as Promise<void>;
export const scrollWheelX = (app: AppHandle, delta: number) => call("scrollWheelX", { app, delta }) as Promise<void>;

export const getAppElement = (app: AppHandle) => call("getAppElement", { app }) as Promise<ElementHandle | null>;

export const walkElement = (root: ElementHandle, filterPath: unknown) => call("walkElement", { root, filterPath }) as Promise<ElementHandle | null>;
export const disposeElement = (element: ElementHandle) => call("disposeElement", { element }) as Promise<void>;
export const performElementAction = (element: ElementHandle, action: string) => call("performElementAction", { element, action }) as Promise<void>;
export const setElementAttribute = (element: ElementHandle, attribute: Attribute, value: string | number | boolean) =>
	call("setElementAttribute", { element, attribute, value }) as Promise<void>;

export const observeElementNotification = (element: ElementHandle, notificationName: string) =>
	call("observeElementNotification", { element, notificationName }) as Promise<NotificationRegistrationHandle>;
export const unobserveElementNotification = (notification: NotificationRegistrationHandle) =>
	call("unobserveElementNotification", { notification }) as Promise<void>;

export const getElementAttribute = <T = unknown>(element: ElementHandle, attribute: Attribute) =>
	call("getElementAttribute", { element, attribute }) as Promise<T>;

export async function runFunction(handle: PackHandle, functionName: string, payload?: unknown) {
	const result = await call("runFunction", {
		publisherDomain: handle.publisherDomain,
		packName: handle.packName,
		functionName,
		payload: serialize(payload ?? null),
	});
	return result == null ? undefined : deserialize(result);
}
