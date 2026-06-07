import { plugin } from "bun";
import { App, Element, Function, Key, Modifier, Pack, ScrollWheel, Var, Vars, View, action, type API } from "./globals";
import * as invoke from "./modules/invoke";
import * as invokeUi from "./modules/invoke/ui";

plugin({
	name: "invoke/ui",
	setup(build) {
		build.module("invoke", () => ({ exports: invoke, loader: "object" }));
		build.module("invoke/ui", () => ({ exports: invokeUi, loader: "object" }));
	},
});

Object.assign(globalThis, {
	Pack,
	App,
	Function,
	Element,
	Modifier,
	Key,
	ScrollWheel,
	Var,
	Vars,
	View,
	action,
} satisfies API);
