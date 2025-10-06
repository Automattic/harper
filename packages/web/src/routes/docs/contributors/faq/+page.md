---
title: Frequently Asked Questions
---

This page will be composed of frequently asked questions for contributors.

## What's the Difference Between a `Linter` and a `PatternLinter`?

![A diagram that shows the relationship between a `Linter` and a `PatternLinter`](/images/linter_diagram.png)

A [`Linter`](https://docs.rs/harper-core/latest/harper_core/linting/trait.Linter.html) is a Rust trait for a type that queries a [`Document`](https://docs.rs/harper-core/latest/harper_core/struct.Document.html) to identify grammatical errors, returning a human-readable list of errors, optionally with suggestions that could resolve them.

A [`PatternLinter`](https://docs.rs/harper-core/latest/harper_core/linting/trait.PatternLinter.html) is another Rust trait for a type that can hook into the `PatternLinter` framework.
The `PatternLinter` provides a pattern, which the framework locates inside of documents.
When a sequence of tokens is matched, they are provided to `match_to_lint` to map the match to a human-readable error.

All types that implement `PatternLinter` also implement `Linter` thanks to a [blanket implementation](https://doc.rust-lang.org/reference/glossary.html?highlight=blanket#blanket-implementation) of the latter upon the former.
