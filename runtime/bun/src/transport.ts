import type { RouterConfig } from "@merlin.audio/router";
import { once } from "node:events";
import { createConnection } from "node:net";
import { createInterface } from "node:readline";

// The pack wire protocol — one JSON value per line, both directions.
//
//   request    {"id":1,"declareVar":{"name":"theme"}}
//   event      {"runFunction":{"function":3,"payload":null}}
//   response   [1,0,7]
//   response   [1,-1,"no such element"]
//
// An array is a response — [id, status, body], status OK (0) or ERROR (-1). An
// object is a request — {method: args}, with an id when it expects a response;
// no id means a fire-and-forget event. proto.rs is the reference.

const OK = 0;
const ERROR = -1;

/**
 * Outbound `call`s awaiting their response, keyed by a wire id — the
 * TypeScript mirror of `Pending` on the Rust side. `issue` parks a call and
 * puts the request on the wire; `complete`/`fail` settle one by id; `failAll`
 * settles every parked call at once when the connection drops.
 */
class Pending {
	#next = 0;
	#waiting = new Map<number, PromiseWithResolvers<unknown>>();

	// Reserve an id, hand it to `send` to put the request on the wire, and
	// resolve once the response carrying that id comes back.
	issue(send: (id: number) => void): Promise<unknown> {
		const id = this.#next++;
		const slot = Promise.withResolvers<unknown>();
		this.#waiting.set(id, slot);
		send(id);
		return slot.promise;
	}

	complete(id: number, body: unknown) {
		this.#take(id)?.resolve(body);
	}

	fail(id: number, error: string) {
		this.#take(id)?.reject(new Error(error));
	}

	failAll() {
		for (const { reject } of this.#waiting.values()) reject(new Error("closed"));
		this.#waiting.clear();
	}

	// Claim the slot waiting on `id`, removing it — a call settles at most once.
	#take(id: number) {
		const slot = this.#waiting.get(id);
		this.#waiting.delete(id);
		return slot;
	}
}

/** Issue one request to the host and await its response. */
export type Call = (method: string, args: unknown) => Promise<unknown>;

/**
 * Connect to the host over `socketPath` and serve the pack wire protocol.
 * Inbound requests route into `router` by method name; inbound responses
 * settle the matching call. `reviver` is passed to every inbound `JSON.parse`;
 * `onClose` runs once the host disconnects. Returns the outbound [`Call`].
 */
export async function start(
	socketPath: string,
	router: RouterConfig<unknown, unknown, unknown>,
	reviver: (key: string, value: any) => any,
	onClose?: () => void,
): Promise<Call> {
	const socket = createConnection(socketPath);
	socket.on("error", error => console.error("[pack] socket error:", error));
	await once(socket, "connect");

	const pending = new Pending();
	const write = (message: unknown) => socket.write(JSON.stringify(message) + "\n");

	// Handle one inbound request — {id?, method: args} — by routing it through
	// `router`, then writing [id, status, body] back if it expects a response.
	async function dispatch({ id, ...rest }: Record<string, any>) {
		const method = Object.keys(rest)[0]!;
		const args = rest[method];
		const respond = (status: number, body: unknown) => {
			if (id != null) write([id, status, body]);
		};

		const route = router.routes[method];
		if (!route) return respond(ERROR, `MethodNotFound:${method}`);

		try {
			const ctx = route.context?.(args);
			const input = route.validate(args);
			const output = await route.handler(await input, await ctx);
			respond(OK, output);
		} catch (error) {
			respond(ERROR, error instanceof Error ? error.message : String(error));
		}
	}

	// Demux one inbound line: an array is a response, an object a request.
	function receive(line: string) {
		let message: any;
		try {
			message = JSON.parse(line, reviver);
		} catch (error) {
			return console.error("[pack] undecodable line:", error);
		}
		if (Array.isArray(message)) {
			const [id, status, body] = message;
			status === OK ? pending.complete(id, body) : pending.fail(id, String(body));
		} else {
			dispatch(message);
		}
	}

	// The socket is a raw byte stream; readline reframes it into lines.
	createInterface({ input: socket }).on("line", receive);

	socket.on("close", () => {
		pending.failAll();
		onClose?.();
	});

	return (method, args) => pending.issue(id => write({ id, [method]: args }));
}
