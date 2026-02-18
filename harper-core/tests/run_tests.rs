use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::OrgMode;
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};

/// Creates a unit test checking that the linting of a Markdown document (in
/// `tests_sources`) produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.md, $correct_expected:expr, $dialect:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_ $filename _correctly>](){
                let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(stringify!($filename), ".md")
                    )
                );

                let dict = FstDictionary::curated();
                let document = Document::new_markdown_default(&source, &dict);

                let mut linter = LintGroup::new_curated(dict, $dialect);
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

/// Creates a unit test checking that the linting of an Org mode document (in
/// `tests_sources`) produces the expected number of lints.
macro_rules! create_org_test {
    ($filename:ident.org, $correct_expected:expr, $dialect:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_ $filename _correctly>](){
                let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(stringify!($filename), ".org")
                    )
                );

                let dict = FstDictionary::curated();
                let document = Document::new(&source, &OrgMode, &dict);

                let mut linter = LintGroup::new_curated(dict, $dialect);
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

create_test!(whack_bullets.md, 1, Dialect::American);
create_test!(issue_109.md, 0, Dialect::American);
create_test!(issue_109_ext.md, 0, Dialect::American);
create_test!(chinese_lorem_ipsum.md, 2, Dialect::American);
create_test!(obsidian_links.md, 3, Dialect::American);
create_test!(issue_267.md, 0, Dialect::American);
create_test!(proper_noun_capitalization.md, 3, Dialect::American);
create_test!(amazon_hostname.md, 0, Dialect::American);
create_test!(issue_159.md, 1, Dialect::American);
create_test!(issue_358.md, 0, Dialect::American);
create_test!(issue_195.md, 0, Dialect::American);
create_test!(issue_118.md, 0, Dialect::American);
create_test!(lots_of_latin.md, 1, Dialect::American);
create_test!(pr_504.md, 1, Dialect::American);
create_test!(pr_452.md, 2, Dialect::American);
create_test!(hex_basic_clean.md, 0, Dialect::American);
create_test!(hex_basic_dirty.md, 1, Dialect::American);
create_test!(misc_closed_compound_clean.md, 0, Dialect::American);
create_test!(statist_localist.md, 0, Dialect::American);
create_test!(yogurt_british_clean.md, 0, Dialect::British);
create_test!(issue_1581.md, 0, Dialect::British);
create_test!(issue_2054.md, 6, Dialect::British);
create_test!(issue_1988.md, 0, Dialect::American);
create_test!(issue_2054_clean.md, 0, Dialect::British);
create_test!(issue_1873.md, 0, Dialect::British);
create_test!(issue_2246.md, 0, Dialect::American);
create_test!(title_case_errors.md, 2, Dialect::American);
create_test!(title_case_clean.md, 0, Dialect::American);
create_test!(issue_2233.md, 0, Dialect::American);
create_test!(issue_2240.md, 0, Dialect::American);
create_test!(allows_domain_extensions.md, 0, Dialect::American);
// It just matters that it is > 1
create_test!(issue_2151.md, 4, Dialect::British);

// Make sure it doesn't panic
create_test!(lukas_homework.md, 4, Dialect::American);

// Org mode tests
create_org_test!(index.org, 49, Dialect::American);
