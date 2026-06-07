import { Flight as Base } from "./flight";

const registry: Record<string, Record<string, any>> = {};

_require.u = (chunkId: string) => chunkId;
function _require(id: string) {
	const mod = registry[id];
	if (!mod) throw new Error(`Flight: unknown module "${id}"`);
	return mod;
}

(globalThis as any).__webpack_require__ = _require;
(globalThis as any).__webpack_chunk_load__ = () => Promise.resolve();

const { createFromReadableStream } = await import("react-server-dom-webpack/client.browser");

class ClientFlight extends Base {
	constructor(modules: Record<string, Record<string, any>>) {
		super(modules);
		for (const [id, exports] of Object.entries(modules)) registry[id] = exports;
	}

	deserialize(payload: string, opts?: { callServer?: (id: string, args: any) => Promise<any> }): Promise<any> {
		return createFromReadableStream(new Response(payload).body!, { callServer: opts?.callServer });
	}
}

export { ClientFlight as Flight };
