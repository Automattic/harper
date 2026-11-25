# docs-sync

Builds the Harper documentation in [`docs/`](../../docs) into pages for
[writewithharper.com](https://writewithharper.com/docs/about) and sends them there.

The website is a WordPress site. Its `harper-docs` plugin, in the
[wpcomvip/harper-wp](https://github.com/wpcomvip/harper-wp) repository, receives what this
package sends. The Markdown here stays the source of truth: anything edited on the site is
replaced by the next publish.

## Writing docs

One Markdown file per page. `docs/integrations/obsidian.md` is published at
`/docs/integrations/obsidian`.

- **Frontmatter.** Every page starts with a `title` and a one-sentence `description`.
- **Plain Markdown.** No HTML and no Svelte. To show literal angle brackets, put them in
  backticks.
- **Links.** Link to another page with a relative link to its file, such as
  `[the language server](./language-server.md#Configuration)`. Links to any other file in
  the repository become links to GitHub.
- **Headings.** Each heading gets an id made from its text, with capitals kept and
  punctuation dropped: "Code Actions" is `#Code-Actions`.
- **Images.** Put the file in `packages/web/static/images` and write `/images/name.png`.
- **Sidebar.** Add a new page to `docs/sidebar.yml`, or the build fails. A folder of pages
  needs a group with a `section` in that file.

Three things Markdown cannot say are written as code fences with a special label. On GitHub
they read as ordinary code blocks.

| Write | To get |
| --- | --- |
| An empty fence labelled <code>ts file=packages/harper.js/src/Linter.ts</code> | That file's contents, highlighted. The path starts at the repository root. |
| A fence labelled <code>harper-editor</code> holding sample text | The live editor, started with that text. |
| An empty fence labelled <code>harper-rules</code> | The table of every rule, generated from `harper.js`. |

## Building

The rule table and the API reference come from `harper.js`, so build that first.

```bash
just build-harperjs
pnpm --filter docs-sync build
```

This writes `packages/docs-sync/dist/bundle.json` and fails, listing every problem at once,
if the docs have a broken link, a link to a heading that does not exist, a missing image,
raw HTML, or a page missing from the sidebar. `client-links.json` lists the headings that
installed Harper clients open directly. Renaming one of those fails the build too.

```bash
pnpm --filter docs-sync test
```

## Publishing

The [Docs workflow](../../.github/workflows/docs_sync.yml) builds on every pull request
that touches the docs and publishes on every push to `master`. It can also be run by hand.

Publishing sends the site a list of every page with a hash, uploads only the pages the
site does not already have in that form, then lets the site remove pages that are gone.
Publishing twice in a row changes nothing the second time.

No secret is involved. The site checks the OpenID Connect token that GitHub issues to the
workflow run, and accepts one only from `docs_sync.yml`, on `master`, in this repository.
The site's address is the repository variable `DOCS_SITE_URL`.

The site filters everything it receives, so a page cannot carry a script or a style onto
the website whatever this package sends.

To see what a publish would change without changing anything:

```bash
node src/publish.js --url https://writewithharper.com --dry-run
```

Outside GitHub Actions that needs a token the site accepts, in `HARPER_DOCS_SYNC_TOKEN`.
Only a local development site issues those.
