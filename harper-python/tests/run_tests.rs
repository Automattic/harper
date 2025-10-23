use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use harper_python::PythonParser;

/// Creates a unit test checking Python source code parsing.
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

                 let parser = PythonParser::default();
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

create_test!(docstrings.py, 4);
create_test!(field_docstrings.py, 2);
create_test!(comments.py, 1);
