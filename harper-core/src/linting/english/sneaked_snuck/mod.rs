use super::merge_linters::merge_linters;
use crate::{
    CharStringExt, Lint, Token,
    expr::Expr,
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::Word,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Prefer {
    Snuck,
    Sneaked,
}

pub struct PreferSnuck {
    expr: Word,
}
pub struct PreferSneaked {
    expr: Word,
}

fn build_expr(flag: Prefer) -> Word {
    match flag {
        Prefer::Snuck => Word::new("sneaked"),
        Prefer::Sneaked => Word::new("snuck"),
    }
}

const SNEAKED: &str = "sneaked";
const SNUCK: &str = "snuck";

fn to_lint(toks: &[Token], src: &[char], pref: Prefer) -> Option<Lint> {
    let tokspan = toks.first()?.span;
    let word = tokspan.get_content(src);

    let (target_word, source_word) = match pref {
        Prefer::Snuck => {
            if word.eq_ch(&['s', 'n', 'e', 'a', 'k', 'e', 'd']) {
                (SNUCK, SNEAKED)
            } else {
                return None;
            }
        }
        Prefer::Sneaked => {
            if word.eq_ch(&['s', 'n', 'u', 'c', 'k']) {
                (SNEAKED, SNUCK)
            } else {
                return None;
            }
        }
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

impl_expr_linter!(PreferSnuck, Prefer::Snuck, "Prefer `snuck` over `sneaked`.");

impl_expr_linter!(
    PreferSneaked,
    Prefer::Sneaked,
    "Prefer `sneaked` over `snuck`."
);

merge_linters! {
    SneakedSnuck =>
        PreferSneaked,
        PreferSnuck
        => "Enforces `sneaked` v `snuck` preferences."
}

#[cfg(test)]
mod tests {
    use super::{PreferSneaked, PreferSnuck};
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    // Prefer "snuck"

    #[test]
    fn correct_sneaked_to_snuck() {
        assert_suggestion_result(
            "He sneaked in around the back.",
            PreferSnuck::default(),
            "He snuck in around the back.",
        );
    }

    #[test]
    fn correct_sneaked_uppercase() {
        assert_suggestion_result(
            "He Sneaked in around the back.",
            PreferSnuck::default(),
            "He Snuck in around the back.",
        );
    }

    #[test]
    fn correct_sneaked_allcaps() {
        assert_suggestion_result(
            "He SNEAKED in around the back.",
            PreferSnuck::default(),
            "He SNUCK in around the back.",
        );
    }

    #[test]
    fn correct_to_snuck_from_github() {
        assert_suggestion_result(
            "recycled transitions, lingering inflation, copula swaps that sneaked through",
            PreferSnuck::default(),
            "recycled transitions, lingering inflation, copula swaps that snuck through",
        );
    }

    #[test]
    fn dont_flag_snuck_when_it_is_preferred() {
        assert_no_lints(
            "I'm not sure exactly when this fix snuck in",
            PreferSnuck::default(),
        );
    }

    // Prefer "sneaked"

    #[test]
    fn correct_snuck_to_sneaked() {
        assert_suggestion_result(
            "He snuck in around the back.",
            PreferSneaked::default(),
            "He sneaked in around the back.",
        );
    }

    #[test]
    fn correct_snuck_uppercase() {
        assert_suggestion_result(
            "He Snuck in around the back.",
            PreferSneaked::default(),
            "He Sneaked in around the back.",
        );
    }

    #[test]
    fn correct_snuck_allcaps() {
        assert_suggestion_result(
            "He SNUCK in around the back.",
            PreferSneaked::default(),
            "He SNEAKED in around the back.",
        );
    }

    #[test]
    fn correct_to_sneaked_from_github() {
        assert_suggestion_result(
            "A few unhandled Errors snuck their way into the code base over time f.ex. here:",
            PreferSneaked::default(),
            "A few unhandled Errors sneaked their way into the code base over time f.ex. here:",
        );
    }

    #[test]
    fn dont_flag_sneaked_when_it_is_preferred() {
        assert_no_lints(
            "Something related to recent experiments of WASM support sneaked into the main branch.",
            PreferSneaked::default(),
        );
    }
}
