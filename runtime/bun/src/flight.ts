import { Flight, references } from "@merlin.audio/react-flight/server";
import type { CSSProperties, ReactNode } from "react";

export type InvokeUI = {
	Button: { action: (...args: any[]) => Promise<any>; children: ReactNode; style?: CSSProperties; className?: string };
	Text: { children: ReactNode };
	Stack: { gap?: number; children: ReactNode };
	Card: { children: ReactNode };
	Function: { name: string };
};

export const id = Math.random().toString(36).slice(2);
export const flight = new Flight({ "invoke/ui": references<InvokeUI>("invoke/ui", "Button", "Text", "Stack", "Card", "Function") }, id);

const views = new Map<number, { render(): Promise<string> }>();
let nextViewId = 0;

export function registerView(view: { render(): Promise<string> }) {
	const viewId = nextViewId++;
	views.set(viewId, view);
	return viewId;
}

export function registerAction<T extends (...args: any[]) => any>(fn: T): T {
	return flight.action(fn);
}

export function serializeView(element: any) {
	return flight.serialize(element);
}

export function renderRegisteredView(viewId: number) {
	return views.get(viewId)?.render() ?? null;
}

export async function runRegisteredAction(actionId: string, args: unknown, viewId: number) {
	await flight.execute(actionId, Array.isArray(args) ? args : []);
	return renderRegisteredView(viewId);
}
