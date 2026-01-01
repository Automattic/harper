use harper_asciidoc::AsciidocParser;
use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};

/// Creates a unit test checking Asciidoc source code parsing.
macro_rules! create_test {
    ($filename:ident.$ext:ident, $correct_expected:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_$ext _ $filename _correctly>](){
                 let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(
                        stringify!($filename), ".", stringify!($ext))
                    )
                 );

                 let parser = AsciidocParser::default();
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

create_test!(basic.adoc, 2);
create_test!(table.adoc, 1);
create_test!(comment.adoc, 2);
