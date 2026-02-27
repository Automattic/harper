# `harper-core`

`harper-core` is the fundamental engine behind [Harper](https://writewithharper.com), the private grammar checker.

`harper-core` is [available on `crates.io`](https://crates.io/crates/harper-core) to enable Rust engineers to integrate high-quality grammar checking directly into their apps and workflows.
Feel free to use `harper-core` in your projects.
If you run into problems with the code, open an issue or, even better, create a pull request.
We are also happy to chat with you on [Discord](https://discord.com/invite/JBqcAaKrzQ).

[The documentation for `harper-core` is available online.](https://docs.rs/harper-core/latest/harper_core/)

If you would prefer to run Harper from inside a JavaScript runtime, [we have a package for that as well.](https://www.npmjs.com/package/harper.js)

## Example

Here's what a full end-to-end linting pipeline could look like using `harper-core`.

```rust
use std::sync::Arc;

use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::PlainEnglish;
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};

let text = "This is an test.";
let parser = PlainEnglish;

let document = Document::new_curated(text, &parser);

let dict = Arc::new(FstDictionary::curated());
let mut linter = LintGroup::new_curated(dict, Dialect::American);

let lints = linter.lint(&document);

for lint in lints {
    println!("{:?}", lint);
}
```

## Features

`concurrent`: Whether to use thread-safe primitives (`Arc` vs `Rc`). Disabled by default.
It is not recommended unless you need thread-safely (i.e. you want to use something like `tokio`).

## Other Relevant Packages

- [`harper-ls`](https://crates.io/crates/harper-ls)
- [`harper-tree-sitter`](https://crates.io/crates/harper-tree-sitter)
