use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    expr::{Expr, SequenceExpr},
    irregular_verbs::IrregularVerbs,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct PastParticipleWithoutAuxiliary {
    expr: SequenceExpr,
}

impl Default for PastParticipleWithoutAuxiliary {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default().then_verb_past_participle_form(),
        }
    }
}

impl ExprLinter for PastParticipleWithoutAuxiliary {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        src: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let target = matched_tokens.first()?;
        if !target.kind.is_verb_past_participle_form() || target.kind.is_verb_simple_past_form() {
            return None;
        }

        let (before, _) = context?;
        let leading_idx = token_before_verb_phrase(before)?;
        let leading_token = &before[leading_idx];

        if is_auxiliary_word(leading_token, src) {
            return None;
        }

        let past_participle = target.get_str(src);
        let irregular_verbs = IrregularVerbs::curated();
        let simple_past = irregular_verbs
            .get_preterite_for_past_participle(&past_participle)
            .filter(|simple_past| !simple_past.eq_ignore_ascii_case(&past_participle))?;

        let should_flag = if leading_token.get_str(src).eq_ignore_ascii_case("and") {
            let previous_verb_idx = find_previous_verb(&before[..leading_idx])?;
            let previous_verb = &before[previous_verb_idx];

            (previous_verb.kind.is_verb_simple_past_form()
                || previous_verb.kind.is_verb_past_form())
                && !has_auxiliary_before(before, previous_verb_idx, src)
        } else {
            (leading_token.kind.is_pronoun()
                || leading_token.kind.is_noun()
                || leading_token.kind.is_proper_noun())
                && !has_auxiliary_before(before, before.len(), src)
        };

        if !should_flag {
            return None;
        }

        Some(Lint {
            span: target.span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case(
                simple_past.chars().collect(),
                target.get_ch(src),
            )],
            message: format!(
                "Use the simple past `{}` instead of the past participle `{}` when there is no auxiliary verb.",
                simple_past, past_participle
            ),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects irregular past participles used without an auxiliary verb."
    }
}

fn token_before_verb_phrase(before: &[Token]) -> Option<usize> {
    let mut i = before.len();

    while i > 0 && before[i - 1].kind.is_whitespace() {
        i -= 1;
    }

    while i > 0 && before[i - 1].kind.is_adverb() {
        i -= 1;
        while i > 0 && before[i - 1].kind.is_whitespace() {
            i -= 1;
        }
    }

    i.checked_sub(1)
}

fn find_previous_verb(before: &[Token]) -> Option<usize> {
    before.iter().rposition(|tok| tok.kind.is_verb())
}

fn has_auxiliary_before(before: &[Token], verb_idx: usize, src: &[char]) -> bool {
    if verb_idx == 0 {
        return false;
    }

    let mut i = verb_idx - 1;
    loop {
        let tok = &before[i];
        if tok.kind.is_whitespace() {
            if i == 0 {
                break;
            }
            i -= 1;
            continue;
        }

        if tok.kind.is_adverb() {
            if i == 0 {
                break;
            }
            i -= 1;
            continue;
        }

        if is_auxiliary_word(tok, src) {
            return true;
        }

        break;
    }

    false
}

fn is_auxiliary_word(tok: &Token, src: &[char]) -> bool {
    let word = tok.get_str(src);
    [
        "am", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "having",
        "i've", "you've", "we've", "they've", "he's", "she's", "it's", "ive", "youve", "weve",
        "theyve", "hes", "shes", "its",
    ]
    .iter()
    .any(|aux| aux.eq_ignore_ascii_case(&word))
}

#[cfg(test)]
mod tests {
    use super::PastParticipleWithoutAuxiliary;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn fixes_simple_case() {
        assert_suggestion_result(
            "I seen a Spectrum.",
            PastParticipleWithoutAuxiliary::default(),
            "I saw a Spectrum.",
        );
    }

    #[test]
    fn fixes_with_never_and_conjunction() {
        assert_suggestion_result(
            "I never used and never seen a zx spectrum in real life.",
            PastParticipleWithoutAuxiliary::default(),
            "I never used and never saw a zx spectrum in real life.",
        );
    }

    #[test]
    fn fixes_with_single_adverb() {
        assert_suggestion_result(
            "We walked and quickly seen the issue.",
            PastParticipleWithoutAuxiliary::default(),
            "We walked and quickly saw the issue.",
        );
    }

    #[test]
    fn fixes_multiple_pairing() {
        assert_suggestion_result(
            "She left and never gone back.",
            PastParticipleWithoutAuxiliary::default(),
            "She left and never went back.",
        );
    }

    #[test]
    fn fixes_run_form() {
        assert_suggestion_result(
            "They moved and then run the benchmark.",
            PastParticipleWithoutAuxiliary::default(),
            "They moved and then ran the benchmark.",
        );
    }

    #[test]
    fn fixes_done_form() {
        assert_suggestion_result(
            "I started and then done the migration.",
            PastParticipleWithoutAuxiliary::default(),
            "I started and then did the migration.",
        );
    }

    #[test]
    fn fixes_known_form() {
        assert_suggestion_result(
            "I read and always known the risk.",
            PastParticipleWithoutAuxiliary::default(),
            "I read and always knew the risk.",
        );
    }

    #[test]
    fn fixes_written_form() {
        assert_suggestion_result(
            "She drafted and finally written the summary.",
            PastParticipleWithoutAuxiliary::default(),
            "She drafted and finally wrote the summary.",
        );
    }

    #[test]
    fn does_not_flag_have_seen() {
        assert_no_lints(
            "I have seen a Spectrum.",
            PastParticipleWithoutAuxiliary::default(),
        );
    }

    #[test]
    fn does_not_flag_had_gone() {
        assert_no_lints(
            "They had gone before lunch.",
            PastParticipleWithoutAuxiliary::default(),
        );
    }

    #[test]
    fn does_not_flag_shared_auxiliary() {
        assert_no_lints(
            "I have used and seen many tools.",
            PastParticipleWithoutAuxiliary::default(),
        );
    }

    #[test]
    fn does_not_flag_be_passive() {
        assert_no_lints(
            "The issue was seen by everyone.",
            PastParticipleWithoutAuxiliary::default(),
        );
    }

    #[test]
    fn does_not_flag_when_second_verb_is_not_participle() {
        assert_no_lints(
            "I used and saw a Spectrum.",
            PastParticipleWithoutAuxiliary::default(),
        );
    }

    #[test]
    fn does_not_flag_when_no_previous_verb() {
        assert_no_lints(
            "And seen from afar, it looks fine.",
            PastParticipleWithoutAuxiliary::default(),
        );
    }

    #[test]
    fn does_not_flag_regular_past_participle() {
        assert_no_lints(
            "I walked and painted the wall.",
            PastParticipleWithoutAuxiliary::default(),
        );
    }
}
