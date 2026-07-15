---
title: Website
---

This page describes how to develop [writewithharper.com](https://writewithharper.com) locally.
Make sure you read the [introduction to contributing](./introduction) and [committing](./committing) guides before opening a pull request.

## Environment

Make sure you have [Set up your environment](./environment). 

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

Documentation sidebar entries are defined in `packages/web/vite.config.ts`. When you add a new doc page, register it there so it appears in the left navigation pane.

## Updating the Documentation

The Harper documentation (available on this site) is in `web/src/routes/docs`. It is written in Markdown.

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

Admin pages, available at `/admin` are hidden by default, but can be enabled by setting the `ENABLE_ADMIN_ROUTES=true` environment variable.

## Deployment

Merges to `web-prod` trigger the **Build Web** workflow, which builds the root `Dockerfile` and publishes the site image via a private TeamCity instance.

## Related Reading

- [Harper's architecture](./architecture) — how `harper-core`, `harper.js`, and integrations fit together
- [Reviewing pull requests](./review) — testing patches, including the Docker demo workflow
- [`harper.js` documentation](../harperjs/introduction) — if you are wiring lint behavior on the site
