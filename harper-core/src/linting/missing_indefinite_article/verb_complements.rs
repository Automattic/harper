use super::phrase::{ArticlePhrase, lint_phrases, previous};
use crate::{
    CharStringExt, Dialect, Document, Token,
    linting::{Lint, Linter},
};
use harper_brill::UPOS;

/// Checks objects and nominal predicates after verbs, excluding auxiliary
/// constructions and coordinated adjective predicates.
pub(super) struct VerbComplements(pub Dialect);

impl Linter for VerbComplements {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        lint_phrases(document, self.0, accepts)
    }
    fn description(&self) -> &'static str {
        "Checks missing articles in verb complements."
    }
}

/// Requires a verbal governor and rejects participles already governed by an
/// auxiliary, determiner, or preposition rather than acting as finite verbs.
fn accepts(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    let Some(previous) = previous(tokens, phrase) else {
        return false;
    };
    if previous.kind.is_determiner()
        || !(previous.kind.is_upos(UPOS::VERB) || previous.kind.is_upos(UPOS::AUX))
        || is_coordinated_predicate(tokens, phrase, source)
        || takes_infinitive(previous, phrase, source)
        || is_adjective_predicate(previous, tokens, phrase, source)
    {
        return false;
    }
    if previous.kind.is_upos(UPOS::AUX)
        && phrase.first.kind.is_upos(UPOS::VERB)
        && (phrase.first.kind.is_verb_past_participle_form()
            || phrase.first.kind.is_verb_progressive_form())
    {
        return false;
    }
    if phrase.start >= 4
        && (previous.kind.is_verb_past_form()
            || previous.kind.is_verb_past_participle_form()
            || previous.kind.is_verb_progressive_form())
        && (tokens[phrase.start - 4].kind.is_determiner()
            || tokens[phrase.start - 4].kind.is_preposition())
    {
        return false;
    }
    (phrase.first.span != phrase.head.span
        || previous.get_ch(source).eq_any_ignore_ascii_case_str(&[
            "am", "is", "are", "was", "were", "be", "been", "being", "become", "becomes", "became",
            "have", "has", "had", "need", "needs", "needed", "want", "wants", "wanted", "buy",
            "buys", "bought",
        ]))
        && !(previous.kind.is_verb_past_participle_form()
            && phrase.start >= 4
            && tokens[phrase.start - 4].kind.is_upos(UPOS::AUX))
}

/// Common predicative adjectives may be tagged as nouns ("be patient").
/// Dictionary adjective membership alone is not enough: it includes attributive
/// nouns such as "teacher". Restrict this check to predicative uses, with only
/// adverb modifiers, rather than suppressing nominal objects like "injured patient".
fn is_adjective_predicate(
    governor: &Token,
    tokens: &[Token],
    phrase: &ArticlePhrase<'_>,
    source: &[char],
) -> bool {
    governor
        .get_ch(source)
        .eq_any_ignore_ascii_case_str(&["am", "is", "are", "was", "were", "be", "been", "being"])
        && phrase.head.kind.is_adjective()
        && phrase.head.get_ch(source).eq_any_ignore_ascii_case_str(&[
            "patient",
            "firm",
            "vintage",
            "ready",
            "available",
            "safe",
            "free",
            "open",
            "closed",
            "late",
            "early",
            "well",
            "awake",
            "asleep",
            "alive",
            "alone",
        ])
        && tokens[phrase.start..phrase.end]
            .iter()
            .all(|t| t.kind.is_whitespace() || t.kind.is_adverb())
}

/// Modal and do-support auxiliaries govern verbs, not article-bearing objects.
/// Infinitival `to` can also receive an AUX tag; it never governs a verb object.
/// `Help` can govern either a bare infinitive or a noun phrase; require a tagged
/// base verb there so nominal objects such as "injured child" remain eligible.
fn takes_infinitive(governor: &Token, phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    if governor.kind.is_upos(UPOS::AUX)
        && governor.get_ch(source).eq_any_ignore_ascii_case_str(&[
            "can", "could", "may", "might", "must", "shall", "should", "will", "would", "do",
            "does", "did", "to",
        ])
    {
        return true;
    }
    governor
        .get_ch(source)
        .eq_any_ignore_ascii_case_str(&["help", "helps", "helped", "helping"])
        && phrase.first.kind.is_upos(UPOS::VERB)
        && phrase.first.kind.is_verb_lemma()
}

/// Recognizes coordinated modifiers ending in an adjective, stopping at clause boundaries.
fn is_coordinated_predicate(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    let Some(previous) = phrase.start.checked_sub(2).and_then(|i| tokens.get(i)) else {
        return false;
    };
    if !previous
        .get_ch(source)
        .eq_any_ignore_ascii_case_str(&["am", "is", "are", "was", "were", "be", "been", "being"])
    {
        return false;
    }
    let mut coordinated = false;
    let mut last_word = None;
    for token in &tokens[phrase.end + 1..] {
        if token.kind.is_whitespace() || token.kind.is_comma() {
            continue;
        }
        if token.kind.is_upos(UPOS::CCONJ) {
            coordinated = true;
        } else if token.kind.is_upos(UPOS::NOUN) || token.kind.is_upos(UPOS::ADJ) {
            last_word = Some(token);
        } else {
            break;
        }
    }
    coordinated && last_word.is_some_and(|t| t.kind.is_upos(UPOS::ADJ))
}
