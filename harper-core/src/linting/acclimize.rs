use crate::{
    Lint, Token,
    expr::Expr,
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

pub struct Acclimize {
    expr: WordSet,
}

impl Default for Acclimize {
    fn default() -> Self {
        Self {
            expr: WordSet::new(&[
                "acclimise",
                "acclimised",
                "acclimises",
                "acclimising",
                "acclimize",
                "acclimized",
                "acclimizes",
                "acclimizing",
            ]),
        }
    }
}

impl ExprLinter for Acclimize {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (ize, ate) = (
            match (toks[0].get_ch(src)[7], toks[0].get_ch(src).last()?) {
                ('s', 'e') => "acclimatise",
                ('z', 'e') => "acclimatize",
                ('s', 'd') => "acclimatised",
                ('z', 'd') => "acclimatized",
                ('s', 's') => "acclimatises",
                ('z', 's') => "acclimatizes",
                ('s', 'g') => "acclimatising",
                ('z', 'g') => "acclimatizing",
                _ => return None,
            },
            match toks[0].get_ch(src).last()? {
                'e' => "acclimate",
                'd' => "acclimated",
                's' => "acclimates",
                'g' => "acclimating",
                _ => return None,
            },
        );

        let suggestions = [ize, ate]
            .iter()
            .map(|&s| Suggestion::replace_with_match_case(s.chars().collect(), toks[0].get_ch(src)))
            .collect();
        let message = "Did you mean `acclimate` or `acclimatize`/`acclimise`?".to_string();

        Some(Lint {
            span: toks[0].span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `acclitize`/`acclimise` to `acclimate` and `acclimatise`/`acclimatize`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_good_and_bad_suggestions;

    use super::Acclimize;

    #[test]
    fn acclimize() {
        assert_good_and_bad_suggestions(
            "It takes about 2 weeks for your body to fully acclimize.",
            Acclimize::default(),
            &[
                "It takes about 2 weeks for your body to fully acclimatize.",
                "It takes about 2 weeks for your body to fully acclimate.",
            ],
            &[],
        );
    }

    #[test]
    fn acclimise() {
        assert_good_and_bad_suggestions(
            "he's reached his accessibility limit and must wait a week to acclimise",
            Acclimize::default(),
            &[
                "he's reached his accessibility limit and must wait a week to acclimatise",
                "he's reached his accessibility limit and must wait a week to acclimate",
            ],
            &[],
        );
    }

    #[test]
    fn acclimized() {
        assert_good_and_bad_suggestions(
            "You acclimized to flip and slide phones, no wonder modern phones are so foreign.",
            Acclimize::default(),
            &[
                "You acclimated to flip and slide phones, no wonder modern phones are so foreign.",
                "You acclimatized to flip and slide phones, no wonder modern phones are so foreign.",
            ],
            &[],
        );
    }

    #[test]
    fn acclimised() {
        assert_good_and_bad_suggestions(
            "Anything less than this and you'll risk not being properly acclimised.",
            Acclimize::default(),
            &[
                "Anything less than this and you'll risk not being properly acclimated.",
                "Anything less than this and you'll risk not being properly acclimatised.",
            ],
            &[],
        );
    }

    #[test]
    fn acclimises() {
        assert_good_and_bad_suggestions(
            "as if my body acclimises to the drug before the 20mg or so hits the stomach",
            Acclimize::default(),
            &[
                "as if my body acclimates to the drug before the 20mg or so hits the stomach",
                "as if my body acclimatises to the drug before the 20mg or so hits the stomach",
            ],
            &[],
        );
    }

    #[test]
    fn acclimizing() {
        assert_good_and_bad_suggestions(
            "I can't afford to spend a week acclimizing in Ecuador beforehand",
            Acclimize::default(),
            &[
                "I can't afford to spend a week acclimating in Ecuador beforehand",
                "I can't afford to spend a week acclimatizing in Ecuador beforehand",
            ],
            &[],
        );
    }

    #[test]
    fn acclimising() {
        assert_good_and_bad_suggestions(
            "So, some leaders are acclimising to uncertainty but much of their workforce are not feeling the same sense of optimism.",
            Acclimize::default(),
            &[
                "So, some leaders are acclimating to uncertainty but much of their workforce are not feeling the same sense of optimism.",
                "So, some leaders are acclimatising to uncertainty but much of their workforce are not feeling the same sense of optimism.",
            ],
            &[],
        );
    }
}
