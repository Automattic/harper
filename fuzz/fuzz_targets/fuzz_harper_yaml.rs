#![no_main]

use harper_core::parsers::StrParser;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let parser = harper_yaml::YamlParser::default();
    let _res = parser.parse_str(data);
});
