# cargo-fuzz targets

## Setup

Follow the rust-fuzz [setup guide](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html).
You need a nightly toolchain and the cargo-fuzz plugin.

Simple installation steps:

- `rustup install nightly`
- `cargo install cargo-fuzz`

## Adding a new fuzzing target

To add a new target, run `cargo fuzz add $TARGET_NAME`

## Doing a fuzzing run

If possible, prefill the `fuzz/corpus/$TARGET_NAME` directory with appropriate examples to speed up fuzzing.
The fuzzer should be coverage aware, so providing a well formed input document to fuzzing targets only expecting a string as input can speed things up a lot.

Then, run `cargo +nightly fuzz run $TARGET_NAME -- -timeout=$TIMEOUT`

The timeout flag accepts a timeout in seconds, after which a long-running test case will be aborted.
This should be set to a low number to quickly report endless loops / deep recursion in parsers.

The normal fuzzing run will continue until a crash is found.

Alternatively, if you want to run all the fuzzing targets at once: `cargo +nightly fuzz list | parallel -j0 cargo +nightly fuzz run {} -- -timeout=$TIMEOUT`

## Minifying a test case

Once the fuzzer finds a crash, we probably want to minify the result.
This can be done with `CARGO_PROFILE_RELEASE_LTO=false cargo +nightly fuzz tmin $TARGET $TEST_CASE_PATH`
