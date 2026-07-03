use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::MarkdownOptions;
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use harper_literate_haskell::LiterateHaskellParser;

/// Creates a unit test checking that the linting of a Markdown document (in
/// `tests_sources`) produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.lhs, $correct_expected:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_ $filename _correctly>](){
                 let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(stringify!($filename), ".lhs")
                    )
                 );

                 let dict = FstDictionary::curated();
                 let document = Document::new_curated(&source, &LiterateHaskellParser::new_markdown(MarkdownOptions::default()));

                 let mut linter = LintGroup::new_curated(dict, Dialect::American);
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

create_test!(bird_format.lhs, 2);
create_test!(latex_format.lhs, 2);
create_test!(mixed_format.lhs, 4);
