# Pack authoring API (the `invoke` TypeScript module)

A pack is `index.ts` importing from `"invoke"`. The **authoritative, version-exact**
type definitions ship with the install at:

```
<packs root>/node_modules/invoke/invoke.d.ts
```

where `<packs root>` is the parent of the publisher directories — i.e. two levels up
from `invoke pack path <name>` (`.../packs/<publisher>/<pack>` → `.../packs/`). From a
pack's `index.ts`, that's what `import … from "invoke"` resolves to. **Read that
`.d.ts` for anything not covered here**, and read the seed `pack init` writes (its
comments are canonical) plus any installed pack (e.g. `getinvoke.com/abletonlive`) for
real-world patterns. This file is the distilled common subset, verified against the
shipped types.

## Binding an app

```ts
import { app, menubar, Role, Subrole, type Element } from "invoke";

export const finder = await app("com.apple.finder"); // AppDelegate
```

If exactly one app is exported from the module, **every exported async function is
auto-registered** as a pack function — no manual registration needed.

## Querying elements — `.$(...)`

`.$(...)` takes **one or more filter steps as separate arguments** (not a JSON array —
that's the CLI's form). Each extra argument descends one level into a matching
descendant. It returns a lazy `ElementDelegate`; nothing happens until you act or await
an attribute.

```ts
const window = finder.$({ role: Role.WINDOW, subrole: Subrole.STANDARD_WINDOW });
const grid = window.$(
	{ role: Role.SPLIT_GROUP },
	{ role: Role.SCROLL_AREA },
	{ identifier: "IconView" }, // strings glob: * and ?
);
```

A filter step is an object of attribute → pattern. Keys mirror the CLI vocabulary
(`identifier`, `title`, `description`, `role`, `subrole`, `value`, `roleDescription`,
…). String values **glob**. Use the `Role.*` / `Subrole.*` enums (or a raw `"AXGroup"`
string) for `role`/`subrole`.

### `has` — a useful filter to match by a stable descendant

A step may carry `has: <step>`, which matches the element only if some descendant
matches the inner step. This is the standard trick for finding a **container by a
stable child identifier** when the container itself has only a localized title:

```ts
// the group that *contains* Transport.Play
const transport = mainGroup.$({ has: { identifier: "Transport.Play" } });
```

Prefer stable `identifier`s over localized titles, labels, descriptions, etc. wherever possible, as they change
with the user's language whereas identifiers are usually stable.

## Acting on an element

All async, all on the `ElementDelegate` returned by `.$(...)`:

```ts
await button.press(); // click button / menu item / checkbox
await stepper.increment(); // slider / stepper / incrementor
await stepper.decrement();
await el.showMenu(); // open a popup/menu button's menu
await el.pick(); // select a menu item / row
await el.raise(); // bring window to front
await el.confirm(); // default button
await el.cancel(); // cancel button
await el.runAction("press"); // any action by name (escape hatch)
```

## Reading and writing attributes

```ts
const focused = await window.focused; // await the getter directly
const id = await file.identifier;
const sel = await grid.selectedChildren; // returns Element[] for element-typed attrs
const lang = await (await finder.element)!.getAttribute("AXPreferredLanguage");
await field.setAttribute("value", "hello"); // settable attrs only (value, focus, …)
```

`.walk(...steps)` resolves an element imperatively (returns `Promise<Element | null>`),
useful when you need to search dynamically (e.g. across all windows) rather than bind a
static lazy path. Note `.walk()`/`.$()` don't backtrack across siblings — a static path
only descends the first matching branch.

## Simulated input (the HID escape hatch)

Prefer element actions above; reach for keys only when no element exposes the operation.

```ts
await finder.key.press("cmd+n"); // press combo
await finder.key.down("cmd"); // hold
await finder.key.up("cmd");
```

Scroll and other input surfaces exist too — see `invoke.d.ts` for the exact shape.

## The menubar helper

Presses a menu item by walking titles. Locale-dependent (titles are localized), so
consider whether the pack will be distributed.

```ts
await menubar(finder, "File", "New Finder Window"); // File → New Finder Window
```

## Vars — caller-readable context flags

`Vars` declares named booleans a pack exposes; callers (a keyboard binding, etc.) can
gate on them (`when: "windowFocused"`). The pack keeps them current from AX events.

```ts
const when = Vars({ windowFocused: false });

finder.on("focusedWindowChanged", async () => {
	when.windowFocused = Boolean(await window.focused);
});
```

## Reacting to accessibility events

```ts
finder.on("focusedUIElementChanged", event => {
	/* … */
});
finder.onactivate = () => {
	/* app brought to front */
};
```

macOS only fires `focusedUIElementChanged` for focus moves _within_ an app, never when
you switch _into_ it — re-read `focusedUIElement` on `onactivate` if you gate on focus.

## Design the function, not the mechanism

Name the **app operation** (`waveformZoom(amount)`, `bounceSelection(opts)`,
`newWindow()`), not the caller's mechanics (`up`/`down`/`delta`). The ugly AX/HID work
hides inside; the exported signature reads like the app concept. That clean boundary is
the entire point — see `docs/invoke-model.md` framing if available.
