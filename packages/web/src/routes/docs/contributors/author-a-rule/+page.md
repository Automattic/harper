---
title: Author a Rule
---

[Harper's grammatical rules are many](../rules), but most are relatively easy to understand.
Before we get into how to write a rule, it is important that we get some of the language cleared up.

When we refer to a Harper rule, we are talking about [an implementation of the Linter trait](https://docs.rs/harper-core/latest/harper_core/linting/trait.Linter.html).
As you can see, there is an enormous amount of flexibility in this trait and a wide variety of potential strategies for querying the provided document to locate errors.

This guide will go through one easy way to add a complex rule to Harper.
The lofty goal is for this to be doable by someone with little to no Rust experience.
You should, however, be able to figure out how to use Git.

While this guide should be enough to get stared, [others](https://elijahpotter.dev/articles/writing_a_grammatical_rule_for_harper) have been written.

## Fork the Harper Monorepo

Before you can open a pull request or modify any code, you need a mutable copy of our monorepo.
The best way to do that is to [fork it in GitHub](https://github.com/Automattic/harper/fork).

Next, you'll want to copy this fork onto your computer and create a new branch.
GitHub has an [excellent page explaining how to clone repositories](https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository).

## Get Your Environment Set Up

Please read our [guide for getting your environment set up](./environment).

## Open a Draft Pull Request

Next, you'll want to open a draft pull request.
This gives us (the Harper maintainers) a better view of what is actively being worked on.
It also makes it much easier to ask questions about how Harper works while you're working on your rule.
[This page has more detail on why we want draft pull requests as early as possible.](https://elijahpotter.dev/articles/never_wait).

GitHub has some [good documentation on how to create a draft PR](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request-from-a-fork) if this is your first time.

## Determine Your Rule's Needed Complexity

A vast plurality of potential grammatical rules are pretty simple.
If you're trying to extend Harper to identify a given phrase (like "all of the sudden") and replace it with something else (like "all of a sudden"), you can do this without any complex programming at all.
All you have to do is add a line to `harper-core/src/linting/phrase_corrections/mod.rs`:

```rust
"AllOfASudden" => (
    // The offending phrase
    ["all of the sudden"],
    // The correct phrase
    ["all of a sudden"],
    // The message to notify the user of the error
    "The phrase is `all of a sudden`, meaning `unexpectedly`.",
    // A description of the rule.
    "Corrects `all of the sudden` to `all of a sudden`."
),
```

This method also covers more complex cases, like if one of the words contains capitalization or the phrase is split by a line break.

You can actually add multiple offending phrases and correct phrases to the same rule:

```rust
"EnMasse" => (
    // Multiple offending phrases
    ["on mass", "on masse", "in mass"],
    ["en masse"],
```
```rust
"InOfItself" => (
    ["in of itself"],
    // Multiple correct phrases
    ["in itself", "in and of itself"],
```

For more complex corrections with multiple variants, singular and plural, or multiple verb tenses, you can use `harper-core/src/linting/phrase_set_corrections/mod.rs`.

It has two sections. The `add_1_to_1_mappings` section is simpler, supporting sets of corrections with a single name, where each pair is a single offending phrase and a single correct phrase.
```rust
"Ado" => (
    &[
        // Multiple variants but only one offending phrase
        // and one correct phrase
        ("further adieu", "further ado"),
        ("much adieu", "much ado"),
    ],
    "Use `ado` (meaning 'fuss') not `adieu` (meaning 'farewell').",
    "Corrects `adieu` to `ado` in common phrases."
),
```

The `add_many_to_many_mappings` section is more complex. Each set still has a single name, but each pair can have multiple offending phrases and multiple correct phrases.
```rust
"ChangeTack" => (
    &[
        // Both multiple variants and also multiple offending
        // phrases and/or correct phrases
        (&["change tact", "change tacks"], &["change tack"]),
        (&["changed tact", "changed tacks"], &["changed tack"]),
    ],
    "A change in direction is a change of `tack` (not `tact`).",
    "Corrects the idiom `change tack`."
),
```


Similarly, if you just want Harper to enforce proper capitalization of a multi-token proper noun (like "Tumblr Blaze") you just need to add an entry to `harper-core/proper_noun_rules.json`.

```javascript
// The name of the rule
"TumblrNames": {
    // The canonical capitalization of the proper noun.
	"canonical": [
		"Tumblr Blaze",
		"Tumblr Pro",
		"Tumblr Live",
		"Tumblr Ads",
		"Tumblr Communities",
		"Tumblr Shop",
		"Tumblr Dashboard"
	],
    // A description to be shown to the user when they make a mistake.
	"description": "Ensure proper capitalization of Tumblr-related terms."
},
```

If neither of those work for the rule you have in mind, continue on to the next section.

## Create Your Rule's Module

Now that we've established that your rule is of a non-trivial level of complexity, here is what you need to do.

We separate each rule into its own file inside the `harper-core/src/linting` [directory.](https://github.com/Automattic/harper/tree/master/harper-core/src/linting)
Create a new file under that directory with the name of your rule in `snake_case`.
If you can't decide yet, just call it `my_rule.rs`.

Don't put anything in this file yet, there's some bookkeeping we have to do first.

## Register Your Rule

Before we start describing to Harper what grammatical errors to look for, we need to register your rule within the system.

First, add your rule's module to the tree by adding it to [the top of the mod file](https://github.com/Automattic/harper/blob/master/harper-core/src/linting/mod.rs).
It should look something like this:

```rust title="harper-core/src/linting/mod.rs"
mod an_a;
mod avoid_curses;
mod boring_words;
mod capitalize_personal_pronouns;
// [svp! df:+]mod my_rule;
// [svp! df:+]pub use my_rule::MyRule;
```

Next, we need to configure whether your rule will be enabled by default.
While you're working on it, we **highly suggest** you enable it to avoid confusion.

To do that, import your rule at the top of the `lint_group` [file](https://github.com/Automattic/harper/blob/master/harper-core/src/linting/mod.rs).

```rust title="harper-core/src/linting/lint_group.rs"
use super::an_a::AnA;
use super::avoid_curses::AvoidCurses;
use super::boring_words::BoringWords;
use super::capitalize_personal_pronouns::CapitalizePersonalPronouns;
use super::correct_number_suffix::CorrectNumberSuffix;
// [svp! df:+]use super::my_rule::MyRule;
```

Finally, enable it in a macro invocation near the bottom:

```rust title="harper-core/src/linting/lint_group.rs"
insert_struct_rule!(AdjectiveOfA, true);
insert_expr_rule!(BackInTheDay, true);
insert_struct_rule!(WordPressDotcom, true);
insert_expr_rule!(OutOfDate, true);
// [svp! df:+]insert_expr_rule!(MyRule, true);
```

If you use a `ExprLinter`, use `insert_expr_rule` to take advantage of Harper's aggressive caching.
Otherwise, use `insert_struct_rule`.

## Write Your Rule

Defining an expression and [implementing the ExprLinter trait](https://docs.rs/harper-core/latest/harper_core/linting/trait.ExprLinter.html) is the easiest way to define a new rule for Harper.
Here's a template to get you started:

```rust title="my_rule.rs"
use crate::{
    Lrc, Token
};

use super::{Lint, ExprLinter};

pub struct MyRule {
    expr: Box<dyn Expr>,
}

impl Default for MyRule {
    fn default() -> Self {
        // Define the grammatical expr the rule should look for in user text.
        let mut expr = todo!();

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ThatWhich {
    /// Pass the expr to the ExprLinter framework.
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    /// Any series of tokens that match the expr provided in the `default()` method above will
    /// be provided to this function, which you are required to map into a [`Lint`] object.
    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        unimplemented!();
    }

    fn description(&self) -> &'static str {
        "Replace this text with a description of what your rule looks for."
    }
}
```

## Test Your Changes

To test your rule, first write out an example of the error it looks for in a test file at the root of the Harper monorepo.

```markdown title="test.md"
This is an test of the `an_a` rule.
Your test should look different.
```

### Using the Command Line

From there, you can run `just lint <test filename>`.
It should emit a readable report of the grammatical errors in the document.
If the error your rule looks for does _not_ appear in this list, something is wrong.

If you need any help writing or debugging rules, don't be afraid to contact the Harper team in your draft pull request.

> **Note:** if two lints (or suggestions) overlap or address the same problem, this command will only display the first one.
> In that case, you might want to use another method of debugging.

### Using Visual Studio Code

First make sure you have [the extension installed from the marketplace](https://marketplace.visualstudio.com/items?itemName=elijah-potter.harper).
Then, configure the path of the `harper-ls` binary the Visual Studio Code extension uses in the settings page.
Set it to `<harper repo>/target/release/harper-ls`.

![How to change the `harper-ls` path](/images/vscode_harper_path.webp)

Every time you want to test a change, you'll have to recompile `harper-ls` and reload Visual Studio Code with the `Developer: Reload Window` command in the command palette.

```bash
cargo build --release # Run in the monorepo to compile `harper-ls`.
```

:::note
This workflow only works if all you're changing is the Rust code. If your changes include updates to the VS Code extension or if you'd like to test your new rule's setting in VS Code by adding it to `package.json`, then you'd need to open the extension in an [Extension Development Host](./visual-studio-code#Running-the-Extension).
:::

## Elevate Your Pull Request

Once you're satisfied with your rule, you can go ahead and elevate your pull request to mark it as "ready for review."
At that point, a maintainer on the Harper team take a look at it and (hopefully) merge it.
