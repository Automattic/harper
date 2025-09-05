use super::{ExprLinter, Suggestion};
use crate::Lint;
use crate::expr::{Expr, LongestMatchOf, SequenceExpr};
use crate::linting::LintKind;
use crate::linting::expr_linter::find_the_only_token_matching;
use crate::{CharStringExt, Token};

pub struct Cant {
    expr: Box<dyn Expr>,
}

impl Default for Cant {
    fn default() -> Self {
        let nom_cant = SequenceExpr::default()
            .then_kind_except(|kind| kind.is_nominal(), &["or"])
            .t_ws()
            .t_aco("cant");
        let cant_pron = SequenceExpr::aco("cant").t_ws().then_personal_pronoun();
        let cant_verb = SequenceExpr::aco("cant")
            .t_ws()
            .then_kind_is_but_is_not(|kind| kind.is_verb(), |kind| kind.is_noun());

        Self {
            expr: Box::new(LongestMatchOf::new(vec![
                Box::new(nom_cant),
                Box::new(cant_pron),
                Box::new(cant_verb),
            ])),
        }
    }
}

// TODO: This can be removed once #1730 is merged
fn is_verb_lemma(tok: &Token, src: &[char]) -> bool {
    tok.kind.is_verb()
        && !tok
            .span
            .get_content(src)
            .ends_with_ignore_ascii_case_chars(&['s'])
        && !tok
            .span
            .get_content(src)
            .ends_with_ignore_ascii_case_chars(&['e', 'd'])
        && !tok
            .span
            .get_content(src)
            .ends_with_ignore_ascii_case_chars(&['i', 'n', 'g'])
}

impl ExprLinter for Cant {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let token = find_the_only_token_matching(toks, src, |tok, src| {
            tok.span
                .get_content(src)
                .eq_ignore_ascii_case_chars(&['c', 'a', 'n', 't'])
        })?;

        let jargon = token.span.get_content(src);
        let cannot = "can't";

        Some(Lint {
            span: token.span,
            lint_kind: LintKind::Enhancement,
            suggestions: vec![Suggestion::replace_with_match_case_str(cannot, jargon)],
            message: "`Cant` is secret language or jargon. If that's not what you mean you should use `can't` here.".to_string(),
            priority: 127,
        })
    }

    fn description(&self) -> &'static str {
        "Suggests correcting `cant` to `can't`."
    }
}

#[cfg(test)]
mod tests {
    use super::Cant;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_pronoun_cant() {
        assert_suggestion_result(
            "I cant go to the store.",
            Cant::default(),
            "I can't go to the store.",
        );
    }

    #[test]
    fn corrects_proper_noun_cant() {
        assert_suggestion_result(
            "Bob cant go to the store.",
            Cant::default(),
            "Bob can't go to the store.",
        );
    }

    #[test]
    fn corrects_common_noun_cant() {
        // "dog" and "cat" are
        assert_suggestion_result(
            "A horse cant drink bottled water.",
            Cant::default(),
            "A horse can't drink bottled water.",
        );
    }

    #[test]
    fn corrects_cant_pronoun() {
        assert_suggestion_result(
            "Cant you go to the store?",
            Cant::default(),
            "Can't you go to the store?",
        );
    }

    #[test]
    fn dont_flag_if_cant_is_part_of_noun_phrase() {
        assert_lint_count("Cant cant be the same as jargon.", Cant::default(), 0);
    }

    #[test]
    fn dont_flag_cant_project() {
        assert_lint_count(
            "The CANT project is designed to allow people to screw around with CAN easily at layers 1/2.",
            Cant::default(),
            0,
        );
    }

    #[test]
    #[ignore = "'Convert' is also a noun, so a 'cant convert' could be a person who switched to speaking jargon"]
    fn corrects_cant_verb() {
        assert_suggestion_result(
            "Cant convert widget to input",
            Cant::default(),
            "Can't convert widget to input",
        );
    }

    #[test]
    fn dont_flag_legit_noun_sense() {
        assert_lint_count(
            "CB Slang Dictionary is the distinctive anti-language, argot or cant which developed amongst users of citizens' band radio (CB), especially truck drivers",
            Cant::default(),
            0,
        );
    }
}
