use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use harper_ink::InkParser;

/// Creates a unit test checking that the linting of a Ink document (in
/// `tests_sources`) produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.ink, $correct_expected:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_ $filename _correctly>](){
                 let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(stringify!($filename), ".ink")
                    )
                 );

                 let dict = FstDictionary::curated();
                 let document = Document::new(&source, &InkParser::default(),
                      &FstDictionary::curated()
                      );

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

create_test!(good.ink, 0);
create_test!(bad.ink, 5);
