---
title: Spans
---

When you lint a document using `harper.js`, you'll get back a series of `Lint` objects, each with a `span` method available.
There are a number of questions that come up about this method.

## What is a span?

A span is a struct that contains a start index and an end index.
The Rust code looks something like this:

```rust
struct Span {
    start: usize,
    end: usize
}
```

For the uninitiated, a `usize` is an unsigned integer.
Most commonly (and always in the context of a `Lint`), spans are referring to character windows.
More precisely, spans are windows or slices into an array of [unicode scalar values](https://www.unicode.org/glossary/#unicode_scalar_value).
This is less relevant to JavaScript consumers.
Some get confused and believe these are indices into byte arrays (C-style strings).

##  Why do I need to obtain the span through a method call?

The actual span for a `Lint` is stored inside WebAssembly memory.
In order to access that data from JavaScript, a tiny bit of WebAssembly code must be run to serialize it and convert its indices to JavaScript [number types](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number).

## What can I use it for?

In a `Lint`, the span represents two things:

1. The location of the problematic text.
1. The text that would be edited by the relevant suggestions.

In other words, you use the span to underline the problem, then again to solve it.
