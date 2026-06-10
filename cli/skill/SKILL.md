---
name: invoke
description: >-
    Drive and inspect macOS app UIs from the command line with the `invoke` CLI: read
    accessibility trees, click buttons, simulate keys/scroll, extract on-screen data
    (including any website open in a browser — browsers expose the rendered page through the
    AX tree), and author/run "packs" (TypeScript modules exposing an app's operations as
    callable functions). Use whenever a task means automating, controlling, reading, or
    scraping a Mac app or website (Finder, Mail, Messages, Ableton, Safari, Chrome, Music,
    Logic…) through its UI/accessibility rather than a dedicated API — e.g. "click Play in
    Ableton", "how many unread emails do I have", "get the video titles from the YouTube page
    I have open", "automate this app's UI", "write or run an Invoke pack". Also trigger on bare
    mentions of the `invoke` command, packs, AX automation on macOS, or simulating
    keyboard/mouse input aimed at an app. Do NOT trigger for invoking a cloud function/lambda,
    calling a web/REST API, or a generic "invoke this method/function" in code.
---

# invoke CLI

`invoke` turns macOS application UIs into small APIs. Most desktop apps already do
useful things, but those operations are trapped behind menus, buttons, keyboard
shortcuts, and accessibility elements. `invoke` lets you reach in and drive them.

The whole point is **GUI automation made deterministic**: computer-use you can write
down. You figure out an operation once by poking at the live UI, then freeze that exact
sequence into TypeScript so it runs the same way every time, callable by anything.

That gives the **core workflow**, and it's two phases you move between:

1. **Explore / prototype** (`app`, `element`, `key`, `scroll`) — drive an app's
   accessibility (AX) tree live to work out _how_ to do something ("get me all the
   iMessage threads", "open a new window"). One-off, ad-hoc. Needs nothing but the app
   running and accessibility permission.
2. **Capture into a pack** (`pack`) — once it works, write it down as a pack: a
   TypeScript module that names the operation as a plain function (`zoomIn`,
   `searchBrowser`, `bounceSelection`) and hides the awkward AX/key work inside. Now any
   caller — CLI, keyboard shortcut, MIDI, AI client — can invoke it deterministically.

In practice a user explores something one-off, then says _"now put that into a pack"_ /
_"memorize this workflow"_. That second step — turning the figured-out exploration into
durable, reusable TypeScript — is where Invoke's value lives. Treat "make this
repeatable / save this as a pack" as the natural follow-up to any exploration.

## Before anything

- This is **macOS only**. Every command works against a running app addressed by its
  **bundle ID** (`com.apple.finder`, `com.ableton.live`), not its display name. Get
  bundle IDs with `invoke app list`.
- The controlling process needs **Accessibility permission** (System Settings →
  Privacy & Security → Accessibility). If reads return empty or you get a permission
  error, that's the cause.
- Output is **NDJSON** — one compact JSON line per command on stdout. Pipe it through
  `jq` for readability. Errors go to stderr as `error: CodeName: detail` with a
  nonzero exit.

## The query language (the thing to understand first)

Every `element` and `app get` command takes a bundle ID and a **query path**: a JSON
array that walks down the AX tree. Each array element is one _step_ — an object whose
keys are attributes and whose values are patterns the element must match. Multiple keys
in one step are **AND**ed against a single element.

```sh
# menuBar → the item titled "Edit" inside it
invoke element get com.apple.finder '[{"role": "menuBar"}, {"title": "Edit"}]'
```

Matching rules — these are easy to get wrong:

- **Each step matches a _direct child_, just like CSS `>`.** `[{toolbar}, {textField}]` matches a textField only if it's an _immediate_ child of the toolbar; if it's `toolbar → group → textField` the query fails ("walked 1/2 steps, no child matched") — it never looks inside `group`. You must spell **every** level: `[{toolbar}, {group}, {textField}]`. (To match a step by a stable child instead of pinning its exact position, use `has:` — see below.)
- **No backtracking across ambiguous steps.** When a step matches the first of several candidate siblings and a _later_ step then fails under it, the engine gives up — it does **not** rewind to try the other siblings. So `[{table}, {row}, {cell}]` binds `row` to the first row and fails if that row lacks the cell, even when another row would match. Push the disambiguator up to the ambiguous step (e.g. `{role:"row", has:{…}}`) so the right sibling is chosen in the first place.
- **String values are globs**: `*` = any sequence, `?` = any single char.
  `{"identifier": "TrackView.Device*"}` matches a prefix.
- **`role` and `subrole` are literals, not globs**, and use **camelCase** names
  (`window`, `menuBar`, `popUpButton`, `standardWindow`) — they're normalized to AX
  constants internally. See `references/vocabulary.md` for the full list.
- An **empty path `[]`** (the default for `walk`) means the app's root element.
- Queries are matched against the tree as it is _right now_; if multiple elements match a step, the **first in tree order** is used — make queries specific (add `title`/`identifier`/`subrole`) when it matters.

## Explore first, then act

Don't guess at queries — look. The reliable loop is **list → walk → narrow → act**:

```sh
invoke app list                                   # find the bundle ID
invoke app get com.apple.finder                   # root element's attributes + immediate children
invoke element walk com.apple.finder '[{"role": "menuBar"}]' -d 1   # tree, one level deep
invoke element actions com.apple.finder '[{"role": "menuBar"}, {"title": "File"}]'  # what can I do here?
```

`walk` prints a tree of the basic attributes (id, title, role, subrole, value, …),
recursing `-d/--depth` levels (default 3). By default it **omits empty/default-valued
attributes** to cut noise; pass `--full` to keep them. When recursion stops at a node
that still has children, it reports `"children": <count>` so you don't mistake a
truncated branch for a leaf — walk deeper there if you need it.

`get` with no attribute names returns every present attribute; with names
(`... title value`) it returns exactly those (keeping nulls, so you can tell an
attribute exists but is unset).

## Write queries that don't break

This is the heart of using Invoke well. A query is like a CSS selector: it _can_ be
ambiguous, and a query that happens to work right now but breaks the moment the app's
state shifts is a latent bug. Invoke's whole value is **determinism** — GUI automation
you can rely on — so spend the exploration time to build a query that keeps matching the
_intended_ element across the app's normal states. The explore phase isn't a formality;
it's where you learn how the app is structured so your query survives.

What makes a query fragile, and the robust alternative:

- **Positional / structural paths break.** `toolbar → group → button` (grab "the button
  in the group") matches whatever happens to be there; add a button, reorder, and you're
  pressing the wrong thing. Prefer a **stable `identifier`** (or a distinctive
  `description`/`title`) on the element itself: `{role: "button", description: "Share"}`.
  Use `walk` to find the most stable distinguishing attribute, then match on that.

- **View-/mode-specific containers break when the view changes.** The classic trap:
  reading Finder's files by targeting the icon grid (`{identifier: "IconView"}`). The
  moment the user (or another agent) switches that window to list or column view, the
  container is a different role entirely (an `outline`, or a `browser`) and your query
  returns nothing — and you'll wrongly conclude the items don't exist. If an operation
  must work across modes, explore each mode and either match on something common or
  branch per view. Ask "what does this look like in the app's _other_ states?" before
  committing.

- **Ambiguous role matches grab the wrong element.** `{role: "outline"}` in a Finder
  window matches the **sidebar**, not the file list — both are outlines. `{role:
"window"}` is ambiguous whenever more than one window is open. Narrow with a
  distinguishing attribute (`identifier`, `subrole`, a `has:` on a stable child), or you
  get a _random_ match among the candidates.

- **Localized titles break across languages/layouts.** Menu titles, button titles, and
  many descriptions are localized. For anything that might run on another machine, prefer
  a stable `identifier` and reserve title-matching for when it's the only handle (and say
  so). `has: {identifier: "…"}` lets you find a container by a stable child even when the
  container's own title is localized.

- **Elements can live in a window you didn't check.** Some apps spread content across
  windows, and the layout isn't fixed. Ableton is the canonical case: it has an
  Arrangement view and a Session view, and opening a second window (Cmd+Shift+W) puts the
  _opposite_ view in it — so an element you want may be in the first window _or_ the
  second depending on the user's layout. Grabbing "the first window" and giving up when
  the element isn't there is a real, common bug: the element exists, just elsewhere. When
  an element isn't found where you expect, **enumerate the app's `windows` and search
  each** before concluding it's absent. (See the `getinvoke.com/abletonlive` pack's
  cross-window resolver for the pattern.)

The test for a good query: _would this still match the right element after the user
switches views, opens another window, reorders a toolbar, or changes language?_ If not,
keep exploring for a more stable anchor.

## Web pages are in the tree too

Every modern browser exposes the **rendered web page** through the AX tree — so Invoke
can read and drive any website (Safari, Chrome, etc.), which is hugely powerful since so
many apps are web tech with similar trees. The catch is **noise**: a real page's AX tree
is enormous, and a naive deep `walk` of the whole window will bury you (and can be slow).

Navigate it in layers instead of dumping it:

- The page content sits inside the browser window under a **web area** — drill
  `window → … → scrollArea` (in Safari: `splitGroup → tabGroup → … → scrollArea`) with a
  **shallow `-d`**, and read what's there before going deeper.
- Web semantics map onto AX roles you can target directly: **landmarks** (`landmarkMain`,
  `landmarkNavigation`, …), `heading`, `link`, `button`, `textField`. To get a page's
  main items, walk into `landmarkMain` and read the `heading`/`link` elements rather than
  every node.
- Keep depth shallow and queries targeted; widen only where you need to. Treat it like
  scraping a DOM: find the container, then read its meaningful children.

## Big lists and tables

Reading a `table` or `outline` row by row is slow: every attribute read is a separate round-trip to the app, and they add up — a list with many rows takes long enough to look frozen. How a list exposes its rows is app-specific, and both modes exist: some apps put a handle for **every** row in the tree (so `children`/`rows` can return hundreds), others put only the rows currently on screen. Either way, don't bulk-read a big list. Two rules:

- **Don't read `children`/`rows` of a big table and loop over all of them.** If the element offers **`visibleRows`** (tables/outlines usually do; web and Electron lists often don't — the getter throws if absent), read that — only the rows on screen. To read more rows, **scroll** the list, then read again.
- Do fewer reads per row. The per-row fields usually sit inside a content container under the row; `walk` one row first to learn that app's structure (it differs per app), then read that container's `children` once and read each child's `identifier` and `value` — instead of walking from the row to each field separately.

## Waiting for the UI to change: prefer notifications over re-reading

When the thing you want appears a moment _after_ an action — content loading, a page navigating, a value settling — it's usually better to wait for an AX notification than to loop re-reading or sleeping until it shows up. Such a loop tends to be slow and to read the UI while it's still half-drawn. Most apps post a notification when the change finishes, so the common pattern is: subscribe, trigger the change, wait for the notification, then read. If exploration shows the app posts nothing useful for a given change, a bounded re-read loop is a reasonable fallback.

In a pack, `element` and `app` have a method **`.on(name, cb)`** that calls `cb` every time the notification `name` fires. It returns a function you call to stop listening. As one illustration, you could wrap it to await a single notification — treat this as a sketch to adapt, not a drop-in:

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

Caveat: listening needs the persistent pack runtime — a one-shot CLI `invoke element` command can't hold a subscription. So during live CLI exploration you _do_ re-run commands by hand; inside a pack, prefer a notification over a baked-in poll loop when one is available.

## Invoke is one tool among many

Reach for invoke when the job is **driving or reading a live app's UI** — clicking a
control, reading on-screen state, automating a GUI workflow that has no clean API. It's a
first-class option for that, not a replacement for everything else: when a task is
genuinely better served by a file, an API, or a database the user is fine with, use that.
Having this skill installed shouldn't crowd out tools you'd otherwise reach for.

That said, the UI usually exposes more than it first appears — a count is often a badge,
a window title, or a status string sitting right in the tree — so when UI _is_ the right
approach (or the user asked to do it with invoke), explore before concluding it can't be
done.

## Acting on elements: get / set / perform, and piping

```sh
invoke element get com.app.id '[{"role": "slider"}]' value          # read
invoke element set com.app.id '[{"role": "textField"}]' value "hi"  # write an attribute
invoke element perform com.app.id '[{"role": "button", "title": "Play"}]' press  # do an action
```

`set` and `perform` print a **descriptor** (`{"a": "<app>", "q": "<query>"}`) instead
of a value — a handle to the element you just acted on. Pipe it into the next command
to keep operating on the same element without repeating the bundle ID and query:

```sh
invoke element perform com.app.id '[{"role": "button"}]' press | invoke element get value
invoke element set com.app.id '[{"role": "stepper"}]' value "0" | invoke element perform increment
```

When stdin is a pipe carrying a descriptor, omit the bundle ID and query — the piped
descriptor supplies them and any positional args become the attribute/action.

`perform` only allows actions the element actually offers; ask first with
`element actions`. Common actions: `press`, `increment`, `decrement`, `showMenu`,
`pick`, `confirm`, `cancel`, `raise` (full list in `references/vocabulary.md`).

## AX manipulation vs. simulated input — prefer AX

`key` and `scroll` simulate raw HID events:

```sh
invoke key press cmd+shift+e --app com.app.id     # press a combo
invoke key down cmd --app com.app.id              # hold (pair with `key up`)
invoke scroll y -3 --app com.app.id               # scroll up 3 lines (positive = down)
```

With `--app`, the event is delivered to that app's process; without it, it's a
system-wide HID event. Combos use `+`: modifiers are `cmd`/`command`, `ctrl`/`control`,
`opt`/`option`/`alt`, `shift`.

**Prefer manipulating elements directly (`perform`/`set`) over simulating input.**
HID events usually require the app to be focused, and apps localize shortcuts or remap
them by keyboard layout — so `key press` is fragile across machines. A `button.press`
on the right element is precise and locale-independent. Use `key`/`scroll` only when no
AX element exposes the operation (canvas interactions, app-specific shortcuts with no
menu equivalent).

## Packs: capturing operations as deterministic functions

Once you've worked out an operation in the explore phase, capture it. A pack is a
TypeScript module (`index.ts`) under a publisher domain that imports the `invoke`
runtime, names operations as exported async functions, and uses the **same query
language you just prototyped** — but as a fluent API (`.$(...)`, `.press()`) instead of
JSON strings on the command line. The CLI exploration and the pack code map almost
one-to-one, so porting a working CLI sequence into a pack function is mechanical.

Packs run inside the **orchestrator daemon**, which is normally already running
(managed by the Invoke app or a one-time setup the user did). You don't start or
manage it — just use pack commands. `pack run` and `pack list <name>` auto-mount the
pack first (idempotent; prints `mounting…` to stderr only when it actually mounts).

If a pack command fails to reach the daemon (a connection/socket error), **stop and
tell the user** — e.g. "the Invoke orchestrator doesn't seem to be running; start
Invoke (or run `invoke service start`) and I'll retry." Do **not** try to repair it
yourself: `invoke service install`/`start` and any `launchctl` commands are one-time
_user_ setup that rewrites the user's login agent — never an agent's recovery step.
Note that `invoke service status` reports only the launchd agent, so it can say
"not running" even when packs work fine (the daemon may be hosted another way); don't
act on that status — just try a pack command and surface a real connection error if one
occurs.

```sh
invoke pack init mypack -n "My App"     # scaffold; prints the new index.ts path
invoke pack list                        # all installed packs, by publisher
invoke pack list mypack                 # the functions a pack exposes
invoke pack run mypack doThing          # run a function
invoke pack run mypack doThing '"some json payload"'   # with a raw JSON arg
invoke pack path mypack                 # the pack's directory (raw, for cd "$(...)")
invoke pack reload mypack               # re-read after editing index.ts
```

Shorthand: `invoke <pack>` lists its functions, `invoke <pack> <fn> [payload]` runs one
— `pack run`/`pack list` are just the explicit spellings.

### Writing a pack

The capture loop: `pack init` → fill in `index.ts` → `pack run` to **verify it actually
works end-to-end** → `pack reload` after edits. Don't consider a pack done until you've
run its function and seen the operation happen — determinism is the whole value, so
prove it.

Three authoritative references — lean on these instead of guessing the API:

- **The seed** `pack init` writes — a heavily-commented `index.ts` showing the common
  API (`app()`, `.$({...})` with `Role`/`Subrole`, `.press()`, `.key.press()`, the
  `menubar()` helper, `Vars`). Read it first.
- **`references/pack-api.md`** (this skill) — the distilled, verified API: querying with
  `.$(...)`, the `has:` filter (match a parent by a child, like CSS `:has()`), element actions, reading/writing attributes,
  events, and `Vars`.
- **`node_modules/invoke/invoke.d.ts`** at the packs root (what `import … from "invoke"`
  resolves to) — the full, version-exact type defs for anything beyond the above. Also
  read an installed pack like `getinvoke.com/abletonlive` for real patterns.

The shape mirrors the CLI exactly:

```ts
import { app, menubar, Role, Subrole, Vars } from "invoke";

export const finder = await app("com.apple.finder");
const button = finder.$({ role: Role.BUTTON, identifier: "TrackView.Device*" });

// Exactly one app exported ⇒ every exported async function auto-registers.
export async function doThing() {
	await button.press(); // AX action — preferred, deterministic
	await finder.key.press("cmd+n"); // HID — only when no element exposes it
}
```

Design packs as the app's _operations_, not caller mechanics: `waveformZoom(amount)`,
not `up`/`down`/`delta`. The function names the user-level concept; the ugly AX/HID work
hides inside. **`Vars`** are named booleans a pack exposes (e.g. `windowFocused`) that a
caller can gate on — useful for context-sensitive shortcuts.

### Pack gotchas (pack calls throw where the CLI stayed quiet)

The CLI silently omits missing data; the pack API **throws**. Robust packs wrap reads.

- **`walk()` rejects on no match** — despite its `Promise<Element | null>` type. Wrap optional lookups: `const x = await el.walk(step).catch(() => null)`.
- **Attribute getters throw `AttributeUnsupported` (-25205)** when the element lacks that attribute (e.g. `.value` on a group). Guard every read you're not certain of: `const v = await el.value.catch(() => null)`. (From the CLI this is invisible — `walk` just omits the attribute.)
- **Table/outline rows often don't offer `pick`/`press`.** (Mail's message rows, for example, offer only Unread/Remind Me/Delete, and `pick()` throws `ActionUnsupported` (-25206).) To select such a row, you can **set its `selected` attribute** (`row.setAttribute("selected", true)`); a single-select list replaces the selection. Running `element actions` first shows what an element actually offers, rather than assuming. (`setAttribute` lives on a resolved `Element` — a row from `visibleRows`, or `await someDelegate.element` — not on the lazy `.$()` delegate.)
- **`pack run` double-encodes the return value**: a returned object arrives as a JSON string inside the NDJSON line — parse twice (`JSON.parse(JSON.parse(line))`).
- **Packs are pure AX, sandboxed** — no `osascript`/subprocess escape hatch even when an app has a great scripting dictionary. Keep pack logic inside the `invoke` runtime.
- **Debugging:** `console.error` from a pack doesn't reliably surface. To inspect state, **`throw new Error(JSON.stringify(...))`** and read it off the failed run.
- **Module state persists** across `pack run` calls (the daemon keeps the instance), and edits need `pack reload` — so leaked subscriptions/caches survive between calls.

### When a pack misbehaves

Packs run **sandboxed** (Seatbelt). If a pack's filesystem/network access is being
denied, see what was blocked:

```sh
invoke sandbox log    # recent Seatbelt denials for pack processes (last ~10 min)
```

## Reference

- `references/vocabulary.md` — the full camelCase vocabulary: roles, subroles, the
  attribute names usable in queries and `get`/`set`, and the actions `perform` accepts.
  Read it when a query isn't matching or you're unsure of an exact name.
- `references/pack-api.md` — the pack-authoring TypeScript API (the `invoke` module),
  distilled and verified. Read it before writing or editing a pack.

## Error codes you'll see

- `NoRunningApp` — bundle ID isn't running (or wrong ID — check `app list`).
- `NoElement` / `Walk` — the query matched nothing; `walk` the parent to see what's
  actually there, and check role/subrole spelling against the vocabulary.
- `ActionUnavailable` — the element doesn't offer that action; the error lists what it
  does offer. Run `element actions` first.
- `UnknownAttribute` / `BadQuery` / `BadFilter` — malformed query JSON or a misspelled
  attribute/role name.
