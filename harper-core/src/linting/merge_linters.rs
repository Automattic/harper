macro_rules! merge_linters {
    ($name:ident => $($linter:ident),* => $desc:expr) => {
        pub use merge_rule_hidden::$name;

        mod merge_rule_hidden {
            use paste::paste;
            use crate::{Document, linting::{Lint, Linter}, remove_overlaps};

            $(
                use super::$linter;
            )*

            paste! {
                #[derive(Default)]
                pub struct $name {
                    $(
                        [< $linter:snake >]: $linter,
                    )*
                }

                impl Linter for $name {
                    fn lint(&mut self, document: &Document) -> Vec<Lint>{
                        let mut lints = Vec::new();

                        $(
                            lints.extend(self.[< $linter:snake >].lint(document));
                        )*

                        remove_overlaps(&mut lints);

                        lints
                    }

                    fn description(&self) -> &'static str {
                        $desc
                    }
                }
            }
        }
    };
}

pub(crate) use merge_linters;
