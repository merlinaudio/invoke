# Native app archetypes

Mac apps are built with different UI frameworks, and each exposes a recognizably different AX tree. Identifying the archetype early (one shallow `walk` usually suffices) tells you which tactics apply and saves dead-end exploration. Web-engine apps (Electron/CEF and browsers) have their own file: `web.md`.

## Look at the menu bar first (probably)

Before exploring an app's window at all, consider walking its menu bar: it enumerates **without opening any menu**, and items carry their title, `enabled` state (live — tells you what's currently possible), and keyboard shortcut (`menuItemCmdChar` + `menuItemCmdModifiers`, via `get`; `walk` hides them). `press` on a menu item works even while the app is in the background. That's an app's complete, self-describing command surface in one dump — check it before hunting a window's tree for a button that probably has a menu equivalent.

There is a drawback: menu bar items are often heavily localized and don't provide stable identifiers. So for reliability/long term usage, or widely distribute d solutions, consider targeting the UI elements within the app instead. Of course, if you can maintain the localization or have control over launching the app, it may be okay to rely on the menu bar.

```sh
invoke element walk com.app.id '[{"role":"menuBar"}]' -d 3
invoke element get com.app.id '[{"role":"menuBar"},{"title":"File"},{"role":"menu"},{"title":"New"}]' enabled menuItemCmdChar menuItemCmdModifiers
```

## SwiftUI (System Settings and most new Apple apps)

- **Deep anonymous nesting**, like Chromium: meaningful controls sit many unlabeled `group` levels down with sibling fan-out. Hand-deriving a query from a deep walk frequently fails (anonymous siblings are indistinguishable; first-match binds the wrong branch). Anchor on the few elements that _do_ carry text/identifiers and `has:` your way down — and accept that some elements are unreachable today.
- **Roles lie: check `actions` before trusting `role`.** SwiftUI sidebar "buttons" can expose **zero actions** (not even `press`). Navigation works by setting `selected=true` on the enclosing outline **row** instead. Always run `element actions` before planning around a press.
- **Controls are often unlabeled** (`checkBox` with `title: null`, value 1) — the visible label is a _sibling_ `staticText`. To find "the Dark Mode checkbox", locate the staticText, then target the control near it (e.g. a `has:` on the shared parent).

## Catalyst (Messages, iPad-derived apps)

Generally clean and scriptable — stable identifiers, sensible containers. Expect vocabulary leaks: `AXGenericElement` nodes, some role-less nodes, window content under subrole `iOSContentGroup`. Treat unknown roles as normal, not as a broken tree.

## Terminals (Terminal, Ghostty, iTerm)

The entire rendered screen is a single `textArea` whose `value` is the text — TUIs, ssh sessions, and running CLIs are **readable verbatim** with one `get value`. Pair with `key press --app` to type. This makes invoke a viable supervisor for terminal workflows; remember you're typing into a live shell — be deliberate.

## Documents that aren't in the tree (Preview PDFs)

An open PDF exposes toolbar chrome and a single page element with **no text at any depth** — PDF text lives behind parameterized AX attributes invoke doesn't surface yet. Don't conclude the app is broken or keep walking deeper; read the file directly instead.

## Hybrid apps (Music, App Store)

One window, two worlds: a native sidebar (outline, readable rows) next to a web-rendered content pane (anonymous groups — all of `web.md` applies). When one half of an app behaves differently from the other, you've probably crossed the native/web boundary.

## Phantom and special windows

- Some apps' first window is a phantom — Notes carries a `{subrole: "unknown"}`, title-less, childless panel that `{"role":"window"}` matches _first_, shadowing the real one. Anchor windows with `subrole: "standardWindow"` (or enumerate `windows`) by default.
- Floating windows (`AXFloatingWindow` — plugin windows, inspectors) often have AX-opaque interiors (e.g. third-party audio plugin UIs): the tree ends at the host container. That's canvas territory — `key`/`scroll`, or operate the host app around it.

## Menu-bar status items — currently out of reach

Status-bar icons (the right side of the menu bar) are exposed as an app's `extrasMenuBar` / second `menuBar`, which the query language can't currently address (element-valued attribute; no index step to pick the second `menuBar`). Don't burn time trying — drive the app through its windows or main menus instead.

## A backgrounded app that errors on every read

`AXError -25204` on all reads usually means the app is **App Napped or not responding** — it looks broken but means "wake it first". `open -a <App>` (or setting `frontmost=true`) suffices; then retry.
