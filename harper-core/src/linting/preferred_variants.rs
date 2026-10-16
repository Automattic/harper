use crate::linting::LintGroup;

pub fn lint_group() -> LintGroup {
    let mut group = LintGroup::empty();

    macro_rules! define_preferred_variants {
        (
            $group:ident,
            $(
                ($word_one:ident, $word_two:ident)
            ),+
        ) => {
            paste::paste! {
                use crate::{
                    CharStringExt, Lint, Token,
                    expr::Expr,
                    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
                    patterns::Word,
                };

                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                enum Prefer {
                    $(
                        [<$word_one:camel>],
                        [<$word_two:camel>],
                    )*
                }

                fn build_expr(flag: Prefer) -> Word {
                    match flag {
                        $(
                            Prefer::[<$word_one:camel>] => Word::new(stringify!($word_two)),
                            Prefer::[<$word_two:camel>] => Word::new(stringify!($word_one)),
                        )*
                    }
                }

                fn to_lint(toks: &[Token], src: &[char], pref: Prefer) -> Option<Lint> {
                    let tokspan = toks.first()?.span;
                    let word = tokspan.get_content(src);

                    let (target_word, source_word) = match pref {
                        $(
                            Prefer::[<$word_one:camel>] => {
                                if word.eq_str(stringify!($word_two)) {
                                    (stringify!($word_one), stringify!($word_two))
                                } else {
                                    return None;
                                }
                            }
                            Prefer::[<$word_two:camel>] => {
                                if word.eq_str(stringify!($word_one)) {
                                    (stringify!($word_two), stringify!($word_one))
                                } else {
                                    return None;
                                }
                            }
                        )*
                    };

                    Some(Lint {
                        span: tokspan,
                        lint_kind: LintKind::Usage,
                        suggestions: vec![Suggestion::replace_with_match_case_str(target_word, word)],
                        message: format!("Use `{}` instead of `{}`.", target_word, source_word),
                        ..Default::default()
                    })
                }

                macro_rules! impl_expr_linter {
                    ($name:ident, $pref:expr, $desc:expr) => {
                        impl Default for $name {
                            fn default() -> Self {
                                Self {
                                    expr: build_expr($pref),
                                }
                            }
                        }

                        impl ExprLinter for $name {
                            type Unit = Chunk;

                            fn description(&self) -> &str {
                                $desc
                            }

                            fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
                                to_lint(toks, src, $pref)
                            }

                            fn expr(&self) -> &dyn Expr {
                                &self.expr
                            }
                        }
                    };
                }

                $(
                    pub struct [<Prefer $word_one:camel>] {
                        expr: Word,
                    }
                    pub struct [<Prefer $word_two:camel>] {
                        expr: Word,
                    }

                    impl_expr_linter!(
                        [<Prefer $word_one:camel>],
                        Prefer::[<$word_one:camel>],
                        concat!("Prefer `", stringify!($word_one), "` over `", stringify!($word_two), "`.")
                    );

                    impl_expr_linter!(
                        [<Prefer $word_two:camel>],
                        Prefer::[<$word_two:camel>],
                        concat!("Prefer `", stringify!($word_two), "` over `", stringify!($word_one), "`.")
                    );

                    $group.add(stringify!([<Prefer $word_one:camel>]), [<Prefer $word_one:camel>]::default());
                    $group.config.set_rule_enabled(stringify!([<Prefer $word_one:camel>]), false);
                    $group.add(stringify!([<Prefer $word_two:camel>]), [<Prefer $word_two:camel>]::default());
                    $group.config.set_rule_enabled(stringify!([<Prefer $word_two:camel>]), true);
                )*
            }
        }
    }

    define_preferred_variants!(
        group,
        (pleaded, pled),
        (sneaked, snuck),
        (gradable, gradeable)
    );

    group
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    use super::lint_group;

    // Prefer "snuck"

    #[test]
    fn correct_sneaked_to_snuck() {
        assert_suggestion_result(
            "He sneaked in around the back.",
            lint_group(),
            "He snuck in around the back.",
        );
    }

    #[test]
    fn correct_sneaked_uppercase() {
        assert_suggestion_result(
            "He Sneaked in around the back.",
            lint_group(),
            "He Snuck in around the back.",
        );
    }

    #[test]
    fn correct_sneaked_allcaps() {
        assert_suggestion_result(
            "He SNEAKED in around the back.",
            lint_group(),
            "He SNUCK in around the back.",
        );
    }

    #[test]
    fn correct_to_snuck_from_github() {
        assert_suggestion_result(
            "recycled transitions, lingering inflation, copula swaps that sneaked through",
            lint_group(),
            "recycled transitions, lingering inflation, copula swaps that snuck through",
        );
    }

    #[test]
    fn dont_flag_snuck_when_it_is_preferred() {
        let mut lg = lint_group();
        lg.config.set_rule_pair("PreferSnuck", "PreferSneaked");
        assert_no_lints("I'm not sure exactly when this fix snuck in", lg);
    }

    // Prefer "sneaked"

    #[test]
    fn correct_snuck_to_sneaked() {
        let mut lg = lint_group();
        lg.config.set_rule_pair("PreferSneaked", "PreferSnuck");
        assert_suggestion_result(
            "He snuck in around the back.",
            lg,
            "He sneaked in around the back.",
        );
    }

    #[test]
    fn correct_snuck_uppercase() {
        let mut lg = lint_group();
        lg.config.set_rule_pair("PreferSneaked", "PreferSnuck");
        assert_suggestion_result(
            "He Snuck in around the back.",
            lg,
            "He Sneaked in around the back.",
        );
    }

    #[test]
    fn correct_snuck_allcaps() {
        let mut lg = lint_group();
        lg.config.set_rule_pair("PreferSneaked", "PreferSnuck");
        assert_suggestion_result(
            "He SNUCK in around the back.",
            lg,
            "He SNEAKED in around the back.",
        );
    }

    #[test]
    fn correct_to_sneaked_from_github() {
        let mut lg = lint_group();
        lg.config.set_rule_pair("PreferSneaked", "PreferSnuck");
        assert_suggestion_result(
            "A few unhandled Errors snuck their way into the code base over time f.ex. here:",
            lg,
            "A few unhandled Errors sneaked their way into the code base over time f.ex. here:",
        );
    }

    #[test]
    fn dont_flag_sneaked_when_it_is_preferred() {
        let mut lg = lint_group();
        lg.config.set_rule_pair("PreferSneaked", "PreferSnuck");
        assert_no_lints(
            "Something related to recent experiments of WASM support sneaked into the main branch.",
            lg,
        );
    }

    // OneOfMany / Mutual exclusivity

    #[test]
    fn by_default_one_must_be_enabled_and_one_must_be_disabled_snuck() {
        use crate::Dialect;
        use crate::spell::FstDictionary;

        assert_lint_count(
            "I say 'snuck' but you say 'sneaked'.",
            crate::linting::LintGroup::new_curated(FstDictionary::curated(), Dialect::American),
            1,
        );
    }

    // Prefer "pled"

    #[test]
    fn correct_pleaded_to_pled() {
        assert_suggestion_result(
            "Reality Winner pleaded guilty in June to a single count of transmitting national security information.",
            lint_group(),
            "Reality Winner pled guilty in June to a single count of transmitting national security information.",
        );
    }

    // Prefer "pleaded"

    #[test]
    fn correct_pled_to_pleaded() {
        let mut lg = lint_group();
        lg.config.set_rule_pair("PreferPleaded", "PreferPled");
        assert_suggestion_result(
            "2005 Samsung pled guilty in connection with the cartel and paid a fine.",
            lg,
            "2005 Samsung pleaded guilty in connection with the cartel and paid a fine.",
        );
    }

    // OneOfMany / Mutual exclusivity

    #[test]
    fn by_default_one_must_be_enabled_and_one_must_be_disabled_pled() {
        use crate::Dialect;
        use crate::spell::FstDictionary;

        let mut lg =
            crate::linting::LintGroup::new_curated(FstDictionary::curated(), Dialect::American);
        lg.config.unset_rule_enabled("SpellCheck");

        assert_lint_count("I say 'pled' but you say 'pleaded'.", lg, 1);
    }

    // Others

    #[test]
    fn gradeable_to_gradable() {
        let mut lg = lint_group();
        lg.config.set_rule_pair("PreferGradable", "PreferGradeable");
        assert_suggestion_result(
            "Gradeable status should be part of the Gradeable model",
            lg,
            "Gradable status should be part of the Gradable model",
        );
    }

    #[test]
    fn gradable_to_gradeable() {
        assert_suggestion_result(
            "A research project on the structure of gradable adjectives based on corpus analysis",
            lint_group(),
            "A research project on the structure of gradeable adjectives based on corpus analysis",
        );
    }
}
