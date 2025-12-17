use super::LintGroup;
use crate::weir::WeirLinter;

macro_rules! generate_boilerplate {
    ([$($name:ident),+ $(,)?]) => {
        pub fn lint_group() -> LintGroup {
            let mut group = LintGroup::default();

                {
                    $(
                        group.add_chunk_expr_linter(stringify!($name), WeirLinter::new(include_str!(concat!(env!("WEIR_RULE_DIR"), "/", stringify!($name), ".weir"))).unwrap());
                    )+
                }

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
