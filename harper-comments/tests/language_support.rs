use std::path::Path;

use harper_comments::CommentParser;
use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::MarkdownOptions;
use harper_core::{Dialect, Document, FstDictionary};

/// Creates a unit test checking that the linting of a source file in
/// `language_support_sources` produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.$ext:ident, $correct_expected:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_$ext _ $filename _correctly>](){
                 let filename = concat!(stringify!($filename), ".", stringify!($ext));
                 let source = include_str!(
                    concat!(
                        "./language_support_sources/",
                        concat!(
                        stringify!($filename), ".", stringify!($ext))
                    )
                 );

                 let parser = CommentParser::new_from_filename(Path::new(filename), MarkdownOptions::default()).unwrap();
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

create_test!(multiline_comments.cpp, 3);
create_test!(multiline_comments.ts, 3);
create_test!(multiline_comments.sol, 3);
create_test!(clean.rs, 0);
create_test!(clean.sol, 0);
create_test!(jsdoc.ts, 4);
create_test!(issue_96.lua, 0);
create_test!(merged_lines.ts, 1);
create_test!(javadoc_clean_simple.java, 0);
create_test!(javadoc_complex.java, 5);
create_test!(issue_132.rs, 1);
create_test!(laravel_app.php, 2);
create_test!(ignore_shebang_1.sh, 0);
create_test!(ignore_shebang_2.sh, 0);
create_test!(ignore_shebang_3.sh, 0);
create_test!(ignore_shebang_4.sh, 1);
create_test!(common.mill, 1);
create_test!(basic_kotlin.kt, 0);
create_test!(basic.clj, 12);

// Checks that some comments are masked out
create_test!(ignore_comments.rs, 1);
create_test!(ignore_comments.c, 1);
create_test!(ignore_comments.sol, 1);

// These are to make sure nothing crashes.
create_test!(empty.js, 0);
create_test!(issue_229.js, 0);
create_test!(issue_229.c, 0);
create_test!(issue_229.cs, 0);
create_test!(eof.rs, 0);
