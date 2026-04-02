#![no_main]

use harper_core::parsers::StrParser;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let parser = harper_html::HtmlParser::default();
    let _res = parser.parse_str(data);
});
