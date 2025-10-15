use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::MarkdownOptions;
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use harper_jjdescription::JJDescriptionParser;

/// Creates a unit test checking that the linting of a git commit document (in
/// `tests_sources`) produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.txt, $correct_expected:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_ $filename _correctly>](){
                 let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(stringify!($filename), ".txt")
                    )
                 );

                 let dict = FstDictionary::curated();
                 let document = Document::new(source, &JJDescriptionParser::new(MarkdownOptions::default()), &dict);

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

create_test!(simple_description.txt, 1);
create_test!(complex_verbose_description.txt, 2);
create_test!(conventional_description.txt, 3);
