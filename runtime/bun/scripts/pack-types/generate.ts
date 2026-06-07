import { mkdir, rm } from "node:fs/promises";
import { rollup } from "rollup";
import { packTypesConfig } from "./rollup.config.mjs";

const here = import.meta.dir;
const root = `${here}/../..`;
const outdir = Bun.argv[2] ?? `${root}/.build/pack-types`;
const pkgdir = `${outdir}/node_modules/invoke`;

await rm(outdir, { recursive: true, force: true });
await mkdir(pkgdir, { recursive: true });

for (const options of packTypesConfig(pkgdir)) {
	const bundle = await rollup(options);
	const outputs = Array.isArray(options.output) ? options.output : [options.output];

	try {
		await Promise.all(outputs.map(bundle.write));
	} finally {
		await bundle.close();
	}
}

await Bun.write(`${pkgdir}/package.json`, await Bun.file(`${here}/package.json`).text());
await Bun.write(`${pkgdir}/globals.d.ts`, await Bun.file(`${here}/globals.d.ts`).text());

console.log(outdir);
