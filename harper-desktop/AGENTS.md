# AGENTS.md

## Commands
- Use `pnpm` for frontend packages; `package.json` pins `pnpm@10.10.0`.
- Dev app: `just dev-desktop` from repo root. It runs `cargo tauri dev`; Tauri then runs `pnpm dev` on port `1420`.
- Frontend checks: `pnpm check`.
- Frontend build only: `pnpm build`.
- Full desktop checks: `just check-desktop` from repo root.
- Rust checks: `cargo check -p harper-desktop --all-targets` from repo root.
- Rust formatting/fix loop from repo root: `cargo fmt && cargo check -p harper-desktop --all-targets`.
- Bundle builds match CI: `just build-desktop-linux` or `just build-desktop-macos`.
- `just build-desktop-linux` builds deb/rpm/appimage bundles.
- `just build-desktop-macos` builds app/dmg bundles.

## Architecture
- This is a SvelteKit SPA inside Tauri v2.
- SSR is disabled in `src/routes/+layout.ts`.
- `svelte.config.js` uses `adapter-static` with `fallback: "index.html"`.
- Main Rust entrypoint is `src-tauri/src/main.rs`, which calls `harper_desktop_lib::run()`.
- CLI behavior is in `src-tauri/src/lib.rs`: no subcommand runs the normal Tauri app; `highlighter` runs the native overlay highlighter.
- Tauri config lives in `src-tauri/tauri.conf.json`.
- Tauri capabilities live in `src-tauri/capabilities/default.json`.
- The editor UI is currently `src/routes/+page.svelte`, using workspace dependencies on `harper.js` and `harper-editor`.
- Rust uses local workspace dependencies from `src-tauri/Cargo.toml`, including `harper-core` and `harper-dictionary-wordlist`.

## Highlighter Architecture
- The highlighter is Rust/egui/winit code under `src-tauri/src/highlighter/`.
- `Highlighter` is the public entry point for the overlay system.
- `Window` owns native window/GPU plumbing.
- `WindowManager` owns the winit event loop, monitor windows, cursor hit-testing, and popup selection.
- `RenderState` owns highlight rendering, popup drawing, markdown rendering cache, and popup action dispatch.
- `RenderState` uses `ActionableLint` values from `src-tauri/src/rect.rs`.
- `ActionableLint` stores lint geometry, the Harper `Lint`, and source text needed for suggestion application and popup actions.
- Suggestion popup actions currently include close, apply suggestion, ignore lint, and add to dictionary.
- Popup hover text is implemented in `RenderState` button helpers.
- The highlighter read interval is set in `run_highlighter()` with `with_read_interval(Duration::from_millis(16))`.

## OS Integration
- OS integration is behind `src-tauri/src/os_broker.rs`.
- macOS uses `src-tauri/src/mac_broker.rs`.
- non-macOS currently uses `NoopBroker`.
- macOS highlighter focus handling depends on `MacBroker.last_focused`; clicking the overlay can make the highlighter process focused, so accessibility reads fall back to the last non-highlighter PID.
- Highlighter stdout is reserved for JSON-line IPC. Diagnostics in highlighter paths should use `eprintln!`, not `println!`.

## IPC And App State
- Tauri app state lives in `src-tauri/src/config.rs`.
- `Config` currently stores:
  - `mutable_dictionary: MutableDictionary`
  - `dialect: Dialect`
  - `ignored_lints: IgnoredLints`
  - `lint_config: FlatConfig`
- `Config::new()` uses `MutableDictionary::new()`, `IgnoredLints::new()`, and `FlatConfig::new_curated()`.
- The Tauri app owns shared config as `Arc<Mutex<Config>>`.
- `run_tauri()` spawns the highlighter process and creates a protocol server for it.
- The highlighter process is spawned through `src-tauri/src/highlighter_process.rs`.
- IPC is implemented under `src-tauri/src/communication/`.
- IPC uses newline-delimited JSON over child stdin/stdout.
- The Tauri app is the protocol server.
- The highlighter process is the protocol client.
- Protocol messages live in `src-tauri/src/communication/message.rs`.
- Supported requests:
  - `GetLintConfig`
  - `IgnoreLint { ignored_lints }`
  - `AddToDictionary { word }`
- Supported responses:
  - `GetLintConfig { config }`
  - `Ack`
- `IgnoredLints` is transferred as whole serialized state and merged server-side.
- `AddToDictionary` sends only the word; Rust appends it with `DictWordMetadata::default()`.
- The highlighter uses a Tokio current-thread runtime to call async IPC client methods from synchronous UI callbacks.
- That runtime is bridge plumbing only; it is not used for linting or UI rendering.

## Dictionary And Linting Gotchas
- Keep dictionary construction centralized through `create_dictionary(user_dictionary: MutableDictionary) -> Arc<MergedDictionary>`.
- Keep linter construction centralized through `create_linter(dictionary: Arc<MergedDictionary>) -> LintGroup`.
- The same merged dictionary source must be used for both `Document::new_markdown_default(text, &dictionary)` and `LintGroup::new_curated(dictionary, Dialect::American)`.
- Do not use `Document::new_markdown_default_curated(text)` in the main highlighter lint callback when user dictionary words should suppress spelling lints.
- Harper spelling lint behavior depends on document token metadata, so a linter with the updated dictionary is not enough if the `Document` was built with the curated dictionary only.
- `RenderState` may still use curated document construction for localized source extraction where dictionary membership does not matter.

## Frontend Integration
- Tauri commands in `src-tauri/src/lib.rs` include:
  - `get_lint_config`
  - `ignore_lint`
  - `add_to_dictionary`
- JS helper lives in `src/lib/client.ts`.
- JS helper class is named `Client` and exposes static methods.
- `Client.ignoreLint(...)` uses Harper JS ignored-lints export data and invokes the Rust command.
- `Client.addToDictionary(word)` sends only the word to Rust.
- Vite's dev server must stay on port `1420`; `vite.config.js` uses `strictPort: true` because Tauri expects that port.
- Vite intentionally ignores `src-tauri/**` during frontend file watching.

## Repo-Specific Gotchas
- Harper Desktop is integrated into the root Cargo and pnpm workspaces.
- CI installs `wasm-pack`, `tauri-cli`, and `cargo-hack` with `cargo binstall`, then runs the relevant root `just` task.
- Avoid mixing behavior fixes with structural refactors in the highlighter; popup/UI, IPC, dictionary behavior, and OS-coordinate fixes are easier to review separately.

## Known Review Findings
- Secondary-monitor highlighter coordinates may be wrong. Accessibility rectangles appear to be global screen coordinates, while each overlay window's egui origin is local to that monitor/window. Translate by monitor/window origin or render through one virtual-desktop-sized overlay.
- macOS text range lookup may be wrong after emoji or other non-BMP characters. Harper spans are char-based, while macOS accessibility ranges are NSString/UTF-16 based. Convert spans before calling `AXBoundsForRangeParameterizedAttribute`.
