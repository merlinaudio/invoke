declare global {
	type Key = import("invoke").Key;
	type Modifier = import("invoke").Modifier;
	type Var = import("invoke").Var;
	type Function = import("invoke").Function;
	type App = import("invoke").App;
	type Pack = import("invoke").Pack;
	type FilterStep = import("invoke").FilterStep;
	type View = import("invoke").View;

	var App: typeof import("invoke").App;
	var Pack: typeof import("invoke").Pack;
	var Function: typeof import("invoke").Function;
	var Key: typeof import("invoke").Key;
	var ScrollWheel: typeof import("invoke").ScrollWheel;
	var Modifier: typeof import("invoke").Modifier;
	var Var: typeof import("invoke").Var;
	var Vars: import("invoke").Vars;
	var View: typeof import("invoke").View;
	var action: typeof import("invoke").action;

	type AccessibilityEvent<Info = undefined> = import("invoke").AccessibilityEvent<Info>;
}

// Make this file a module so `import "./globals"` activates the declare global block.
export {};
