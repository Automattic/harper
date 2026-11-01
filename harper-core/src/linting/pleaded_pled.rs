use crate::{
    CharStringExt, Lint, Token,
    expr::Expr,
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::Word,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Prefer {
    Pled,
    Pleaded,
}

pub struct PreferPled {
    expr: Word,
}
pub struct PreferPleaded {
    expr: Word,
}

fn build_expr(flag: Prefer) -> Word {
    match flag {
        Prefer::Pled => Word::new("pleaded"),
        Prefer::Pleaded => Word::new("pled"),
    }
}

const PLEADED: &str = "pleaded";
const PLED: &str = "pled";

fn to_lint(toks: &[Token], src: &[char], pref: Prefer) -> Option<Lint> {
    let tokspan = toks.first()?.span;
    let word = tokspan.get_content(src);

    let (target_word, source_word) = match pref {
        Prefer::Pled => {
            if word.eq_ch(&['p', 'l', 'e', 'a', 'd', 'e', 'd']) {
                (PLED, PLEADED)
            } else {
                return None;
            }
        }
        Prefer::Pleaded => {
            if word.eq_ch(&['p', 'l', 'e', 'd']) {
                (PLEADED, PLED)
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

impl_expr_linter!(PreferPled, Prefer::Pled, "Prefer `pled` over `pleaded`.");

impl_expr_linter!(
    PreferPleaded,
    Prefer::Pleaded,
    "Prefer `pleaded` over `pled`."
);

#[cfg(test)]
mod tests {
    use super::{PreferPleaded, PreferPled};
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    // Prefer "pled"

    #[test]
    fn correct_pleaded_to_pled() {
        assert_suggestion_result(
            "Reality Winner pleaded guilty in June to a single count of transmitting national security information.",
            PreferPled::default(),
            "Reality Winner pled guilty in June to a single count of transmitting national security information.",
        );
    }

    // Prefer "pleaded"

    #[test]
    fn correct_pled_to_pleaded() {
        assert_suggestion_result(
            "2005 Samsung pled guilty in connection with the cartel and paid a fine.",
            PreferPleaded::default(),
            "2005 Samsung pleaded guilty in connection with the cartel and paid a fine.",
        );
    }

    // OneOfMany / Mutual exclusivity

    #[test]
    fn by_default_one_must_be_enabled_and_one_must_be_disabled() {
        use crate::Dialect;
        use crate::spell::FstDictionary;

        let mut lg =
            crate::linting::LintGroup::new_curated(FstDictionary::curated(), Dialect::American);
        lg.config.unset_rule_enabled("SpellCheck");

        assert_lint_count("I say 'pled' but you say 'pleaded'.", lg, 1);
    }
}
