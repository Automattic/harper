# HarperMac

**HarperMac:** **Mac app** · [Chrome extension](../harper-mac-chrome) · [Safari extension](../harper-mac-safari)

System-wide grammar and spelling checking for macOS, powered by [Harper](https://writewithharper.com).
Everything runs on your Mac; nothing you type is sent anywhere.

HarperMac is a menu-bar app. As you type in almost any app, it underlines mistakes with a wavy
line. Click an underlined word to see what's wrong and pick a fix.

## What you need

- macOS 13 or later
- Swift (the Xcode Command Line Tools are enough: `xcode-select --install`)
- A copy of the `harper-ls` binary. Either:
  - download a prebuilt one with `tools/fetch_harper.sh` (needs the [GitHub CLI](https://cli.github.com), `gh`), or
  - build it from this repository: `cargo build --release -p harper-ls`, then copy
    `target/release/harper-ls` to `harper-mac/vendor/harper-ls/harper-ls`.

## Build and run

From this folder:

```bash
tools/fetch_harper.sh     # downloads harper-ls into vendor/
./build_app.sh            # builds build/HarperMac.app with harper-ls bundled inside
open build/HarperMac.app
```

The first time it runs, macOS asks for **Accessibility** access. Approve it in
*System Settings > Privacy & Security > Accessibility*. This is how the app reads the text you are
typing and works out where each word is on screen. Without it, nothing can be checked.

> The app is signed ad hoc, so macOS treats each rebuild as a new app. If underlines stop
> appearing after a rebuild, remove HarperMac from the Accessibility list, add it again, and relaunch.

## Using it

Click the menu-bar icon to:

- **Check my writing**: master on/off switch.
- **Check in <app>**: turn checking on or off for the app you were just using. Terminals and code
  editors start off, because they would be checked on every keystroke.
- **Spelling**: choose American, British, Australian, Canadian or Indian English (British by default).
- **Open at login**.

Click an underlined word to see the explanation and suggestions. You can also **add the word to your
dictionary** or **ignore** that suggestion.

## Browsers

Web pages don't expose their text to macOS very reliably, so there are companion browser extensions
that draw the underlines inside the page itself. They use this app as their grammar engine, so the
app **must be running**:

- [Chrome extension](../harper-mac-chrome) (also Brave, Edge and Arc)
- [Safari extension](../harper-mac-safari)

While an extension is handling a text field, the app stays out of the way for that browser so
nothing is underlined twice. Anywhere the extension can't run, the app still covers it.

## How it works

| File | What it does |
|---|---|
| `HarperServer.swift` | Starts `harper-ls` and talks to it (JSON over a pipe) |
| `Accessibility.swift` | Reads the focused text field, finds where words are on screen, applies fixes |
| `Checker.swift` | Watches the field, waits for you to pause typing, runs a check, handles clicks |
| `Overlay.swift` | Click-through windows that draw the wavy underlines |
| `SuggestionPanel.swift` | The popup with the explanation and suggestions |
| `Bridge.swift` | A local-only web server the browser extensions use to reach the grammar engine |
| `AppDelegate.swift`, `Settings.swift`, `main.swift` | Start-up, the menu-bar icon, saved preferences |

Fixes are applied by selecting the word and replacing it through the Accessibility API. If an app
ignores that, the app falls back to pasting the fix (and restores your clipboard afterwards).

## The browser bridge

`Bridge.swift` listens on `http://127.0.0.1:47321` and nowhere else. It only answers requests that
come from a browser extension and carry an `X-HarperMac: 1` header, and it rejects any other
`Host`, so an ordinary web page can't use it. To try it by hand while testing, start the app with
`--allow-origin=<your test page's origin>` and `--port=<n>`.

## Known limits

- Not every app exposes its text. Native Mac apps work best; browsers and Electron apps work better
  with the extensions; apps that draw their own text may not work at all.
- Underlines are drawn above all windows, so a window covering a field can show them through.
- Password fields and URL-looking text are skipped on purpose.

## Licence

Apache-2.0, the same as Harper.
