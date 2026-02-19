# Google Docs Support Implementation Guide
---

## 1. Why Google Docs needs a special integration

Google Docs is not a normal contenteditable surface.

- The extension content script runs in an isolated world.
- Docs keeps key editor APIs in the page (main) world, especially:
  - `window._docs_annotate_getAnnotatedText`
  - selection operations on the annotated text object
- Standard DOM range APIs are not reliable for mapping lint spans to visual boxes.

So the implementation must use a **main-world bridge script** and communicate through DOM state/events.

---

## 2. Required architecture

You need 3 layers working together:

1. **Content script layer (isolated world)**
   - Detects Google Docs pages.
   - Injects the main-world bridge script.
   - Owns the lint framework target registration and update scheduling.

2. **Main-world bridge layer** (`public/google-docs-bridge.js`)
   - Talks directly to Docs internals.
   - Publishes canonical text to a hidden bridge node.
   - Handles rect requests and replacement requests.
   - Emits layout/text events.

3. **Lint box computation layer** (`computeLintBoxes` in lint-framework)
   - Detects the special Google Docs target.
   - Uses the bridge protocol to map lint spans to boxes.
   - Caches/projections to meet frame budget.

Without all three, either linting, positioning, or applying suggestions will fail.

---

## 3. DOM contract and IDs/events

### Core bridge nodes

- Main-world bridge node id: `harper-google-docs-main-world-bridge`
  - hidden, appended to `document.documentElement`
  - carries shared attributes/text for cross-world communication

- Content-script target node id: `harper-google-docs-target`
  - hidden tiny fixed element appended in content script
  - carries `data-harper-google-docs-target="true"`
  - registered as a lint target

### Required custom events

- `harper:gdocs:text-updated`
  - main world -> content script
  - tells content script text changed, relint needed

- `harper:gdocs:layout-changed`
  - main world -> content script
  - tells content script layout epoch changed

- `harper:gdocs:get-rects`
  - lint-framework/content script side -> main world
  - requests display rects for `{ start, end, requestId }`

- `harper:gdocs:replace`
  - lint-framework/content script side -> main world
  - requests text replacement for `{ start, end, replacementText }`

### Required bridge attributes

- `data-harper-layout-epoch`: monotonic integer
- `data-harper-layout-reason`: reason string (`scroll`, `wheel`, `key-scroll`, `resize`, `mutation`, `init`, etc.)
- `data-harper-rects-<requestId>`: JSON array of rects written by main world
- `textContent` of main-world bridge: canonical Docs plain text

This contract must remain stable if you want compatibility with the rest of the current code.

---

## 4. Main-world bridge responsibilities

### 4.1 Ensure hidden bridge exists

On load, create/find `#harper-google-docs-main-world-bridge`.

Requirements:
- `aria-hidden="true"`
- `display:none`
- append to document root

### 4.2 Sync canonical text

Loop:
- call `window._docs_annotate_getAnnotatedText()`
- store result at `window.__harperGoogleDocsAnnotatedText`
- call `annotated.getText()` and copy into bridge `textContent`
- emit `harper:gdocs:text-updated` only when text changed

Current cadence:
- initial call once
- then `setInterval(syncText, 100)`

### 4.3 Emit layout epochs

Any meaningful layout change must bump epoch and set reason.

Current triggers:
- Docs editor scroll changes
- wheel on Docs editor area
- keydown `PageDown/PageUp/Home/End`
- window resize
- mutations on `.kix-appview-editor` subtree (childList/attributes style/class)

Use microtask coalescing so many signals in one tick become one epoch increment.

### 4.4 Rect request handler

On `harper:gdocs:get-rects`:

1. Validate request detail and resolve `annotated` object.
2. Save state:
   - current selection (if available)
   - relevant scroll state (window + candidate elements)
3. Compute rects:
   - set selection to span start/end
   - read visible caret rects (`.kix-cursor-caret`)
   - choose best caret candidates
   - derive highlight box from start/end caret geometry
4. Restore prior selection.
5. Restore scroll state **only if**:
   - no scroll state changed during compute, and
   - user is not actively scrolling (anti snap-back safeguard)
6. Write JSON rect array into `data-harper-rects-<requestId>`.

### 4.5 Replace request handler

On `harper:gdocs:replace`:

1. Get fresh annotated object.
2. Set Docs selection to `[start, end]`.
3. Find Docs text target iframe (`.docs-texteventtarget-iframe`).
4. Dispatch synthetic `ClipboardEvent('paste')` with `text/plain` payload.
5. Schedule `syncText` shortly after.

This path mimics user edit flow better than direct DOM writes.

---

## 5. Content script responsibilities

### 5.1 Page detection

Google Docs page check:
- hostname `docs.google.com`
- path starts with `/document/`

### 5.2 Injection and target setup

On Docs pages:
- inject `google-docs-bridge.js` into main world (`chrome.runtime.getURL(...)` script tag)
- create/find `#harper-google-docs-target` with `data-harper-google-docs-target="true"`
- copy main-world bridge text into target `textContent`
- register target with lint framework once (`fw.addTarget(bridge)`)

### 5.3 Sync scheduling

When receiving `harper:gdocs:text-updated`:
- schedule async bridge sync
- after sync, call `fw.update()`

Avoid overlapping sync calls (`inFlight` + `pending` flags).

### 5.4 Frame-based layout refresh

Run an rAF loop while on Docs:
- read `data-harper-layout-epoch`
- if epoch changed since last frame, call `fw.refreshLayout()`

This is what enables per-frame refresh timing without event storm reliance.

---

## 6. Lint box computation responsibilities

Google Docs path is in `computeLintBoxes` when target has `data-harper-google-docs-target="true"`.

### 6.1 Early validation

Before computing boxes:
- find `.kix-appview-editor`
- get main-world bridge
- read layout epoch/reason
- ensure lint source exactly matches bridge text (`lint.source === bridge.textContent`)

If source mismatches, return no boxes (prevents stale span mapping).

### 6.2 Cache model

Cache key: `"<span.start>:<span.end>"`

Cache entry stores:
- `rects`
- `scrollTop`
- `layoutEpoch`

Global invalidation conditions:
- bridge text changed -> clear cache
- non-scroll layout epoch change -> clear cache

### 6.3 Fast-paths

Order:
1. Exact epoch hit: use cached rects directly.
2. Scroll-layout hit (`reason` in `scroll/wheel/key-scroll`):
   - project cached rects by `deltaY = currentScrollTop - cached.scrollTop`
   - store projected rects with new epoch/scrollTop
3. Miss: request fresh rects through `harper:gdocs:get-rects`.

This is the key performance strategy for one-frame updates during scrolling.

### 6.4 Fallback path

On cache miss:
- dispatch event with requestId/start/end
- synchronously read `data-harper-rects-<requestId>` from main-world bridge
- parse/validate JSON rect array
- cache valid result

### 6.5 Box output

Return `IgnorableLintBox[]` with:
- coordinates from rects
- `source` set to editor element
- suggestion handler dispatching `harper:gdocs:replace`
- optional ignore callback

---

## 7. Highlight rendering requirements

For Google Docs source elements, highlights are rendered specially:

- render host attached to `document.body`
- host style fixed and full-viewport/inert
- per-highlight boxes rendered with `position: fixed` and transform coordinates

This avoids issues with Docs internal transformed containers and keeps overlays aligned.

---

## 8. Performance constraints and required safeguards

If reimplementing from scratch, these are mandatory to avoid regressions:

1. **Never full-scan DOM per rect request**
   - avoid `querySelectorAll('*')` in hot path

2. **Avoid forced recompute for every lint on scroll**
   - use epoch+reason-based cache and scroll delta projection

3. **Guard against scroll snap-back**
   - if user is actively scrolling, do not restore saved scroll state

4. **Coalesce layout invalidations**
   - microtask batching for epoch bumps

5. **Frame-align layout refresh**
   - detect epoch changes in rAF loop, call `refreshLayout()` there

6. **Be resilient to Docs transient errors**
   - catch around bridge ops and fail soft

---

## 9. End-to-end rebuild plan (recommended order)

1. Implement page detection + bridge script injection in content script.
2. Implement hidden target node and one-time `fw.addTarget(...)` registration.
3. Implement main-world text sync loop and `text-updated` event.
4. Implement layout epoch + reason signaling in main world.
5. Implement rAF-driven epoch watcher in content script with `fw.refreshLayout()`.
6. Add Google Docs branch in `computeLintBoxes` with source validation.
7. Add synchronous rect request/response protocol.
8. Add cache + scroll-delta projection.
9. Add replacement event path (`harper:gdocs:replace`) in main world.
10. Ensure highlight renderer treats Docs source as fixed overlay on body.
11. Add/verify anti snap-back logic in rect handler.

At each step, verify basic correctness before moving on.

---

## 10. Test and verification checklist

Minimum manual checks:

1. Load a long Google Doc and wait for lints to appear.
2. Scroll continuously:
   - highlights stay roughly aligned
   - no slideshow-level jank
   - no forced jump-back scrolling
3. Use PageDown/PageUp/Home/End and verify layout updates.
4. Apply a suggestion:
   - replacement appears in Docs
   - bridge text updates
   - lints refresh
5. Edit document content quickly and ensure stale highlights disappear.
6. Resize window and verify highlight refresh.

Automated coverage currently exists for bridge wiring/rect behavior in chrome-plugin tests, but real Docs perf/UX should still be validated manually.

---

## 11. Known tradeoffs in current design

- Rect computation is caret-proxy based, not perfect geometric extraction.
- The fallback rect request path is synchronous and can still be expensive on miss.
- Scroll-delta projection is an approximation; non-pure-scroll layout shifts require invalidation.
- Polling (`syncText` interval) is simple and robust but not event-perfect.

These tradeoffs are deliberate to balance reliability and performance in Docs’ constrained environment.

---

## 12. If you want to go beyond parity

Not required for parity, but likely future improvements:

- batch rect requests for many lints in one event payload
- move more highlight paths to CSS Highlights API where possible
- adaptive text sync cadence based on activity/visibility
- performance telemetry counters (cache hit rate, rect miss cost, epoch frequency)

