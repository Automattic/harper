use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use harper_yaml::YamlParser;

/// Creates a unit test checking that the linting of a YAML source file (in
/// `test_sources`) produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.yaml, $correct_expected:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_ $filename _correctly>](){
                 let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(stringify!($filename), ".yaml")
                    )
                 );

                 let parser = YamlParser::default();
                 let dict = FstDictionary::curated();
                 let document = Document::new(&source, &parser, &dict);

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

create_test!(clean.yaml, 0);
create_test!(dirty.yaml, 1);
create_test!(filtered_values.yaml, 0);
