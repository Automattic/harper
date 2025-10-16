#![no_main]

// use harper_core::parsers::StrParser;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|_data: &str| {
    // TODO: figure out how to create a literate haskell parser
    // let _res = typst.parse_str(&data);
});
