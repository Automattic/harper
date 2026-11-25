---
title: Website
description: Develop the Harper website locally with SvelteKit and SveltePress, and find its source files and build commands.
---

# Website

This page describes how to develop [writewithharper.com](https://writewithharper.com) locally.
Make sure you read the [introduction to contributing](./introduction.md) and [committing](./committing.md) guides before opening a pull request.

## Environment

Make sure you have [Set up your environment](./environment.md). 

## Where Code Lives

| Path | Purpose |
| --- | --- |
| `packages/web/src/routes/` | Site routes: marketing pages (`.svelte`) and API handlers (`+server.ts`) |
| `docs/` | Contributor and user documentation (this page is `docs/contributors/website.md`) |
| `packages/docs-sync/` | Builds the documentation and publishes it to the website |
| `packages/web/src/lib/` | Shared Svelte components, marketing sections, and small utilities |
| `packages/web/vite.config.ts` | Vite config |
| `packages/lint-framework/` | Browser linting UI (underlines, popups) reused by the site and extensions |
| `packages/harper-editor/` | Embeddable Harper editor used on the homepage and `/editor` |
| `packages/components/` | Shared UI primitives consumed by the site and other packages |
| `packages/harper.js/` | JavaScript API over `harper-wasm` |

## Updating the Documentation

The Harper documentation (available on this site) is in the `docs` folder at the root of the repository.
Each page is one Markdown file: `docs/integrations/obsidian.md` is published at `/docs/integrations/obsidian`.

The docs are plain Markdown, with no HTML and no Svelte.
When you add a page, register it in `docs/sidebar.yml` so it appears in the left navigation pane.
Link to other pages with a relative link to the file, such as `[the language server](../integrations/language-server.md)`.

`packages/docs-sync` checks the docs on every pull request and publishes them when a change merges to `master`.
Its [README](../../packages/docs-sync/README.md) describes the format in full, including how to show a file from the repository on a page.

## Running the Site

The recommended way to start a dev server:

```bash
just dev-web
```

This builds all the site's dependencies and runs a development server.
You do not need to rerun anything to hot-reload changes to the `web` package.
If you make changes to a dependency package (i.e. `components`, `harper-editor`), you MUST rerun `just dev-web` to see the changes appear on the development site.

> `just build-web` only produces a production build; it does **not** start a dev server. Use `just dev-web` when you want to preview changes interactively.

## Database

The web application needs a database available for some functionality, but not most.
You can spin up a development database locally using `docker-compose.dev.yml`.
You can make a copy of `docker-compose.yml` to run a full instance of the entire web application.

## Admin Pages

Admin pages, available at `/admin` are hidden by default, but can be enabled by setting the `ENABLE_ADMIN_ROUTES=true` environment variable before building the site.
At the time of writing, they are not admin routes as much as they are insights into the reports submitted by users.

## Deployment

Merges to `web-prod` trigger the **Build Web** workflow, which builds the root `Dockerfile` and publishes the site image via a private TeamCity instance.

## Related Reading

- [Harper's architecture](./architecture.md) — how `harper-core`, `harper.js`, and integrations fit together
- [Reviewing pull requests](./review.md) — testing patches, including the Docker demo workflow
- [`harper.js` documentation](../harperjs/introduction.md) — if you are wiring lint behavior on the site
