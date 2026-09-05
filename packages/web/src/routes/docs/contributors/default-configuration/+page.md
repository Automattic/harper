---
title: Default Configuration
---

`harper-core/default_config.json` decides which of Harper's [rules](../rules) are enabled out of the box.
It also decides how those rules are grouped and labelled everywhere a user browses them, including the Chrome extension's options page, the Obsidian plugin's settings tab, and Harper Desktop.

Read this page before editing that file.
Adding a rule without registering it there fails the test suite, and writing a malformed setting panics Harper on startup.

## Two Views of the Same Configuration

Harper keeps rule configuration in two shapes, and it helps to know which one you are looking at.

A [`FlatConfig`](https://docs.rs/harper-core/latest/harper_core/linting/struct.FlatConfig.html) is what the engine actually runs on.
It maps a rule name to enabled, disabled, or unset, and it knows nothing about grouping or presentation.
A rule with no entry in the map is treated as disabled.

A [`StructuredConfig`](https://docs.rs/harper-core/latest/harper_core/linting/struct.StructuredConfig.html) is the shape humans and settings UIs work with.
It nests rules into labelled groups, carries display text, and can express a relationship between rules that a flat map has no way to represent.

`default_config.json` is the serialized form of the second one.
`LintGroup::new_curated` deserializes the file, flattens it, and runs on the result, so the file is not a description of the curated defaults.
It is the definition of them.

## Setting Types

The top level of the file is a `settings` array.
Every element is an object with exactly one key, and that key names the type of setting.
There are three.

### `Group`

A labelled container for other settings.
Groups are what produce the collapsible sections in the settings UIs, and the top level of `default_config.json` is entirely groups.

```json
{
	"Group": {
		"label": "Proper Nouns",
		"description": "Checks names of places, organizations, products, and other proper nouns that should keep their standard capitalization.",
		"child": {
			"settings": []
		}
	}
}
```

| Field | Required | Meaning |
| --- | --- | --- |
| `description` | yes | Prose shown alongside the group. |
| `child` | yes | A nested object with its own `settings` array. |
| `label` | no | The group's heading. Defaults to an empty string. |

Groups nest, so a `child` may contain further groups, though the shipped file is currently only one level deep.

### `Bool`

A single rule the user turns on or off.
This is the overwhelming majority of the file.

```json
{
	"Bool": {
		"name": "AmazonNames",
		"state": true,
		"label": "Amazon Names"
	}
}
```

| Field | Required | Meaning |
| --- | --- | --- |
| `name` | yes | The rule's registered name, exactly as passed to `insert_expr_rule!` or `insert_struct_rule!`. |
| `state` | yes | Whether the rule is enabled by default. |
| `label` | no | Display text. Falls back to `name`. |

### `OneOfMany`

A set of rules where at most one may be enabled.
Think of it as a dropdown rather than a row of switches.

Reach for it when two or more rules encode opposing preferences and enabling both would produce contradictory lints.
A rule that enforces the Oxford comma and a rule that enforces its absence will each flag the other's preferred output, so offering them as two independent checkboxes lets a user select a combination that can never be satisfied.
`OneOfMany` removes that combination from the interface entirely.

```json
{
	"OneOfMany": {
		"names": ["PreferFoo", "PreferBar"],
		"name": "PreferFoo",
		"labels": ["Prefer “Foo”", "Prefer “Bar”"]
	}
}
```

| Field | Required | Meaning |
| --- | --- | --- |
| `names` | yes | The registered names of the rules to choose between, in the order they should be presented. |
| `name` | no | The name that is selected by default. Must be one of `names`. Omit it, or set it to `null`, to start with nothing selected. |
| `labels` | no | Display text, one entry per element of `names`, in the same order. Falls back to `names`. |

#### What Flattening Produces

Only the selected rule is written into the `FlatConfig`, and it is written as enabled.
The rules that were not selected get no entry at all, which the engine reads as disabled.
There is therefore no way to express "two of these three" with a single `OneOfMany`, and no way for an unselected alternative to fall back to some other default.

When `name` is absent or `null`, nothing at all is written, so every rule in the set is off.
That is a legitimate state, so nothing downstream should assume a selection exists.

Reading in the other direction, from a `FlatConfig` back into a `StructuredConfig`, the selection becomes the **first** name in `names` that is currently enabled.
If some other code path has enabled two rules from the same set, the later one is silently dropped on the next read.
Order `names` deliberately rather than alphabetically by habit.

#### Requirements

Two invariants are checked, and both are easy to break by hand.

- `labels`, when present, must have exactly as many entries as `names`.
- `name`, when present, must appear in `names`.

A violation makes `StructuredConfig::curated()` return nothing, and `LintGroup::new_curated` unwraps that, so Harper panics the first time anything builds a curated linter rather than failing gracefully.
Run `cargo test -p harper-core -- structured_config` to catch it.

#### The Rules Must Be Registered Separately

`OneOfMany` selects between rules by name, so each alternative needs its own registered name in `LintGroup::new_curated`.

This rules out `merge_linters!`.
That macro exists to collapse several sub-linters behind one public name, which is usually what you want, but it leaves the configuration system with a single name and therefore nothing to choose between.
If you are converting a merged rule into a set of alternatives, splitting it into separately registered rules is part of the work, not an incidental detail.

## Keeping the File in Sync

`curated_default_config_lists_every_registered_rule` compares the rule names in `default_config.json` against the rules actually registered at runtime and fails on any difference in either direction.
A new rule that is not in the file will fail it, and so will a name left behind in the file after a rule is renamed or removed.
Every name inside a `OneOfMany`'s `names` array counts, so splitting a merged rule means adding each half.

## Downstream Consumers

The same three setting types cross into JavaScript.
`harper.js` exposes them through `Linter.getStructuredLintConfig`, documented under [configuring rules](../harperjs/configurerules#Structured-Configuration).
Anything reading that structure has to handle all three, and a consumer written before `OneOfMany` existed will quietly skip those rules rather than fail loudly.

## Related Reading

- [Author a Rule](./author-a-rule) for registering a rule in the first place
- [Configure Rules](../harperjs/configurerules) for the `harper.js` side
