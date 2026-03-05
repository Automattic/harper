use crate::{
    CharStringExt, Lrc, Token,
    expr::{Expr, FirstMatchOf, LongestMatchOf, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Chunk},
    patterns::{ModalVerb, Word, WordSet},
};

use super::super::NOUN_VERB_PAIRS;

/// Pronouns that can come before verbs but not nouns
const PRONOUNS: &[&str] = &["he", "I", "it", "she", "they", "we", "who", "you"];

/// Linter that corrects common noun/verb confusions
pub(super) struct GeneralNounInsteadOfVerb {
    expr: Box<dyn Expr>,
}

impl Default for GeneralNounInsteadOfVerb {
    fn default() -> Self {
        // Adverbs that can come before verbs but not nouns
        // Note: "Sometimes" can come before a noun.
        let adverb_of_frequency = |tok: &Token, src: &[char]| {
            tok.kind.is_frequency_adverb()
                && !tok
                    .span
                    .get_content(src)
                    .eq_ignore_ascii_case_str("sometimes")
        };

        let pre_context = FirstMatchOf::new(vec![
            Box::new(WordSet::new(PRONOUNS)),
            Box::new(ModalVerb::with_common_errors()),
            Box::new(WordSet::new(&["do", "don't", "dont"])),
            Box::new(adverb_of_frequency),
            Box::new(Word::new("to")),
        ]);

        let nouns = Lrc::new(WordSet::new(
            &NOUN_VERB_PAIRS
                .iter()
                .map(|&(noun, _)| noun)
                .collect::<Vec<_>>(),
        ));

        let basic_pattern = Lrc::new(
            SequenceExpr::with(pre_context)
                .then_whitespace()
                .then(nouns.clone()),
        );

        let pattern_followed_by_punctuation =
            SequenceExpr::with(basic_pattern.clone()).then_punctuation();

        let pattern_followed_by_word = SequenceExpr::with(basic_pattern.clone())
            .then_whitespace()
            .then_any_word();

        Self {
            expr: Box::new(LongestMatchOf::new(vec![
                Box::new(pattern_followed_by_punctuation),
                Box::new(pattern_followed_by_word),
                Box::new(basic_pattern),
            ])),
        }
    }
}

impl ExprLinter for GeneralNounInsteadOfVerb {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let prev_tok = &toks[0];

        // If we have the next word token, try to rule out compound nouns
        if toks.len() > 4 {
            let following_tok = &toks[4];
            if following_tok.kind.is_noun()
                && !following_tok.kind.is_proper_noun()
                && !following_tok.kind.is_preposition()
            {
                // But first rule out marginal "nouns"
                if !following_tok
                    .span
                    .get_content(src)
                    .eq_any_ignore_ascii_case_str(&["it", "me", "on", "that"])
                {
                    return None;
                }
            }

            // If the previous word is "to", use the following word to disambiguate
            if prev_tok
                .span
                .get_content(src)
                .eq_ignore_ascii_case_chars(&['t', 'o'])
                && !following_tok.kind.is_determiner()
            {
                return None;
            }
        }

        // If we don't have the next word token, don't continue if the previous token is "to"
        // since "to" is a preposition and an infinitive marker and there's not enough context to disambiguate.
        if toks.len() <= 4
            && prev_tok
                .span
                .get_content(src)
                .eq_ignore_ascii_case_chars(&['t', 'o'])
        {
            return None;
        }

        let noun_tok = &toks[2];
        let noun_chars = noun_tok.span.get_content(src);
        let noun_text = noun_tok.span.get_content_string(src);
        let noun_lower = noun_text.to_lowercase();

        let verb = NOUN_VERB_PAIRS
            .iter()
            .find(|(noun, _)| *noun == noun_lower)
            .map(|(_, verb)| verb)?;

        Some(Lint {
            span: noun_tok.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                verb.chars().collect(),
                noun_chars,
            )],
            message: format!("`{noun_text}` is a noun, the verb should be `{verb}`."),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects nouns used instead of verbs when the two are related."
    }
}
