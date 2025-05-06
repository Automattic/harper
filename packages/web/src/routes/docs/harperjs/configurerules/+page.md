---
title: Configure Rules
---

We add new [rules](/docs/rules) to Harper on a daily basis.
As such, it is not recommended for consumers of `harper.js` to rely on any rule to exist.
Further, consumers should allow space (in their UI, database, etc.) for additional rules to be added whenever a new version of `harper.js` is published.

To make this easier, `harper.js` exposes a [`LintConfig`](/docs/harperjs/ref/harper.js.lintconfig.html) type, which can be obtained via `Linter.getLintConfig` and written using `Linter.setLintConfig`.

Each key refers to a specific rule. Each rule can be disabled (set the value to `false`), enabled (set the value to `true`), or to the default (set the value to `undefined`).
For example, the following code disables `SpellCheck`, enables `ExplanationMarks`, and sets `SameAs` to assume the default value.

```javascript
let linter = new WorkerLinter();

await linter.setLintConfig({
    SpellCheck: false,
    ExplanationMarks: true,
});
```
