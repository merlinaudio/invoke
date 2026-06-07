import type { ClientReference } from "@merlin.audio/react-flight/server";
import { flight, type InvokeUI } from "../../flight";

type References<T> = Record<keyof T, ClientReference>;

export const { Button, Text, Stack, Card, Function } = flight.modules["invoke/ui"]! as References<InvokeUI>;
