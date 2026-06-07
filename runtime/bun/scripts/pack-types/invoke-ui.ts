import type { InvokeUI } from "../../src/flight";

type Component<Props> = (props: Props) => any;

export declare const Button: Component<InvokeUI["Button"]>;
export declare const Text: Component<InvokeUI["Text"]>;
export declare const Stack: Component<InvokeUI["Stack"]>;
export declare const Card: Component<InvokeUI["Card"]>;
export declare const Function: Component<InvokeUI["Function"]>;
