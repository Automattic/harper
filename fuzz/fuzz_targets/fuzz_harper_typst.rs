#![no_main]

use harper_core::parsers::StrParser;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let typst = harper_typst::Typst;
    let _res = typst.parse_str(data);
});
