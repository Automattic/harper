use crate::{
    Lint, Token,
    char_ext::CharExt,
    expr::{Expr, SequenceExpr},
    irregular_verbs::IrregularVerbs,
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct WentAheadAndAgreement {
    expr: SequenceExpr,
}

impl Default for WentAheadAndAgreement {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["went", "gone"])
                .t_ws()
                .then_word_seq(&["ahead", "and"])
                .t_ws()
                .then_kind_where(|k| {
                    k.is_verb_lemma()
                        && !k.is_verb_past_form() // looked
                        && !k.is_verb_simple_past_form() // saw
                        && !k.is_verb_past_participle_form() // seen
                }),
        }
    }
}

impl ExprLinter for WentAheadAndAgreement {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));

        let (go_tok, verb2_tok) = (toks.first()?, toks.last()?);

        enum Go {
            Went,
            Gone,
        }
        use Go::*;

        let go: Go = if go_tok.kind.is_verb_simple_past_form() {
            Went
        } else if go_tok.kind.is_verb_past_participle_form() {
            Gone
        } else {
            return None;
        };

        let mut past_verbs: Vec<Vec<char>> = Vec::new();

        let verb2_str = verb2_tok.get_str(src);

        let irreg = IrregularVerbs::curated();
        if let Some(irregular_past) = match go {
            Went => irreg.get_preterite_for_lemma(&verb2_str),
            Gone => irreg.get_past_participle_for_lemma(&verb2_str),
        } {
            past_verbs.push(irregular_past.chars().collect());
        }

        let verb2_ch = verb2_tok.get_ch(src);

        // 1. try adding -d
        let mut verb2_plus_d = verb2_ch.to_vec();
        verb2_plus_d.push('d');
        past_verbs.push(verb2_plus_d);
        // 2. try adding -ed
        let mut verb2_plus_ed = verb2_ch.to_vec();
        verb2_plus_ed.push('e');
        verb2_plus_ed.push('d');
        past_verbs.push(verb2_plus_ed);
        // 3. try double last consonant then adding =ed
        if let Some(last) = verb2_ch.last() 
            && !last.is_vowel() {
                let mut verb2_plus_dd = verb2_ch.to_vec();
                verb2_plus_dd.push(*last);
                verb2_plus_dd.push('e');
                verb2_plus_dd.push('d');
                past_verbs.push(verb2_plus_dd);
            
        }

        let verb2_span = verb2_tok.span;

        let suggestions = past_verbs
            .iter()
            .map(|pv| Suggestion::replace_with_match_case(pv.to_vec(), verb2_span.get_content(src)))
            .collect();

        Some(Lint {
            span: verb2_span,
            lint_kind: LintKind::Agreement,
            suggestions,
            message: "The tense of the verb after `and` should match the tense of `go`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Checks for `went ahead and` followed by a present tense verb, which should be past tense."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::WentAheadAndAgreement;

    #[test]
    fn went_add() {
        assert_suggestion_result(
            "I went ahead and add a note to it's javadocs that the reason argument doesn't impact it's equality",
            WentAheadAndAgreement::default(),
            "I went ahead and added a note to it's javadocs that the reason argument doesn't impact it's equality",
        );
    }

    #[test]
    fn went_build() {
        assert_suggestion_result(
            "I went ahead and build out creating a shiny input from a json schema as as separate package using reactR and react-jsonschema-form",
            WentAheadAndAgreement::default(),
            "I went ahead and build out creating a shiny input from a json schema as as separate package using reactR and react-jsonschema-form",
        );
    }

    #[test]
    fn went_change() {
        assert_suggestion_result(
            "So I went ahead and change the behavior to explicitly fills the default domain into the domain field if no domain is specified.",
            WentAheadAndAgreement::default(),
            "So I went ahead and changed the behavior to explicitly fills the default domain into the domain field if no domain is specified.",
        );
    }

    #[test]
    fn went_do() {
        assert_suggestion_result(
            "compiler automatically identified vectorization opportunities and went ahead and do vectorization",
            WentAheadAndAgreement::default(),
            "compiler automatically identified vectorization opportunities and went ahead and did vectorization",
        )
    }

    #[test]
    fn went_enable() {
        assert_suggestion_result(
            "I went ahead and enable it for those systems and fixed the resulting errors that were previously unsurfaced.",
            WentAheadAndAgreement::default(),
            "I went ahead and enabled it for those systems and fixed the resulting errors that were previously unsurfaced.",
        )
    }

    #[test]
    fn gone_make() {
        assert_suggestion_result(
            "Hi - I've gone ahead and make a conda package for UMICollapse",
            WentAheadAndAgreement::default(),
            "Hi - I've gone ahead and made a conda package for UMICollapse",
        )
    }

    #[test]
    fn gone_open() {
        assert_suggestion_result(
            "I would've gone ahead and open a PR in this project for the style guide",
            WentAheadAndAgreement::default(),
            "I would've gone ahead and opened a PR in this project for the style guide",
        )
    }

    #[test]
    #[ignore = "we're aware of this edge case but we don't handle it yet"]
    fn dont_flag_have_gone_ahead_and_have_added() {
        assert_no_lints(
            "I have gone ahead and have added a +1 and have added your case to the request in support of it.",
            WentAheadAndAgreement::default(),
        )
    }
}
