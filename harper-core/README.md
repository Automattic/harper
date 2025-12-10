# `harper-core`

`harper-core` is the fundamental engine behind [Harper](https://writewithharper.com), the grammar checker for developers.

`harper-core` _is_ [available on `crates.io`](https://crates.io/crates/harper-core). However, improving the API is not currently a high priority.
Feel free to use `harper-core` in your projects.
If you run into issues, create a pull request.

## Example

Here's what a full end-to-end linting pipeline could look like using `harper-core`.

```rust
use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::PlainEnglish;
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};

let text = "This is an test.";
let parser = PlainEnglish;

let document = Document::new_curated(text, &parser);

let dict = FstDictionary::curated();
let mut linter = LintGroup::new_curated(dict, Dialect::American);

let lints = linter.lint(&document);

for lint in lints {
    println!("{:?}", lint);
}
```

## Features

`concurrent`: Whether to use thread-safe primitives (`Arc` vs `Rc`). Disabled by default.
It is not recommended unless you need thread-safely (i.e. you want to use something like `tokio`).
