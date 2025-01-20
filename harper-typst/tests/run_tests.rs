use harper_core::linting::{LintGroup, LintGroupConfig, Linter};
use harper_core::{Document, FstDictionary};
use harper_typst::Typst;

/// Creates a unit test checking that the linting of a document in
/// `tests_sources` produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.$ext:ident, $correct_expected:expr) => {
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
                 let document = Document::new(&source, &Typst, &dict);

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

create_test!(complex_document.typ, 0);
create_test!(simplified_document.typ, 0);
create_test!(complex_document_with_spelling_mistakes.typ, 4);
