---
title: Author a Rule
---

[Harper's grammatical rules are many](../rules), but most are relatively easy to understand.
Before we get into how to write a rule, it is important that we get some of the language cleared up.

When we refer to a Harper rule, we are talking about [an implementation of the Linter trait](https://docs.rs/harper-core/latest/harper_core/linting/trait.Linter.html).
As you can see, there is an enormous amount of flexibility in this trait and a wide variety of potential strategies for querying the provided document to locate errors.

This guide will go through one easy way to add a rule to Harper.
The lofty goal is for this to be doable by someone with little to no Rust experience.
You should, however, be able to figure out how to use Git.

## Fork the Harper Monorepo

Before you can open a pull request or modify any code, you need a mutable copy of our monorepo.
The best way to do that is to [fork it in GitHub](https://github.com/Automattic/harper/fork).

Next, you'll want to copy this fork onto your computer and create a new branch.
GitHub has an [excellent page explaining how clone repositories](https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository).

## Get Your Environment Set Up

Please read our [guide for getting your environment set up](./environment).

## Open a Draft Pull Request

Next, you'll want to open a draft pull request.
This gives us (the Harper maintainers) a better view of what is actively being worked on.
It also makes it much easier to ask questions about how Harper works while you're working on your rule.

GitHub has some [good documentation on how to create a draft PR](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request-from-a-fork) if this is your first time.

## Create Your Rule's Module

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

Finally, enable it in the macro invocation near the bottom:

```rust title="harper-core/src/linting/lint_group.rs"
create_lint_group_config!(
    SpelledNumbers => false,
    AnA => true,
    SentenceCapitalization => true,
    UnclosedQuotes => true,
// [svp! df:+]    MyRule => true
);
```

That's it!

## Write Your Rule

Defining a pattern and [implementing the PatternLinter trait](https://docs.rs/harper-core/latest/harper_core/linting/trait.PatternLinter.html) is the easiest way to define a new rule for Harper.
Here's a template to get you started:

```rust title="my_rule.rs"
use crate::{
    Lrc, Token
};

use super::{Lint, PatternLinter};

pub struct MyRule {
    pattern: Box<dyn Pattern>,
}

impl Default for MyRule {
    fn default() -> Self {
        let mut pattern = todo!();

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for ThatWhich {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    /// Any series of tokens that match the pattern provided in the `default()` method above will
    /// be provided to this function, which you are required to map into a [`Lint`] object.
    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint {
        unimplemented!();
    }

    fn description(&self) -> &'static str {
        "Replace this text with a description of what your rule looks for."
    }
}
```

## Test Your Changes

To test your rule, write out an example of the error it looks for in a test file at the root of the Harper monorepo.

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
Then, can configure the path of the `harper-ls` binary the Visual Studio Code extension uses in settings.
Set it to `<harper repo>/target/release/harper-ls`.

![How to change the `harper-ls` path](/images/vscode_harper_path.webp)

Now every time you want to test a change, you'll have to recompile `harper-ls` and reload Visual Studio Code using `Developer: Reload Window`.

```bash
cargo build --release # Run in the monorepo to compile `harper-ls`.
```

## Elevate Your Pull Request

Once you're satisfied with your rule, you can go ahead and elevate your pull requests to mark it as "ready for review."
At that point, a maintainer on the Harper team take a look at it and (hopefully) merge it.
