---
title: Website
---

This page describes how to develop [writewithharper.com](https://writewithharper.com) locally.
If you are comfortable with JavaScript or TypeScript but new to Svelte, start with [Prerequisites](#Prerequisites) and [Running the Site](#Running-the-Site), then skim [Where Code Lives](#Where-Code-Lives).

Make sure you read the [introduction to contributing](./introduction) and [committing](./committing) guides before opening a pull request.

## Notes

- Almost all website code lives under [`packages/web`](https://github.com/Automattic/harper/tree/master/packages/web).
- The site is a [SvelteKit](https://kit.svelte.dev/) app. Documentation pages use [SveltePress](https://sveltepress.site/) (Markdown routes under `src/routes/docs/`).
- The homepage demo and `/editor` route use workspace packages [`harper.js`](../../harperjs/introduction), [`lint-framework`](https://github.com/Automattic/harper/tree/master/packages/lint-framework), [`harper-editor`](https://github.com/Automattic/harper/tree/master/packages/harper-editor), and [`components`](https://github.com/Automattic/harper/tree/master/packages/components). `just dev-web` builds them for you.
- You can look at the project's [`justfile`](https://github.com/Automattic/harper/blob/master/justfile) to see exactly what the `just` recipes below do.
- CI builds the production site with the [`Build Web`](https://github.com/Automattic/harper/blob/master/.github/workflows/build_web.yml) workflow (Docker image from the root [`Dockerfile`](https://github.com/Automattic/harper/blob/master/Dockerfile)).

## Prerequisites

- [Set up your environment](./environment). For website-only work you need `node`, `pnpm`, and `just` in your `$PATH`. You do **not** need a full Rust toolchain unless you are changing `harper-wasm` or `harper-core`.
- From the repository root, run `just setup` once (or at least `just build-web`) so workspace dependencies and WASM artifacts are built.
- Node **22+** and **pnpm 10** match what Harper uses in CI (see the root `package.json` `engines` field).

## Where Code Lives

| Path | Purpose |
| --- | --- |
| `packages/web/src/routes/` | Site routes: marketing pages (`.svelte`), docs (`.md`), and API handlers (`+server.ts`) |
| `packages/web/src/routes/docs/` | Contributor and user documentation (this page is `contributors/website/+page.md`) |
| `packages/web/src/lib/` | Shared Svelte components, marketing sections, and small utilities |
| `packages/web/vite.config.ts` | Vite config, SveltePress theme, and **documentation sidebar** |
| `packages/lint-framework/` | Browser linting UI (underlines, popups) reused by the site and extensions |
| `packages/harper-editor/` | Embeddable Harper editor used on the homepage and `/editor` |
| `packages/components/` | Shared UI primitives consumed by the site and other packages |
| `packages/harper.js/` | JavaScript API over `harper-wasm` |

Documentation sidebar entries are defined in `packages/web/vite.config.ts`. When you add a new doc page, register it there so it appears in the left nav.

## Running the Site

The recommended way to start a dev server:

```bash
just dev-web
```

This builds `harper.js`, `lint-framework`, `components`, and `harper-editor`, then runs `pnpm dev` in `packages/web`.

Open [http://localhost:3000](http://localhost:3000). Vite hot-reloads most edits to `.svelte`, `.ts`, and `.md` files automatically.

> `just build-web` only produces a production build; it does **not** start a dev server. Use `just dev-web` when you want to preview changes interactively.

### If You Change Lower-Level Packages

- Edits under `packages/web` usually reload without restarting.
- If you change `packages/components`, `packages/harper-editor`, or `packages/lint-framework`, rebuild that package (or re-run `just dev-web`).
- If you change `harper-wasm` / `harper-core`, re-run `just dev-web` (or `just build-harperjs`) so the site picks up a new WASM build.

### Manual Alternative

From the repo root:

```bash
just build-harperjs build-lint-framework build-components build-harper-editor
cd packages/web
pnpm install
pnpm dev
```

## Previewing Your Changes

| What you changed | Where to look |
| --- | --- |
| A documentation page | `http://localhost:3000/docs/...` (path mirrors `src/routes/docs/...`) |
| Homepage or marketing UI | `http://localhost:3000/` |
| Live grammar demo / editor | `http://localhost:3000/` (demo) or `http://localhost:3000/editor` |
| Rules catalog | `http://localhost:3000/docs/rules` |
| Weir studio | `http://localhost:3000/weir/studio` |

SveltePress doc pages support standard Markdown plus SveltePress features (admonitions, embedded code, etc.). Use an existing page under `src/routes/docs/` as a template.

Each doc page has an **Edit this page** link (configured in `vite.config.ts`) that opens the matching file on GitHub.

## Svelte and SvelteKit (For JS/TS Developers)

You do not need deep Svelte expertise for many contributions.

- **Routes**: Files in `src/routes/` map to URLs. `+page.svelte` is a page component; `+page.md` is a SveltePress doc page; `+server.ts` is a server endpoint.
- **Components**: Reusable UI lives in `src/lib/`. Import with `$lib/...` (SvelteKit alias).
- **Scripts**: `<script lang="ts">` holds component logic. Harper uses **Svelte 5** (runes like `$state` appear in newer components).
- **Styles**: Tailwind CSS v4 is enabled via `@tailwindcss/vite` in `vite.config.ts`. Component-scoped `<style>` blocks are also used.

Official references: [Svelte tutorial](https://svelte.dev/tutorial), [SvelteKit docs](https://kit.svelte.dev/docs).

## Testing and Validation

Before opening a pull request:

```bash
just format
just precommit
```

For website-focused checks:

```bash
cd packages/web
pnpm check
```

`pnpm check` runs `svelte-check` (TypeScript and Svelte diagnostics). The root `just check` recipe also runs `pnpm check` for the whole workspace and includes `just build-web`.

There is no dedicated website unit-test package today; rely on `pnpm check`, a manual click-through in the browser, and CI.

### Production Build Smoke Test

```bash
just build-web
cd packages/web
pnpm preview
```

Then open the URL printed by `vite preview` (defaults differ from dev; confirm in the terminal output).

### Docker (Optional)

Reviewers sometimes test grammar changes via the site demo using Docker ([see also the review guide](./review#Testing-Via-the-Docker-Image)):

```bash
docker build . -q
docker run -p 3000:3000 -it <image-id>
```

You do not need Docker for everyday website UI or documentation work.

## Server Features (Usually Optional)

A few routes persist data with **MariaDB** via Drizzle ORM (for example uninstall feedback and problematic-lint reports). Local API work is optional:

1. Start a database: `docker compose -f docker-compose.dev.yml up` from the repo root.
2. Set `DATABASE_URL` for `packages/web` (see `packages/web/drizzle.config.ts`).
3. Run migrations on app start (`src/hooks.server.ts`).

Most contributors never need this to change docs, marketing pages, or the public demo.

## Deployment

Merges to `master` trigger the **Build Web** workflow, which builds the root `Dockerfile` and publishes the site image. You do not need to deploy manually to test a pull request; maintainers handle production releases.

## Related Reading

- [Harper's architecture](./architecture) — how `harper-core`, `harper.js`, and integrations fit together
- [Reviewing pull requests](./review) — testing patches, including the Docker demo workflow
- [`harper.js` documentation](../harperjs/introduction) — if you are wiring lint behavior on the site
