#!/usr/bin/env bun

import { rm, readdir } from "node:fs/promises";
import { dirname, join } from "node:path";

const root = dirname(import.meta.dir); // runtime/bun
const outdir = join(root, ".build", "pack-runtime");

await rm(outdir, { recursive: true, force: true });

const result = await Bun.build({
	entrypoints: [join(root, "src", "index.ts")],
	compile: {
		outfile: join(outdir, "invoke-pack-runtime"),
		autoloadBunfig: false,
		autoloadDotenv: false,
		autoloadPackageJson: false,
		autoloadTsconfig: false,
		target: "bun-darwin-arm64",
	},
	conditions: ["react-server"],
	// Bake production React into the compiled binary. Without this, React's
	// dev/prod branch (process.env.NODE_ENV) stays a runtime lookup; the shipped
	// app spawns this binary without NODE_ENV=production, so the RSC server picks
	// the development build while the client view bundle (built under
	// NODE_ENV=production) picks production — a fatal dev/prod payload mismatch.
	define: { "process.env.NODE_ENV": JSON.stringify("production") },
	minify: true,
});

if (!result.success) {
	console.error("Failed to build pack runtime:", result.logs);
	process.exit(1);
}

console.log(`build-pack-runtime outputs:\n${result.outputs.map(o => `\t${o.path} [${~~(o.size / 1024)} kB]`).join("\n")}`);

const entries = await readdir(outdir);
if (entries.length !== 1 || entries[0] !== "invoke-pack-runtime") {
	throw new Error(`Pack runtime build expected exactly one invoke-pack-runtime file, got: ${entries.join(", ") || "<none>"}`);
}
