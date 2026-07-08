use crate::{
    Lint, Lrc, Token, TokenKind, TokenStringExt,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

enum RedundancyLevel {
    Probable,
    Possible,
}

const VERBS: &[(&str, RedundancyLevel)] = &[
    ("coined", RedundancyLevel::Probable),
    ("discovered", RedundancyLevel::Probable),
    ("introduced", RedundancyLevel::Possible),
    ("invented", RedundancyLevel::Probable),
    ("originated", RedundancyLevel::Probable),
    ("released", RedundancyLevel::Possible),
];

pub struct RedundantFirsts {
    expr: SequenceExpr,
}

impl Default for RedundantFirsts {
    fn default() -> Self {
        let verbs = Lrc::new(WordSet::new(
            &VERBS.iter().map(|(v, _)| *v).collect::<Vec<_>>(),
        ));

        Self {
            expr: SequenceExpr::any_of([
                Box::new(SequenceExpr::aco("first").t_ws().then(verbs.clone())),
                Box::new(
                    SequenceExpr::with(verbs)
                        .then_optional(
                            SequenceExpr::whitespace()
                                .then_kind_either(TokenKind::is_noun, TokenKind::is_oov),
                        )
                        .t_ws()
                        .then_word_seq(&["for", "the", "first", "time"]),
                ),
            ]),
        }
    }
}

impl ExprLinter for RedundantFirsts {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (verb_index, redundant_range) = match toks.len() {
            3 => (2, (0, 1)),
            9 => (0, (1, 8)),
            11 => (0, (3, 10)),
            _ => return None,
        };

        let the_toks = toks.get_rel_slice(redundant_range.0, redundant_range.1)?;
        let span = the_toks.span()?;

        let verb = toks[verb_index].get_ch(src);

        let message = match VERBS
            .iter()
            .find(|(v, _)| verb.eq_str(*v))
            .map(|(_, level)| level)?
        {
            RedundancyLevel::Probable => {
                "This is probably redundant as this verb already implies the first time."
            }
            RedundancyLevel::Possible => {
                "Did this occur multiple times? If not, this wording may be redundant."
            }
        }
        .to_owned();

        Some(Lint {
            span,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![Suggestion::Remove],
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Looks for redundant use of `first` with verbs that already imply order."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::RedundantFirsts;

    #[test]
    fn fis_first_coined() {
        assert_suggestion_result(
            "Slopsquatting, as researchers are calling it, is a term first coined by Seth Larson, a security developer-in-residence at Python Software Foundation (PSF), for its resemblance to the typosquatting technique.",
            RedundantFirsts::default(),
            "Slopsquatting, as researchers are calling it, is a term coined by Seth Larson, a security developer-in-residence at Python Software Foundation (PSF), for its resemblance to the typosquatting technique.",
        );
    }

    #[test]
    fn fix_first_invented_solution() {
        assert_suggestion_result(
            "Ben Kamens has a great writeup about a version of the solution, first invented by Bruce Tognazzini in the late 80's.",
            RedundantFirsts::default(),
            "Ben Kamens has a great writeup about a version of the solution, invented by Bruce Tognazzini in the late 80's.",
        );
    }

    #[test]
    fn fix_first_discovered() {
        assert_suggestion_result(
            "I first discovered this bug when installing dependencies for navigator and import it on my App.js.",
            RedundantFirsts::default(),
            "I discovered this bug when installing dependencies for navigator and import it on my App.js.",
        );
    }

    #[test]
    fn fix_first_introduced() {
        assert_suggestion_result(
            "The Transformer architecture is a neural network architecture that was first introduced in 2016.",
            RedundantFirsts::default(),
            "The Transformer architecture is a neural network architecture that was introduced in 2016.",
        );
    }

    #[test]
    fn fix_first_released() {
        assert_suggestion_result(
            "This was supposed to revitalize the Tony Hawk franchise. It was first released in 2009.",
            RedundantFirsts::default(),
            "This was supposed to revitalize the Tony Hawk franchise. It was released in 2009.",
        );
    }

    #[test]
    fn fix_first_invented_c() {
        assert_suggestion_result(
            "C was first invented back in 1972 at Bell Labs.",
            RedundantFirsts::default(),
            "C was invented back in 1972 at Bell Labs.",
        );
    }

    #[test]
    fn fix_first_originated() {
        assert_suggestion_result(
            "The Japanese Foreign Ministry, where the rumors first originated, said it still needed to check the reports but were not able to verify them.",
            RedundantFirsts::default(),
            "The Japanese Foreign Ministry, where the rumors originated, said it still needed to check the reports but were not able to verify them.",
        );
    }

    #[test]
    fn fix_discovered_for_the_first_time() {
        assert_suggestion_result(
            "I've just discovered Harper for the first time, but already struck a number of issues, which I have raised in GitHub",
            RedundantFirsts::default(),
            "I've just discovered Harper, but already struck a number of issues, which I have raised in GitHub",
        );
    }
}
