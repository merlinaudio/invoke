// Why .ts and not .d.ts: this file is declaration-only, but it re-exports from
// source files under src/ that contain real runtime code (class bodies, polyfills,
// etc.). rollup-plugin-dts knows how to compile those down to pure types — but
// only when its entry has a .ts extension. A .d.ts entry would make the plugin
// reject the runtime constructs it encounters while walking the import graph.

/// <reference path="./globals.d.ts" />

export * from "../../src/globals";
export * from "../../src/modules/invoke";
