---
title: About Harper's Test Suite
---

Harper's goal is to deliver top-tier grammar checking fast, without compromising privacy.
How do we maintain quality while also iterating quickly on our core engine?

As you know, Harper's core engine is written in Rust.
As a corollary to that, we use Cargo to pull dependencies, build, and test the system.
While we do take advantage of snapshot and integration tests, we tend to focus our efforts on unit tests.

## Performance

In the interest of maintaining fast iteration cycles, we run our tests with `opt-level = 1`.
These optimizations are known to cause issues with debuggers. 
If you plan to use one, you may want to comment them out.

@code(../../../../../../../Cargo.toml)

## Other Reading

- [3 Traits of Good Test Suites](https://elijahpotter.dev/articles/3_traits_of_good_test_suites)
