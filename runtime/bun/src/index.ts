import "./register";

import path from "node:path";
import { writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";
import { App, Function as PackFunction, View } from "./globals";
import { AppDelegate } from "./modules/invoke";
import { connect, ready } from "./rpc";

// The launcher passes two positional arguments: the socket path to connect to,
// then the pack root. Bun's compiled-executable argv puts the first real
// argument at index 2.
const socketPath = Bun.argv[2];
if (socketPath == null) throw new Error("Pack process expected a socket path as its first argument");

// Connect to the host before importing user code. The host disconnecting ends
// the pack — there is no graceful shutdown.
await connect(socketPath, () => process.exit(0));

const packRoot = Bun.argv[3];
if (packRoot == null) throw new Error("Pack process expected a pack directory as its second argument");

const root = path.resolve(packRoot);

// The host starts sandboxed Bun from "/" so Bun finishes its own startup before
// it touches pack-root ancestors. Pack code should still get its own root as
// cwd, so move before importing any user JavaScript.
process.chdir(root);

await assertSeatbeltDeniesParentWrite(root);

const packModule = await import(pathToFileURL(await resolvePackEntrypoint(root)).href);

await getSingleExportedApp(packModule).then(app => app && registerExportedFunctions(app, packModule));

// Initial load and function registration are done; let the host know the pack
// is ready to have its functions run.
await ready();

async function getSingleExportedApp(packModule: Record<string, any>) {
	const apps = Object.values(packModule).filter(value => value instanceof AppDelegate || value instanceof App);
	if (apps.length !== 1) {
		if (apps.length > 1) console.warn(`[pack] skipped exported function registration: expected exactly one exported app, found ${apps.length}`);
		return;
	}
	return apps[0]!;
}

/**
 * If one (and only one) App or AppDelegate was exported, all functions exported
 * by the passed-in pack esm will be registered as functions.
 */
async function registerExportedFunctions(app: App | AppDelegate, packModule: Record<string, any>) {
	await Promise.all(
		Object.entries(packModule).map(async ([name, value]) => {
			if (typeof value !== "function") return;

			let view: View | undefined;
			if (Object.hasOwn(value, "View")) {
				if (value.View instanceof View) view = value.View;
				else if (typeof value.View === "function") view = new View(value.View);
				else console.error(`[pack] invalid View for function "${name}"`, value.View, typeof value.View);
			}

			await PackFunction.init(app.handle, name, value, value.end, view).catch(error => {
				console.error(`[pack] error initializing function "${name}"`, error);
			});
		}),
	);
}

async function resolvePackEntrypoint(packRoot: string) {
	for (const name of ["index.js", "index.ts", "index.tsx"]) {
		const entrypoint = path.join(packRoot, name);
		if (await Bun.file(entrypoint).exists()) return entrypoint;
	}

	throw new Error(`Pack entrypoint not found in ${packRoot}`);
}

async function assertSeatbeltDeniesParentWrite(packRoot: string) {
	const parent = path.dirname(packRoot);
	const deniedPath = path.join(parent, `.invoke-sandbox-write-test-${process.pid}`);

	console.debug(`[sandbox] checking parent write denial: ${deniedPath}`);

	try {
		await writeFile(deniedPath, "Invoke sandbox write probe\n");
	} catch (error) {
		if (error && typeof error === "object" && "code" in error && error.code === "EPERM") {
			console.debug("[sandbox] parent write denied by SBPL as expected");
			return;
		}

		console.debug("[sandbox] parent write probe failed for an unexpected reason", error);
		throw error;
	}

	console.debug("[sandbox] parent write probe unexpectedly succeeded");
	throw new Error(`Pack sandbox is not enforcing the expected SBPL profile; wrote outside the pack root: ${deniedPath}`);
}
