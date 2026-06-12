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
that's the CLI's form). Each extra argument matches a **direct child** (one level down),
not an arbitrary descendant — so you must spell every level of the path. It returns a
lazy `ElementDelegate`; nothing happens until you act or await an attribute.

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

### `has` — a useful filter to match by a stable child

A step may carry `has: <step>`, which matches the element only if a **direct child**
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

Geometry/range attributes read and write as plain objects with the same shape in both
directions: `position` is `{x, y}`, `size` is `{width, height}`, `frame` is
`{x, y, width, height}`, `selectedTextRange` is `{location, length}`. So
`area.setAttribute("selectedTextRange", { location: 0, length: 5 })` selects text, and
what `getAttribute` returns can be passed back to `setAttribute` unchanged. `url` reads
as a string.

`.walk(...steps)` resolves an element imperatively (returns `Promise<Element | null>`),
useful when you need to search dynamically (e.g. across all windows) rather than bind a
static lazy path. Note `.walk()`/`.$()` don't backtrack across siblings — a static path
only descends the first matching branch.

## Simulated input (the HID escape hatch — last resort)

Reach for keys only when no element exposes the operation. Key events need the app
focused (a pack that uses them can't run in the background and steals the user's
focus), are localized/layout-remapped across machines, and land on whatever has focus
rather than the element you meant. Check the menu bar first — most shortcuts have a
menu item, and pressing it via AX (`menubar()` below) works backgrounded.

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
Two more focus-event traps (both hit in the abletonlive pack):

- Don't trust `event.element` — some apps fire `focusedUIElementChanged` reporting the
  bare window while a text field actually holds focus. Re-read the app's
  `focusedUIElement` in the handler instead.
- Clicking into a floating window (plugin windows, inspectors) doesn't always fire
  `focusedUIElementChanged` at all — subscribe to **`focusedWindowChanged`** too when
  focus gating matters.

## Waiting for the UI to change: prefer notifications over re-reading

When the thing you want appears a moment _after_ an action — content loading, a page navigating, a value settling — it's usually better to wait for an AX notification than to loop re-reading or sleeping until it shows up. Such a loop tends to be slow and to read the UI while it's still half-drawn. Most apps post a notification when the change finishes, so the common pattern is: subscribe, trigger the change, wait for the notification, then read. If exploration shows the app posts nothing useful for a given change, a bounded re-read loop is a reasonable fallback.

`.on(name, cb)` calls `cb` every time the notification `name` fires and returns a function you call to stop listening. As one illustration, you could wrap it to await a single notification — treat this as a sketch to adapt, not a drop-in:

```ts
// resolve when `name` fires on `el`, or after `ms`
function waitFor(el, name, ms) {
	return new Promise((resolve) => {
		let done = false;
		const finish = (hit) => { if (done) return; done = true; clearTimeout(t); off().catch(() => {}); resolve(hit); };
		const off = el.on(name, () => finish(true)); // .on() returns the unsubscribe fn
		const t = setTimeout(() => finish(false), ms);
	});
}

// Which notification fires (and on which element) is app-specific — explore to find it.
// The element that POSTS it isn't always the one you read, so subscribing on the app is the safe default.
// Subscribe BEFORE you trigger the change, or a fast update can fire before you're listening.
const loaded = waitFor(app, "loadComplete", 5000); // `app` = your AppDelegate
await row.setAttribute("selected", true);
await loaded;
const text = await content.value; // loaded now — read what you need
```

Useful names (full set in `invoke.d.ts` / `Notification`): **`loadComplete`** (web/HTML content finished loading — web views typically fire it), `selectedRowsChanged`, `layoutChanged`, `valueChanged`, `created`, `UIElementDestroyed`, `titleChanged`, `focusedUIElementChanged`. Stale notifications from a _previous_ state can arrive, so if correctness matters, **re-read on each event and accept only when the tree shows what you expect**, rather than trusting the first event.

## Patterns proven in shipped packs

Recurring problems and the shapes that solved them — reach for these before inventing:

- **Generation counter for racing async updates.** Event handlers that gather state with several awaited reads can interleave; an older handler can finish *after* a newer one and clobber fresh state. Stamp each run (`const gen = ++update.generation`) and discard its writes if `gen !== update.generation` at the end.
- **Window-scoped Vars.** Focus vars (e.g. `arrangementFocused`) must go false when focus moves to a *different window* — a plugin window, a dialog. Gate them on the focused element's window `subrole`: anything `!== "AXStandardWindow"` should read as "user is typing elsewhere" (set your input/text flag true so shortcuts stop firing into it).
- **Retry-once on navigation staleness.** The first read after a page/document change can serve a stale cached element (or throw on an invalidated one). One recursive retry self-heals: `async function f(retried = false) { …; if (bad && !retried) return f(true); }`.
- **Settle delay before re-reads.** Web/async UIs apply `increment`/`setAttribute` after returning; an immediate read sees the old value. ~80ms before the verification read.
- **State-tolerant glob anchors** for buttons whose title changes with state — anchor on the stable part (`"*(k)"` matches Play/Pause/Replay; `"Subscribe to*"` survives channel names).

## Gotchas (pack calls throw where the CLI stayed quiet)

The CLI silently omits missing data; the pack API **throws**. Robust packs wrap reads.

- **`walk()` rejects on no match** — despite its `Promise<Element | null>` type. Wrap optional lookups: `const x = await el.walk(step).catch(() => null)`.
- **Attribute getters throw `AttributeUnsupported` (-25205)** when the element lacks that attribute (e.g. `.value` on a group). Guard every read you're not certain of: `const v = await el.value.catch(() => null)`. (From the CLI this is invisible — `walk` just omits the attribute.)
- **Table/outline rows often don't offer `pick`/`press`.** (Mail's message rows, for example, offer only Unread/Remind Me/Delete, and `pick()` throws `ActionUnsupported` (-25206).) To select such a row, you can **set its `selected` attribute** (`row.setAttribute("selected", true)`); a single-select list replaces the selection. Running `element actions` first shows what an element actually offers, rather than assuming. (`setAttribute` lives on a resolved `Element` — a row from `visibleRows`, or `await someDelegate.element` — not on the lazy `.$()` delegate.)
- **`pack run` double-encodes the return value**: a returned object arrives as a JSON string inside the NDJSON line — parse twice (`JSON.parse(JSON.parse(line))`).
- **Packs are pure AX, sandboxed** — no `osascript`/subprocess escape hatch even when an app has a great scripting dictionary. Keep pack logic inside the `invoke` runtime.
- **Debugging:** `console.error` from a pack doesn't reliably surface. To inspect state, **`throw new Error(JSON.stringify(...))`** and read it off the failed run.
- **Module state persists** across `pack run` calls (the daemon keeps the instance) — so leaked subscriptions/caches survive between calls. But if any pack file changed, the next command remounts first (fresh process, state gone).

Packs run **sandboxed** (Seatbelt). If filesystem/network access is denied, `invoke sandbox log` shows recent Seatbelt denials for pack processes (last ~10 min).

## Design the function, not the mechanism

Name the **app operation** (`waveformZoom(amount)`, `bounceSelection(opts)`,
`newWindow()`), not the caller's mechanics (`up`/`down`/`delta`). The ugly AX/HID work
hides inside; the exported signature reads like the app concept. That clean boundary is
the entire point — see `docs/invoke-model.md` framing if available.
