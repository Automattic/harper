---
title: Local Statistics
---

Harper keeps track of certain aspects of your writing.
Things like:

- What words are misspelled most often?
- How often do you accept Harper's suggestions?
- How much do you write?

Harper does this to help _you_ improve _your_ writing.
In the interest of maintaining our user's data sovereignty, Harper aims to do all this processing __on the device__.
None of this data is sent anywhere without your explicit permission.

This document seeks to detail how Harper's statistics logging works under the hood.

## The `stats.txt` File.

The `stats.txt` file (whose name is subject to change) is a log of actions taken by Harper or the user.
It records specific events, along with some contextual information (like which word was misspelled).

The `stats.txt` file is formatted into lines (so it is easy to open in append-mode), each containing a JSON object.

```
{"kind":{"Lint":{"kind":"Spelling","context":[{"content":"mispelled","kind":{"kind":"Word","value":null}}]}},"when":1743696274,"uuid":"39d29bd0-5eb1-4bad-89ee-5a48531a4cbe"}
{"kind":{"Lint":{"kind":"Spelling","context":[{"content":"isnt","kind":{"kind":"Word","value":null}}]}},"when":1743696281,"uuid":"22e1ca15-e583-49c5-9da3-bc7e625d9682"}
{"kind":{"Lint":{"kind":"Spelling","context":[{"content":"Teasting","kind":{"kind":"Word","value":null}}]}},"when":1743696288,"uuid":"bd955190-a4d9-4f3e-b7df-d4bf6f12a415"}
```

In `harper-ls` it is written to the Harper `data` directory. (On Linux `~/.local/share/harper-ls/stats.txt`, on macOS `~/Library/Application Support/harper-ls/stats.txt`, on Windows `%FOLDERID_LocalAppData%/harper-ls/stats.txt`)
In `harper.js` it is available through methods of objects that implement the [`Linter`](/docs/harperjs/ref/harper.js.linter.html) interface.

A simple dashboard to view a summary of these statistics is available [on our website](/stats).
