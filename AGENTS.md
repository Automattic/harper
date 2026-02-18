# Harper Docs Map for Agents

This repository’s documentation site is powered by Vite + SvelteKit + SveltePress.

Use `packages/web/vite.config.ts` as the source of truth for documentation scope:
- Sidebar and important doc routes are defined in `packages/web/vite.config.ts`.
- Route `/docs/...` maps to `packages/web/src/routes/docs/...`.
- Most docs are in `+page.md`; some are `+page.svelte` or route helpers.

If you're working on the Harper repository itself, please pay special attention to the `contributors/*` pages.
Importantly, all of the tools available in this repository are available via `just`. To learn more, run `just --list`.

## Read First

1. `packages/web/vite.config.ts`: Sidebar source of truth and canonical map of important docs routes.
2. `packages/web/src/routes/docs/about/+page.md`: High-level product overview, privacy model, versioning policy, and ecosystem context.
3. `packages/web/src/routes/docs/weir/+page.md`: Weir rule language reference with syntax, expression types, and examples.
4. `packages/web/src/routes/docs/rules/+page.svelte`: Live/generated rule catalog (rule names, defaults, and descriptions).
5. `packages/web/src/routes/docs/contributors/introduction/+page.md`: Entry point for contributors and links to deeper contributor docs.

## Core Documentation Directories

- `packages/web/src/routes/docs/about`
- `packages/web/src/routes/docs/weir`
- `packages/web/src/routes/docs/rules`
- `packages/web/src/routes/docs/integrations`
- `packages/web/src/routes/docs/harperjs`
- `packages/web/src/routes/docs/contributors`

## Route Prefix to File Prefix

- `/docs/about` -> `packages/web/src/routes/docs/about/+page.md`
- `/docs/weir` -> `packages/web/src/routes/docs/weir/+page.md`
- `/docs/rules` -> `packages/web/src/routes/docs/rules/+page.svelte`
- `/docs/integrations/*` -> `packages/web/src/routes/docs/integrations/*/+page.md`
- `/docs/harperjs/*` -> `packages/web/src/routes/docs/harperjs/*/+page.md`
- `/docs/contributors/*` -> `packages/web/src/routes/docs/contributors/*/+page.md`

## Files Listed in the Sidebar (Local)

- `packages/web/src/routes/docs/about/+page.md`: High-level product overview, privacy model, versioning policy, and ecosystem context.
- `packages/web/src/routes/docs/weir/+page.md`: Weir rule language reference with syntax, expression types, and examples. Very important if you're asked to write a Weir rule.
- `packages/web/src/routes/docs/rules/+page.svelte`: Live/generated rule catalog (rule names, defaults, and descriptions).
- `packages/web/src/routes/docs/integrations/obsidian/+page.md`: Obsidian plugin overview, privacy/value comparison, installation, and support links.
- `packages/web/src/routes/docs/integrations/chrome-extension/+page.md`: End-user Chrome extension overview and install link.
- `packages/web/src/routes/docs/integrations/firefox-extension/+page.md`: End-user Firefox extension overview and install link.
- `packages/web/src/routes/docs/integrations/wordpress/+page.md`: Current WordPress guidance, including migration recommendation to Chrome extension and legacy plugin status.
- `packages/web/src/routes/docs/integrations/language-server/+page.md`: `harper-ls` install methods, dictionaries, code actions, ignore comments, and full configuration reference.
- `packages/web/src/routes/docs/integrations/visual-studio-code/+page.md`: VS Code extension install, command list, and settings reference.
- `packages/web/src/routes/docs/integrations/neovim/+page.md`: Neovim setup using `harper-ls`, plus optional and common config tweaks.
- `packages/web/src/routes/docs/integrations/helix/+page.md`: Helix setup using `harper-ls`, plus optional and common config tweaks.
- `packages/web/src/routes/docs/integrations/emacs/+page.md`: Emacs setup using `harper-ls`, plus optional and common config tweaks.
- `packages/web/src/routes/docs/integrations/zed/+page.md`: Zed extension entry point and link to canonical extension README.
- `packages/web/src/routes/docs/integrations/sublime-text/+page.md`: Sublime Text setup with `harper-ls` and LSP package configuration.
- `packages/web/src/routes/docs/harperjs/introduction/+page.md`: `harper.js` mission, package overview, and installation starting point.
- `packages/web/src/routes/docs/harperjs/linting/+page.md`: Core `harper.js` lint workflow and linter usage patterns.
- `packages/web/src/routes/docs/harperjs/spans/+page.md`: Explains span objects and how to use them to locate/handle lint ranges.
- `packages/web/src/routes/docs/harperjs/configurerules/+page.md`: How to programmatically read and set `LintConfig` to enable/disable rules.
- `packages/web/src/routes/docs/harperjs/node/+page.md`: Node.js-specific usage notes, especially `LocalLinter` vs `WorkerLinter`.
- `packages/web/src/routes/docs/harperjs/CDN/+page.md`: Browser/CDN usage via unpkg and ESM import patterns.
- `packages/web/src/routes/docs/contributors/introduction/+page.md`: Contributor onboarding overview and links to architecture/testing/rule-authoring docs.
- `packages/web/src/routes/docs/contributors/environment/+page.md`: Local development environment setup across Rust, Node/pnpm, and optional Nix shell.
- `packages/web/src/routes/docs/contributors/committing/+page.md`: Commit message conventions and commit hygiene requirements.
- `packages/web/src/routes/docs/contributors/architecture/+page.md`: System architecture and roles of core components like `harper-core`, `harper-ls`, and `harper.js`.
- `packages/web/src/routes/docs/contributors/dictionary/+page.md`: Process for adding or updating curated dictionary entries.
- `packages/web/src/routes/docs/contributors/tests/+page.md`: Test-suite strategy, quality/performance focus, and related testing references.
- `packages/web/src/routes/docs/contributors/author-a-rule/+page.md`: Step-by-step workflow for implementing and testing new grammar rules.
- `packages/web/src/routes/docs/contributors/visual-studio-code/+page.md`: How to run, debug, test, and package the VS Code extension locally.
- `packages/web/src/routes/docs/contributors/chrome-extension/+page.md`: Internal architecture and local development notes for the browser extensions.
- `packages/web/src/routes/docs/contributors/wordpress/+page.md`: How to build and run the WordPress plugin locally.
- `packages/web/src/routes/docs/contributors/obsidian/+page.md`: Obsidian-plugin contributor workflow and plugin-specific constraints.
- `packages/web/src/routes/docs/contributors/review/+page.md`: PR reviewer playbook, including ways to fetch artifacts and test patches locally.
- `packages/web/src/routes/docs/contributors/local-stats/+page.md`: Local stats logging model, `stats.txt` format, locations, and privacy behavior.
- `packages/web/src/routes/docs/contributors/brill/+page.md`: Brief explanation of Harper’s Brill-tagging approach and further reading link.
- `packages/web/src/routes/docs/contributors/faq/+page.md`: Contributor FAQ for conceptual distinctions (for example `Linter` vs `PatternLinter`).

## Documentation Route Helpers (Non-`+page.md`)

- `packages/web/src/routes/docs/about/+page.ts`: Route behavior helper (`ssr = false`) for the About page.
- `packages/web/src/routes/docs/harperjs/CDN/example/+server.ts`: Serves the HTML example used by the `harper.js` CDN documentation page.

## External Sidebar Targets (No Local Source File)

- `https://docs.rs/harper-core/latest/harper_core/`
- `/docs/harperjs/ref/index.html` (generated API reference target)

## Projects Contained in This Repository

- `harper-core`: The core grammar checking engine. This is a dependency to pretty much everything related to Harper.
- `harper-ls`: A Language Server compatible with a number of text editors, including Neovim, Zed, and Helix. See above linked documentation for more details.
- `harper-cli`: A command-line binary for debugging Harper's core engine and markup language support.
- `harper-comments`: Provides parsers for a number of programming langauges to support linting their comments.
- `harper-wasm`: The WebAssembly build target that powers browser and JavaScript integrations such as `harper.js`.
- `packages/lint-framework`: A package containing the tooling necessary to read/write/highlight text on the web for the purpose of linting.
- `packages/components`: Shared Svelte component package used by web-facing packages.
- `packages/web`: The Harper website, including documentation and a live demo that uses the `lint-framework`.
- `packages/harper.js`: The JavaScript package that uses `harper-wasm` to lint text from websites or Node.js processes.
- `packages/chrome-plugin`: The Harper Chrome Extension - uses the `lint-framework`. Also support Firefox.
- `packages/obsidian-plugin`
- `packages/wordpress-plugin`
- `packages/vscode-plugin`: The Harper Visual Studio Code plugin. Uses `harper-ls`.

There are of course projects in this repository not listed above. If relevant, feel free to poke around.

## On Writing New Rules

When asked to write a new rule, keep these guidelines in mind:

- The user is almost always expecting you to write it to a file. Which file and where is up to you to find out.
- You should include at least 15 total tests, covering a wide variety of cases. Cover false-positives, false negatives, true positives, and if relevant, true negatives.
- You should run any and all tests to ensure that you do no break existing behavior and that your new rule runs the way you expect.
- If the rule is related to a closed compound noun, see if you can just add an entry to the existing closed compound linter.

Unless you are specifically requested to write the rule in a specific way, choose the language (Rust or Weir) and methodology that fits the task.

ALWAYS run extensive bullet tests with `cargo run --bin harper-cli --release -- lint <TEXT>` to make sure the new rule isn't already covered by Harper.

## Workflow for Writing Weir Rules

1. Draft the core expression
- Encode the match with `expr main` using words, sequences, alternatives, filters, exceptions, POS tags, wildcards, or punctuation.
- Keep the expression minimal but precise; avoid overmatching.
- If a wordlist is needed, include it as its own expression, used with an expression reference.

2. Add rule metadata
- `let message`, `let description`, `let kind`, `let becomes` (and `let strategy` if needed).
- Use `strategy "Exact"` when casing must be normalized; otherwise default behavior or `MatchCase` as appropriate.

3. Add tests (required)
- Include at least 15 tests.
- Tests must cover: true positives, false positives, false negatives, and (if relevant) true negatives.
- Prefer a mix of casing, punctuation, whitespace, and nearby-token variations.

4. Sanity-check edge cases
- Ensure exceptions do not block valid matches.
- Ensure replacements are correct and not destructive.

5. Run the tests.
- Fix any issues that arise.

### Output Format

Write a Weir rule to a new file with a name of your choosing, including `expr main`, `let` fields, and tests. Make sure it has the extension `.weir`.
