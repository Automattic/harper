use super::LintGroup;
use crate::weir::{WeirLinter, WeirScope};

macro_rules! generate_boilerplate {
    ([$($name:ident),+ $(,)?]) => {
        pub fn lint_group() -> LintGroup {
            let mut group = LintGroup::default();

                {
                    $(
                        let linter = WeirLinter::new(include_str!(concat!(env!("WEIR_RULE_DIR"), "/", stringify!($name), ".weir"))).unwrap();
                        match linter.scope() {
                            WeirScope::Chunk => group.add_chunk_expr_linter(stringify!($name), linter.into_chunk_linter().unwrap_or_else(|_| unreachable!())),
                            WeirScope::Sentence => group.add_sentence_expr_linter(stringify!($name), linter.into_sentence_linter().unwrap_or_else(|_| unreachable!())),
                        };
                    )+
                }

            group.set_all_rules_to(Some(true));

            group
        }

        #[cfg(test)]
        mod tests {
            use paste::paste;
            use crate::weir::tests::assert_passes_all;
            use crate::weir::WeirLinter;

            $(
                paste! {
                    #[test]
                    fn [<run_tests_for_ $name:snake>](){
                        let mut linter = WeirLinter::new(include_str!(concat!(env!("WEIR_RULE_DIR"), "/", stringify!($name), ".weir"))).unwrap();
                        assert_passes_all(&mut linter);
                    }
                }
            )+
        }
    };
}

include!(env!("WEIR_RULE_LIST"));
