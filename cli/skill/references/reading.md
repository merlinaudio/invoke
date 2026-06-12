# Reading data from the UI

How to pull on-screen data out of the AX tree without drowning in noise (web pages) or hanging on huge lists.

## Web pages are in the tree too

Every modern browser exposes the **rendered web page** through the AX tree — so Invoke can read and drive any website (Safari, Chrome, etc.), which is hugely powerful since so many apps are web tech with similar trees. The catch is **noise**: a real page's AX tree is enormous, and a naive deep `walk` of the whole window will bury you (and can be slow).

Navigate it in layers instead of dumping it:

- The page content sits inside the browser window under a **web area** — drill `window → … → scrollArea` (in Safari: `splitGroup → tabGroup → … → scrollArea`) with a **shallow `-d`**, and read what's there before going deeper.
- Web semantics map onto AX roles you can target directly: **landmarks** (`landmarkMain`, `landmarkNavigation`, …), `heading`, `link`, `button`, `textField`. To get a page's main items, walk into `landmarkMain` and read the `heading`/`link` elements rather than every node.
- Keep depth shallow and queries targeted; widen only where you need to. Treat it like scraping a DOM: find the container, then read its meaningful children.

(If the tree looks empty or you're in Chrome/Firefox/Electron, the engine's AX tree may be dormant or too deep to query — see `web.md` for the wake-up handshake and per-engine tactics.)

## Big lists and tables

Reading a `table` or `outline` row by row is slow: every attribute read is a separate round-trip to the app, and they add up — a list with many rows takes long enough to look frozen. How a list exposes its rows is app-specific, and both modes exist: some apps put a handle for **every** row in the tree (so `children`/`rows` can return hundreds), others put only the rows currently on screen. Either way, don't bulk-read a big list. Two rules:

- **Don't read `children`/`rows` of a big table and loop over all of them.** If the element offers **`visibleRows`** (tables/outlines usually do; web and Electron lists often don't — the getter throws if absent), read that — only the rows on screen. To read more rows, **scroll** the list, then read again.
- Do fewer reads per row. The per-row fields usually sit inside a content container under the row; `walk` one row first to learn that app's structure (it differs per app), then read that container's `children` once and read each child's `identifier` and `value` — instead of walking from the row to each field separately.
