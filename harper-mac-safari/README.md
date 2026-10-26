# HarperMac for Safari

**HarperMac:** [Mac app](../harper-mac) · [Chrome extension](../harper-mac-chrome) · **Safari extension**

Grammar and spelling checking in Safari, using the [HarperMac app](../harper-mac) as the engine.
Nothing leaves your Mac. **The HarperMac app must be running.**

This is the same web extension as the [Chrome extension](../harper-mac-chrome), wrapped for Safari.
Safari extensions have to ship inside a Mac app, so `build_safari.sh` uses Apple's converter to wrap
the extension in a small host app. This folder contains only that build script; the extension code
lives in [`harper-mac-chrome`](../harper-mac-chrome).

## What you need

- A Mac with **Xcode** installed (the Command Line Tools alone aren't enough for the converter).
  Xcode doesn't need to be your active developer directory; the script finds `/Applications/Xcode.app`
  (set `XCODE=/path/to/Xcode.app` to use another).
- The [HarperMac app](../harper-mac), built and running.

## Build and install

```bash
./build_safari.sh
open "build/HarperMac Safari.app"    # run it once so Safari discovers the extension
```

You can close the small window it opens. Then in Safari:

1. **Safari > Settings > Advanced**, tick **Show features for web developers**.
2. **Develop > Allow Unsigned Extensions** (asks for your password).
3. **Safari > Settings > Extensions**, tick **HarperMac**.
4. Allow it on websites (**Always Allow on Every Website**), and allow `127.0.0.1` when asked.

The build is signed ad hoc, so step 2 is needed and **resets every time Safari quits**. Signing with
your own Apple developer identity removes that step.

## Troubleshooting

- **Extension not listed:** quit and reopen Safari, then check *Settings > Extensions* again.
- **Toolbar button says the app isn't running:** start HarperMac, and check Safari is allowed to
  reach `127.0.0.1`.
- **Build says Xcode not found:** install Xcode, or set `XCODE`.

## Licence

Apache-2.0, the same as Harper.
