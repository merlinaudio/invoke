import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import dts from "rollup-plugin-dts";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../..");

export function packTypesConfig(outdir = resolve(root, ".build/pack-types/node_modules/invoke")) {
	return [
		{
			input: resolve(here, "invoke.ts"),
			external: ["invoke", "react", "./globals"],
			plugins: [dts()],
			output: {
				file: resolve(outdir, "invoke.d.ts"),
				format: "es",
				banner: '/// <reference path="./globals.d.ts" />\n',
			},
		},
		{
			input: resolve(here, "invoke-ui.ts"),
			external: ["react"],
			plugins: [dts()],
			output: { file: resolve(outdir, "invoke-ui.d.ts"), format: "es" },
		},
	];
}

export default packTypesConfig();
