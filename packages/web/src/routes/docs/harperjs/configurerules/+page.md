---
title: Configure Rules
---

We add new [rules](/docs/rules) to Harper on a daily basis.
As such, it is not recommended for consumers of `harper.js` to rely on any rule to exist.
Further, consumers should allow space (in their UI, database, etc.) for additional rules to be added whenever a new version of `harper.js` is published.

To make this easier, `harper.js` exposes a [`LintConfig`](/docs/harperjs/ref/harper.js.lintconfig.html) type, which can be obtained via `Linter.getLintConfig` and written using `Linter.setLintConfig`.

Each key refers to a specific rule. Each rule can be disabled (set the value to `false`), enabled (set the value to `true`), or reset to the default (set the value to `null`).
For example, the following code disables `SpellCheck`, enables `ExplanationMarks`, and sets `SameAs` to assume the default value.

```javascript
import { WorkerLinter } from 'harper.js';
import { binary } from 'harper.js/binary';

let linter = new WorkerLinter({ binary });

await linter.setLintConfig({
    SpellCheck: false,
    ExplanationMarks: true,
    SameAs: null,
});
```

## Structured Configuration

A `LintConfig` is a flat map, which is all the linter needs but not enough to build a settings screen from.
For that, `harper.js` also exposes a read-only structured view via `Linter.getStructuredLintConfig`, or `Linter.getStructuredLintConfigJSON` if you would rather have the raw string.
It carries the same rules, arranged into labelled groups with display text, and it is the source Harper's own extensions render their options pages from.

There is no structured setter.
Read the structure to decide what to draw, then write changes back through `setLintConfig` using the rule names it gave you.

Every entry in a `settings` array is one of three shapes, distinguished by its single key.

- `Bool` holds one rule the user toggles. It has a `name`, a `state`, and an optional `label`.
- `Group` holds a `label`, a `description`, and a `child` with its own nested `settings`.
- `OneOfMany` holds a set of rules of which at most one may be enabled.

Handle all three.
The set is defined upstream and grows, so treat an unrecognized key as something to skip rather than something to crash on.

### `OneOfMany`

`OneOfMany` exists for rules that contradict each other, such as one rule that enforces the Oxford comma and another that enforces its absence.
Rendering those as two independent switches lets a user pick a combination that can never be satisfied, so Harper groups them and asks for a single choice.

```typescript
interface StructuredLintOneOfManySetting {
	OneOfMany: {
		names: string[];
		name?: string | null;
		labels?: string[] | null;
	};
}
```

`names` holds the rule names, in presentation order.
`labels`, when present, holds one display string per name in the same order, and falls back to `names`.
`name` is whichever rule in the set is currently enabled, and it is `null` when none of them is.
That is a real state your UI has to be able to show, so do not assume a selection exists.

A dropdown is the natural control.
Because `setLintConfig` replaces the configuration wholesale, write back every name in the set rather than only the one that changed.

```javascript
async function chooseOneOf(linter, setting, selected) {
	const config = await linter.getLintConfig();

	for (const name of setting.names) {
		config[name] = name === selected;
	}

	await linter.setLintConfig(config);
}
```

Setting the unselected names to `false` is the part worth remembering.
Leaving them alone keeps whatever they were before, which is how a user ends up with two conflicting rules both reporting on the same sentence.
To hand the whole set back to Harper's defaults rather than choosing within it, set every name to `null` as described above.

Contributors adding to the set of rules on the Rust side should see [the default configuration reference](../contributors/default-configuration).
