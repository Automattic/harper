use harper_core::linting::{LintGroup, LintGroupConfig, Linter};
use harper_core::{
    parsers::{Markdown, Typst},
    Document, FstDictionary,
};

/// Creates a unit test checking that the linting of a document in
/// `tests_sources` produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.$ext:ident, $parser:expr, $correct_expected:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_ $filename _correctly>](){
                 let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(stringify!($filename), ".", stringify!($ext))
                    )
                 );

                 let dict = FstDictionary::curated();
                 let document = Document::new(&source, $parser, &dict);

                 let mut linter = LintGroup::new(
                     LintGroupConfig::default(),
                     dict
                 );
                 let lints = linter.lint(&document);

                 dbg!(&lints);
                 assert_eq!(lints.len(), $correct_expected);

                 // Make sure that all generated tokens span real characters
                 for token in document.tokens(){
                     assert!(token.span.try_get_content(document.get_source()).is_some());
                 }
            }
        }
    };
}

create_test!(whack_bullets.md, &Markdown, 1);
create_test!(preexisting.md, &Markdown, 0);
create_test!(issue_109.md, &Markdown, 0);
create_test!(issue_109_ext.md, &Markdown, 0);
create_test!(chinese_lorem_ipsum.md, &Markdown, 2);
create_test!(obsidian_links.md, &Markdown, 2);
create_test!(issue_267.md, &Markdown, 0);
create_test!(proper_noun_capitalization.md, &Markdown, 2);

#[cfg(feature = "typst")]
create_test!(complex_typst.typ, &Typst, 0);
#[cfg(feature = "typst")]
create_test!(typst_spelling_mistakes.typ, &Typst, 4);
