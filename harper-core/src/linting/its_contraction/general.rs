use harper_brill::UPOS;

use crate::{
    Document, Token, TokenStringExt,
    expr::{All, Expr, ExprExt, OwnedExprExt, SequenceExpr},
    linting::{Lint, LintKind, Linter, Suggestion},
    patterns::{NominalPhrase, Pattern, UPOSSet, WordSet},
};

pub struct General {
    expr: Box<dyn Expr>,
}

impl Default for General {
    fn default() -> Self {
        let positive = SequenceExpr::default().t_aco("its").then_whitespace().then(
            UPOSSet::new(&[UPOS::VERB, UPOS::AUX, UPOS::DET, UPOS::PRON])
                .or(WordSet::new(&[
                    "anywhere",
                    "everywhere",
                    "somewhere",
                    "nowhere",
                ]))
                .or(WordSet::new(&["because"])),
        );

        let exceptions = SequenceExpr::anything()
            .then_anything()
            .then_word_set(&["own", "intended"]);

        let inverted = SequenceExpr::unless(exceptions);

        let expr = All::new(vec![Box::new(positive), Box::new(inverted)]).or_longest(
            SequenceExpr::aco("its")
                .t_ws()
                .then(UPOSSet::new(&[UPOS::ADJ]))
                .t_ws()
                .then(UPOSSet::new(&[UPOS::SCONJ, UPOS::PART])),
        );

        Self {
            expr: Box::new(expr),
        }
    }
}

impl Linter for General {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let source = document.get_source();

        for chunk in document.iter_chunks() {
            lints.extend(
                self.expr
                    .iter_matches(chunk, source)
                    .filter_map(|match_span| {
                        self.match_to_lint(&chunk[match_span.start..], source)
                    }),
            );
        }

        lints
    }

    fn description(&self) -> &str {
        "Detects the possessive `its` before `had`, `been`, or `got` and offers `it's` or `it has`."
    }
}

impl General {
    fn match_to_lint(&self, toks: &[Token], source: &[char]) -> Option<Lint> {
        let offender = toks.first()?;
        let offender_chars = offender.get_ch(source);

        let modifier = toks.get(2)?;
        let modifier_text = modifier.get_str(source);
        let modifier_lower = modifier_text.to_ascii_lowercase();
        let next_kind = toks.get(4).map(|tok| tok.kind.clone());

        if preceding_word(source, offender.span.start).as_deref() == Some("at")
            && matches!(
                modifier_text.as_str(),
                "highest" | "lowest" | "best" | "worst"
            )
        {
            return None;
        }

        let exact_contraction_words = [
            "anybody",
            "anyone",
            "anything",
            "anywhere",
            "everybody",
            "everyone",
            "everything",
            "everywhere",
            "nobody",
            "nothing",
            "nowhere",
            "somebody",
            "someone",
            "something",
            "somewhere",
            "because",
        ];

        let determiner_like_words = [
            "a", "an", "my", "your", "his", "her", "our", "their", "this", "that",
        ];

        let contraction_adjectives = ["common", "easy", "hard"];

        let strong_predicative_verbs = [
            "had", "been", "got", "called", "named", "known", "termed", "titled",
        ];

        let should_consider = if exact_contraction_words.contains(&modifier_lower.as_str())
            || determiner_like_words.contains(&modifier_lower.as_str())
        {
            true
        } else if modifier.kind.is_upos(UPOS::ADJ) {
            contraction_adjectives.contains(&modifier_lower.as_str())
                && next_kind
                    .is_some_and(|kind| kind.is_upos(UPOS::SCONJ) || kind.is_upos(UPOS::PART))
        } else if modifier.kind.is_upos(UPOS::VERB) || modifier.kind.is_upos(UPOS::AUX) {
            let blocks_contraction = !strong_predicative_verbs.contains(&modifier_lower.as_str())
                && (next_non_whitespace_word(source, modifier.span.end).is_some_and(|word| {
                    matches!(
                        word.as_str(),
                        "is" | "was" | "were" | "be" | "been" | "being" | "to"
                    )
                }) || next_kind.is_some_and(|kind| kind.is_noun() || kind.is_proper_noun()));

            !blocks_contraction
        } else {
            false
        };

        if !should_consider {
            return None;
        }

        if modifier.kind.is_upos(UPOS::VERB)
            && NominalPhrase.matches(&toks[2..], source).is_some()
            && !Self::is_likely_predicative_participle(modifier, source)
        {
            return None;
        }

        // Past-participle modifiers can be tagged as verbs even in possessive noun phrases:
        // "its abetted parameter", "its associated parameter", etc.
        if self.is_possessive_participle_noun_phrase(toks, source) {
            return None;
        }

        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::Punctuation,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("it's", offender_chars),
                Suggestion::replace_with_match_case_str("it has", offender_chars),
            ],
            message: "Use `it's` (short for `it has` or `it is`) here, not the possessive `its`."
                .to_owned(),
            priority: 54,
        })
    }

    fn is_possessive_participle_noun_phrase(&self, toks: &[Token], source: &[char]) -> bool {
        let Some(modifier) = toks.get(2) else {
            return false;
        };
        let Some(gap) = toks.get(3) else {
            return false;
        };
        let Some(head) = toks.get(4) else {
            return false;
        };

        if !modifier.kind.is_verb_past_participle_form() || !gap.kind.is_whitespace() {
            return false;
        }

        if !(head.kind.is_noun() || head.kind.is_proper_noun()) {
            return false;
        }

        if Self::is_likely_predicative_participle(modifier, source) {
            return false;
        }

        let modifier_text = modifier.get_str(source);

        !["had", "been", "got"]
            .iter()
            .any(|word| modifier_text.eq_ignore_ascii_case(word))
    }

    fn is_likely_predicative_participle(tok: &Token, source: &[char]) -> bool {
        let text = tok.get_str(source);

        ["called", "named", "known", "termed", "titled"]
            .iter()
            .any(|word| text.eq_ignore_ascii_case(word))
    }
}

fn preceding_word(source: &[char], offset: usize) -> Option<String> {
    let prefix = source.get(..offset)?;
    let mut i = prefix.len().checked_sub(1)?;

    while prefix[i].is_whitespace() {
        i = i.checked_sub(1)?;
    }

    let start = prefix[..=i]
        .iter()
        .rposition(|c| c.is_whitespace())
        .map(|pos| pos + 1)
        .unwrap_or(0);

    Some(
        prefix[start..=i]
            .iter()
            .collect::<String>()
            .to_ascii_lowercase(),
    )
}

fn next_non_whitespace_word(source: &[char], offset: usize) -> Option<String> {
    let suffix = source.get(offset..)?;
    let mut iter = suffix
        .iter()
        .enumerate()
        .skip_while(|(_, c)| c.is_whitespace());
    let start = iter.next()?.0;
    let end = suffix[start..]
        .iter()
        .position(|c| c.is_whitespace() || c.is_ascii_punctuation())
        .map(|len| start + len)
        .unwrap_or(suffix.len());

    Some(
        suffix[start..end]
            .iter()
            .collect::<String>()
            .to_ascii_lowercase(),
    )
}
