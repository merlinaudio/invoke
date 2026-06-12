# Driving browsers and web content

Browsers expose the rendered page through the AX tree, but each engine has its own behavior. Read this before automating Safari, Chrome, Firefox, or any Electron/CEF app (most "desktop" apps of web origin — Spotify, VS Code, Slack-likes). For *extracting data* from pages and big lists, see `reading.md`; this file is about getting the tree to exist at all and acting on it.

## Wake the accessibility tree first (Chromium, Gecko, Electron, CEF)

These engines ship with the renderer's AX tree **dormant**: the window walks as a handful of empty groups and you'll wrongly conclude the app exposes nothing. Wake it by setting an attribute on the **app element** (query `[]`):

```sh
invoke element set com.spotify.client '[]' AXEnhancedUserInterface true   # Chromium/Gecko/CEF
invoke element set com.electron.app '[]' AXManualAccessibility true      # Electron
```

What to expect — this is the part that surprises everyone:

- On Chrome/Firefox/CEF the set **returns an error** (`-25208` cannot-set) **but wakes the tree anyway as a side effect**. Ignore the error; re-walk.
- Electron's `AXManualAccessibility` succeeds silently and populates the tree.
- Population is **asynchronous** (seconds). Re-walk until the node count jumps (e.g. 11 → 300); the first read after the poke is often a skeleton. Firefox's tree visibly mutates between reads while warming up — two consecutive walks can disagree.
- Chrome sometimes exposes only a skeleton (empty groups under the web area) even after the poke; full page content may require Chrome to be running with `--force-renderer-accessibility`. If the user's Chrome wasn't launched that way, prefer Safari for page content when the choice exists.

## Engine differences that change your approach

- **Safari** collapses pages into a compact semantic tree (landmarks a few levels under the window) — the easiest engine to query. **Chromium exposes ~raw DOM**: the same control can sit 18+ anonymous `group` levels below the web area with sibling fan-out, which the direct-child query language often **cannot reach at all** (no descendant step, no index step). You can still *locate* such elements by dumping (`walk -d 30 > /tmp/tree.json`, then `jq 'paths(. == "Seek slider")'`) — but a positional jq path can't be turned into a query, so locating ≠ actionable. When a target is buried like this, act via the browser's **native chrome** (address bar, tabs, toolbar — identical across engines and fully AX-addressable), or pick Safari; site keyboard shortcuts via `key press` are the last resort (they need the app focused and assume the site's bindings).
- **Safari exposes only the front tab's page.** Background tabs have no web area in the tree. Tabs are pressable `radioButton`s (`AXTabButton`) — switch first, then read. In compact tab layout the address field nests *inside the active tab's button*.
- **Firefox**: its notification-banner web area can appear before the page's web area, so `AXWebArea`-anchored queries grab the banner; nearly every element reports `subrole: "unknown"` (nonstandard Gecko subroles) — don't anchor on subrole there.

## Acting inside web content

- **Value writes are silently ignored** by most web-rendered controls: `set value` on a web slider or input returns success and does nothing. Verify every write by reading back. For sliders, converge with `increment`/`decrement` in a loop instead — and wait ~80ms before each re-read (the web applies the step asynchronously; an immediate read returns the old value).
- Some web widgets swallow `press` too (a custom search box whose real `<input>` isn't in the tree). When a web control won't cooperate, **fall back to native chrome**: e.g. navigate by URL instead of fighting an in-page search widget.
- **Navigating by URL** — order matters, and the commit step differs per browser:
  - Safari: set `focused=true` on the address field → set `value` → perform `confirm`. (An unfocused address field *reverts* written values.)
  - Chrome: omnibox has no `confirm` — set `focused=true` → set `value` → `invoke key press return --app com.google.Chrome`. (One of the rare justified `key` uses: no AX action commits the omnibox. The omnibox must hold focus for it to land.)
- **Reading the current URL**: the `url` attribute currently serializes as `null` (known gap), so read the address field's / omnibox's `value` instead.

## Anchors that survive page state

- Window/tab titles can carry live prefixes — YouTube prepends a notification count (`"(3) Video title"`). Title globs must tolerate them: `"*Video title*"`.
- Prefer **state-tolerant globs** for stateful buttons: YouTube's player buttons keep a keyboard-hint suffix, so `{"description": "*(k)"}` matches Play, Pause, *and* Replay.
- Watch for **substring overlap**: "Dislike this video" *contains* "like this video" — a glob that relies on tree order to pick the right one is a latent bug; anchor on the full distinguishing text.
