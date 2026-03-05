use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct ObsessPreposition {
    expr: Box<dyn Expr>,
}

impl Default for ObsessPreposition {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["obsess", "obsessed", "obsesses", "obsessing"])
                    .t_ws()
                    .then_preposition(),
            ),
        }
    }
}

impl ExprLinter for ObsessPreposition {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn description(&self) -> &str {
        "Ensures valid prepositions are used with `obsess`"
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let verb_idx = 0;
        let verb_tok = toks.get(verb_idx)?;
        let verb_span = verb_tok.span;
        let verb_chars = verb_span.get_content(src);

        let prep_idx = toks.len() - 1;
        let prep_tok = toks.get(prep_idx)?;
        let prep_span = prep_tok.span;
        let prep_chars = prep_span.get_content(src);

        #[derive(PartialEq)]
        enum Conj {
            Lemma,
            Ed,
            Es,
            Ing,
        }

        let conj = if verb_chars.ends_with_ignore_ascii_case_chars(&['e', 'd']) {
            Conj::Ed
        } else if verb_chars.ends_with_ignore_ascii_case_chars(&['e', 's']) {
            Conj::Es
        } else if verb_chars.ends_with_ignore_ascii_case_chars(&['i', 'n', 'g']) {
            Conj::Ing
        } else {
            Conj::Lemma
        };

        // 👍
        // obsess* over - pay close attention to details
        // obsessed with - excessively preoccupied with
        // 👎
        // obsessed of

        if prep_chars.eq_ignore_ascii_case_str("over") {
            return None;
        }

        if conj == Conj::Ed && prep_chars.eq_ignore_ascii_case_str("with") {
            return None;
        }

        let ok_prep_vec: &[&str] = if conj == Conj::Ed {
            &["over", "with"]
        } else {
            &["over"]
        };

        let suggestions = ok_prep_vec
            .iter()
            .map(|p| Suggestion::replace_with_match_case(p.chars().collect(), prep_chars))
            .collect();

        let message = if ok_prep_vec.len() == 1 {
            format!("Use 'over' instead of '{}'.", String::from_iter(prep_chars))
        } else {
            "For `excessively preoccupied with` use `obsessed with`. For `paid close attention to details` use `obsessed over`".to_string()
        };

        Some(Lint {
            span: prep_span,
            lint_kind: LintKind::Usage,
            suggestions,
            message,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ObsessPreposition;
    use crate::linting::tests::{
        assert_lint_message, assert_suggestion_result, assert_top3_suggestion_result,
    };

    #[test]
    fn fix_obsess_on() {
        assert_suggestion_result(
            "Obsess on collecting good answers and you might be precise but irrelevant.",
            ObsessPreposition::default(),
            "Obsess over collecting good answers and you might be precise but irrelevant.",
        );
    }

    #[test]
    fn fix_obsessing_on() {
        assert_suggestion_result(
            "Obsessing on finding new solutions to old problems with AI.",
            ObsessPreposition::default(),
            "Obsessing over finding new solutions to old problems with AI.",
        );
    }

    #[test]
    fn fix_obsessing_with() {
        assert_suggestion_result(
            "I spent too long checking my code over and over, obsessing with just what might cause this",
            ObsessPreposition::default(),
            "I spent too long checking my code over and over, obsessing over just what might cause this",
        );
    }

    #[test]
    fn fix_obsess_with() {
        assert_suggestion_result(
            "And as a programmer I've been taught to obsess with that.",
            ObsessPreposition::default(),
            "And as a programmer I've been taught to obsess over that.",
        );
    }

    #[test]
    fn fix_obsesses_with() {
        assert_suggestion_result(
            "Every developer obsesses with micro-optimizations must be made to read it over and over again.",
            ObsessPreposition::default(),
            "Every developer obsesses over micro-optimizations must be made to read it over and over again.",
        );
    }

    #[test]
    fn fix_obsessed_on() {
        assert_top3_suggestion_result(
            "Secondly, if you get obsessed on any idea, then delve in it and don't worry about anything others until you get there.",
            ObsessPreposition::default(),
            "Secondly, if you get obsessed with any idea, then delve in it and don't worry about anything others until you get there.",
        );
    }

    #[test]
    fn fix_obsess_about_2743() {
        assert_lint_message(
            "but don't obsess about it",
            ObsessPreposition::default(),
            "Use 'over' instead of 'about'.",
        );
    }
}
