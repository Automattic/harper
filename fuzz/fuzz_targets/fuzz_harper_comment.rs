#![no_main]

use harper_core::parsers::{MarkdownOptions, StrParser};
use libfuzzer_sys::arbitrary::{Arbitrary, Result, Unstructured};
use libfuzzer_sys::fuzz_target;

#[derive(Debug)]
struct Language(String);

const LANGUAGES: [&str; 34] = [
    "cmake",
    "cpp",
    "csharp",
    "c",
    "dart",
    "go",
    "haskell",
    "javascriptreact",
    "javascript",
    "java",
    "kotlin",
    "lua",
    "nix",
    "php",
    "powershell",
    "python",
    "ruby",
    "rust",
    "scala",
    "shellscript",
    "solidity",
    "swift",
    "toml",
    "typescriptreact",
    "typescript",
    "clojure",
    "go",
    "lua",
    "java",
    "javascriptreact",
    "typescript",
    "typescriptreact",
    "solidity",
    "zig",
];

impl<'a> Arbitrary<'a> for Language {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let &lang = u.choose(&LANGUAGES)?;
        Ok(Language(lang.to_owned()))
    }
}

#[derive(Debug)]
struct Input {
    language: Language,
    text: String,
}

impl<'a> Arbitrary<'a> for Input {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let (language, text) = Arbitrary::arbitrary(u)?;
        Ok(Input { language, text })
    }

    fn arbitrary_take_rest(u: Unstructured<'a>) -> Result<Self> {
        let (language, text) = Arbitrary::arbitrary_take_rest(u)?;
        Ok(Input { language, text })
    }
}

fuzz_target!(|data: Input| {
    let opts = MarkdownOptions::default();
    let parser = harper_comments::CommentParser::new_from_language_id(&data.language.0, opts);
    if let Some(parser) = parser {
        let _res = parser.parse_str(&data.text);
    }
});
