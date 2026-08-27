use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct DigUnderTheHood {
    expr: SequenceExpr,
}

impl Default for DigUnderTheHood {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["dig", "digging", "digs", "dug"])
                .t_ws()
                .then_word_seq(&["under", "the"])
                .t_ws()
                .t_set(&["hood", "bonnet"]),
        }
    }
}

impl ExprLinter for DigUnderTheHood {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;
        let dig = span.get_content(source);

        let look = match dig {
            [.., a, b] => match (a.to_ascii_lowercase(), b.to_ascii_lowercase()) {
                ('i', 'g') => "look",
                ('n', 'g') => "looking",
                ('g', 's') => "looks",
                ('u', 'g') => "looked",
                _ => return None,
            },
            _ => return None,
        };

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                look.chars().collect::<Vec<char>>(),
                dig,
            )],
            message: "If the context is not automechanics, you may be mixing metaphors.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects the mixed metaphor of `digging under the hood/bonnet`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::DigUnderTheHood;

    #[test]
    fn dig_hood() {
        assert_suggestion_result(
            "But then when we actually like, you know, dig under the hood, what we see so much is that, you know, companies are reporting that they're not seeing the return on investment that they expected from generative AI when you have so many investment dollars already bound up in this vision.",
            DigUnderTheHood::default(),
            "But then when we actually like, you know, look under the hood, what we see so much is that, you know, companies are reporting that they're not seeing the return on investment that they expected from generative AI when you have so many investment dollars already bound up in this vision.",
        );
    }

    #[test]
    fn digging_hood() {
        assert_suggestion_result(
            "But then when we actually like, you know, dig under the hood, what we see so much is that, you know, companies are reporting that they're not seeing the return on investment that they expected from generative AI when you have so many investment dollars already bound up in this vision.",
            DigUnderTheHood::default(),
            "But then when we actually like, you know, look under the hood, what we see so much is that, you know, companies are reporting that they're not seeing the return on investment that they expected from generative AI when you have so many investment dollars already bound up in this vision.",
        );
    }

    #[test]
    fn digs_hood() {
        assert_suggestion_result(
            "But Ben digs under the hood and the underlying metrics still look real: same attack angle, same bat speed, still hitting the ball hard.",
            DigUnderTheHood::default(),
            "But Ben looks under the hood and the underlying metrics still look real: same attack angle, same bat speed, still hitting the ball hard.",
        );
    }

    #[test]
    fn dug_hood() {
        assert_suggestion_result(
            "Love that you dug under the hood to turn off output capture to figure out where things were hanging up.",
            DigUnderTheHood::default(),
            "Love that you looked under the hood to turn off output capture to figure out where things were hanging up.",
        );
    }

    #[test]
    fn dig_bonnet() {
        assert_suggestion_result(
            "Dig under the bonnet of that crude metric and you'll find Armagh, Galway, Donegal and Kerry have panels built to last 70 minutes.",
            DigUnderTheHood::default(),
            "Look under the bonnet of that crude metric and you'll find Armagh, Galway, Donegal and Kerry have panels built to last 70 minutes.",
        );
    }

    #[test]
    fn digging_bonnet() {
        assert_suggestion_result(
            "When your days are consumed with writing, analysing data and digging under the bonnet of other people's businesses to find the award-winning gold, ...",
            DigUnderTheHood::default(),
            "When your days are consumed with writing, analysing data and looking under the bonnet of other people's businesses to find the award-winning gold, ...",
        );
    }

    #[test]
    fn dug_bonnet() {
        assert_suggestion_result(
            "Don't know if anyone has dug under the bonnet yet, but any opinions as to whether .htaccess support is something that could be achieved via a patch or plugin?",
            DigUnderTheHood::default(),
            "Don't know if anyone has looked under the bonnet yet, but any opinions as to whether .htaccess support is something that could be achieved via a patch or plugin?",
        );
    }
}
