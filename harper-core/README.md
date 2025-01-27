# `harper-core`

`harper-core` is the fundamental engine behind [Harper](https://writewithharper.com), the grammar checker for developers.

`harper-core` _is_ [available on `crates.io`](https://crates.io/crates/harper-core). However, improving the API is not currently a high priority.
Feel free to use `harper-core` in your projects.
If you run into issues, create a pull request.

## Features

`concurrent`: Whether to use thread-safe primitives (`Arc` vs `Rc`). Disabled by default.
It is not recommended unless you need thread-safely (i.e. you want to use something like `tokio`).
