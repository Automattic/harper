use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use harper_git_commit::GitCommitParser;

/// Creates a unit test checking that the linting of a git commit document (in
/// `tests_sources`) produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.txt, $correct_expected:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_ $filename _correctly>]() {
                let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(stringify!($filename), ".txt")
                    )
                );

                let dict = FstDictionary::curated();
                let document = Document::new(source, &GitCommitParser::default(), &dict);

                let mut linter = LintGroup::new_curated(dict, Dialect::American);
                let lints = linter.lint(&document);

                dbg!(&lints);
                assert_eq!(lints.len(), $correct_expected);

                for token in document.tokens() {
                    assert!(token.span.try_get_content(document.get_source()).is_some());
                }
            }
        }
    };
}

create_test!(simple_commit.txt, 2);
create_test!(complex_verbose_commit.txt, 2);
create_test!(conventional_commit.txt, 3);
create_test!(markdown_body_commit.txt, 3);
