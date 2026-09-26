use crate::{
    DictWordMetadata, Lint, Token,
    char_ext::CharExt,
    expr::{Expr, SequenceExpr},
    irregular_verbs::IrregularVerbs,
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    spell::Dictionary,
};

pub struct WentAheadAndAgreement<D: Dictionary> {
    expr: SequenceExpr,
    dict: D,
}

impl<D: Dictionary> WentAheadAndAgreement<D> {
    pub fn new(dict: D) -> Self {
        Self {
            expr: SequenceExpr::word_set(["went", "gone"])
                .t_ws()
                .then_word_seq(&["ahead", "and"])
                .t_ws()
                .then_kind_where(|k| {
                    k.is_verb_lemma()
                        && !k.is_verb_past_form()
                        && !k.is_verb_simple_past_form()
                        && !k.is_verb_past_participle_form()
                }),
            dict,
        }
    }
}

impl<D: Dictionary> ExprLinter for WentAheadAndAgreement<D> {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let go_tok = toks.first()?;
        let verb2_tok = toks.last()?;

        // Determine which irregular lookup method to use based on the token form
        let is_went = go_tok.kind.is_verb_simple_past_form();
        let is_gone = go_tok.kind.is_verb_past_participle_form();
        if !is_went && !is_gone {
            return None;
        }

        let mut past_verbs: Vec<Vec<char>> = Vec::new();
        let verb2_str = verb2_tok.get_str(src);

        // Handle irregular verbs
        let irreg = IrregularVerbs::curated();
        let irregular_past = if is_went {
            irreg.get_preterite_for_lemma(&verb2_str)
        } else {
            irreg.get_past_participle_for_lemma(&verb2_str)
        };

        if let Some(past) = irregular_past {
            past_verbs.push(past.chars().collect());
        }

        // Handle regular verb suffix variations
        let verb2_ch = verb2_tok.get_ch(src);

        let mut candidates = vec![
            [verb2_ch, &['d']].concat(),
            [verb2_ch, &['e', 'd']].concat(),
        ];

        if let Some(&last) = verb2_ch.last()
            && !last.is_vowel()
        {
            candidates.push([verb2_ch, &[last, 'e', 'd']].concat());
        }

        let is_valid_tense = |md: &DictWordMetadata| {
            md.is_verb_past_form()
                || (is_went && md.is_verb_simple_past_form())
                || (is_gone && md.is_verb_past_participle_form())
        };

        for candidate in candidates {
            if let Some(md) = self.dict.get_word_metadata(&candidate)
                && is_valid_tense(&md)
            {
                past_verbs.push(candidate);
            }
        }

        let verb2_span = verb2_tok.span;
        let original_content = verb2_span.get_content(src);
        let suggestions = past_verbs
            .into_iter()
            .map(|pv| Suggestion::replace_with_match_case(pv, original_content))
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
    use crate::{
        linting::tests::{assert_no_lints, assert_suggestion_result},
        spell::FstDictionary,
    };

    use super::WentAheadAndAgreement;

    // Contrived test for doubled-consonant ending

    #[test]
    fn went_ban() {
        assert_suggestion_result(
            "He went ahead and spam the Discord so I've gone ahead and ban him.",
            WentAheadAndAgreement::new(FstDictionary::curated()),
            "He went ahead and spammed the Discord so I've gone ahead and banned him.",
        )
    }

    // Real-world tests for regular and irregular preterite and past participles

    #[test]
    fn went_add() {
        assert_suggestion_result(
            "I went ahead and add a note to it's javadocs that the reason argument doesn't impact it's equality",
            WentAheadAndAgreement::new(FstDictionary::curated()),
            "I went ahead and added a note to it's javadocs that the reason argument doesn't impact it's equality",
        );
    }

    #[test]
    fn went_build() {
        assert_suggestion_result(
            "I went ahead and build out creating a shiny input from a json schema as as separate package using reactR and react-jsonschema-form",
            WentAheadAndAgreement::new(FstDictionary::curated()),
            "I went ahead and build out creating a shiny input from a json schema as as separate package using reactR and react-jsonschema-form",
        );
    }

    #[test]
    fn went_change() {
        assert_suggestion_result(
            "So I went ahead and change the behavior to explicitly fills the default domain into the domain field if no domain is specified.",
            WentAheadAndAgreement::new(FstDictionary::curated()),
            "So I went ahead and changed the behavior to explicitly fills the default domain into the domain field if no domain is specified.",
        );
    }

    #[test]
    fn went_do() {
        assert_suggestion_result(
            "compiler automatically identified vectorization opportunities and went ahead and do vectorization",
            WentAheadAndAgreement::new(FstDictionary::curated()),
            "compiler automatically identified vectorization opportunities and went ahead and did vectorization",
        )
    }

    #[test]
    fn went_enable() {
        assert_suggestion_result(
            "I went ahead and enable it for those systems and fixed the resulting errors that were previously unsurfaced.",
            WentAheadAndAgreement::new(FstDictionary::curated()),
            "I went ahead and enabled it for those systems and fixed the resulting errors that were previously unsurfaced.",
        )
    }

    #[test]
    fn gone_make() {
        assert_suggestion_result(
            "Hi - I've gone ahead and make a conda package for UMICollapse",
            WentAheadAndAgreement::new(FstDictionary::curated()),
            "Hi - I've gone ahead and made a conda package for UMICollapse",
        )
    }

    #[test]
    fn gone_open() {
        assert_suggestion_result(
            "I would've gone ahead and open a PR in this project for the style guide",
            WentAheadAndAgreement::new(FstDictionary::curated()),
            "I would've gone ahead and opened a PR in this project for the style guide",
        )
    }

    #[test]
    #[ignore = "we're aware of this edge case but we don't handle it yet"]
    fn dont_flag_have_gone_ahead_and_have_added() {
        assert_no_lints(
            "I have gone ahead and have added a +1 and have added your case to the request in support of it.",
            WentAheadAndAgreement::new(FstDictionary::curated()),
        )
    }
}
