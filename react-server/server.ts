import { registerClientReference, registerServerReference, renderToReadableStream } from "react-server-dom-webpack/server";
import { Flight as Base } from "./flight";

class ServerFlight extends Base {
	#prefix: string;

	constructor(modules: Record<string, Record<string, any>>, prefix: string) {
		super(modules);
		this.#prefix = prefix;
	}

	#actions = new Map<string, Function>();
	#counter = 0;
	action<T extends (...args: any[]) => any>(fn: T): T {
		const id = `${this.#prefix}:${this.#counter++}`;
		this.#actions.set(id, fn);
		registerServerReference(fn, id, null);
		return fn;
	}

	execute(id: string, args: any[]): Promise<any> {
		const fn = this.#actions.get(id);
		if (!fn) throw new Error(`Unknown action "${id}" — registered: [${[...this.#actions.keys()]}]`);
		return fn(...args);
	}

	serialize(element: any): Promise<string> {
		return new Response(renderToReadableStream(element, this.#manifest)).text();
	}

	#cachedManifest: Record<string, { id: string; chunks: string[]; name: string }> | null = null;
	get #manifest() {
		if (!this.#cachedManifest) {
			const m: Record<string, { id: string; chunks: string[]; name: string }> = {};
			for (const id of Object.keys(this.modules)) m[id] = { id, chunks: [], name: "" };
			this.#cachedManifest = m;
		}
		return this.#cachedManifest;
	}
}

export { ServerFlight as Flight };

const ClientReference = Symbol("ClientReference");
export type ClientReference = typeof ClientReference;

export function references<T extends Record<string, any>>(moduleId: string, ...names: Array<keyof T>): Record<keyof T, ClientReference> {
	const mod = {} as Record<keyof T, ClientReference>;
	for (const name of names) mod[name] = registerClientReference({}, moduleId, name) as ClientReference;
	return mod;
}
