use super::phrase::{ArticlePhrase, lint_phrases, previous};
use crate::{
    CharStringExt, Dialect, Document, Token,
    linting::{Lint, Linter},
};
use harper_brill::UPOS;

/// Checks prepositional objects, preserving institutional bare nouns and idioms.
pub(super) struct PrepositionalObjects(pub Dialect);

impl Linter for PrepositionalObjects {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        lint_phrases(document, self.0, accepts)
    }
    fn description(&self) -> &'static str {
        "Checks missing articles in prepositional objects."
    }
}

/// Requires a preposition in its syntactic role rather than dictionary membership
/// alone. Bare institutions and repeated-noun idioms permit omission of an article.
fn accepts(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    // Quoted expressions can name language rather than countable objects.
    if tokens[phrase.start].kind.is_quote() {
        return false;
    }
    let Some(previous) = previous(tokens, phrase) else {
        return false;
    };
    if previous.kind.is_determiner()
        || !previous.kind.is_preposition()
        || !(previous.kind.is_upos(UPOS::ADP) || previous.kind.is_upos(UPOS::SCONJ))
    {
        return false;
    }
    // Infinitival `to` may be tagged as a preposition before a hyphenated
    // verb. Only a complete base-verb compound is excluded, not a modifier
    // followed by a nominal head ("to red-brick tower").
    if previous
        .get_ch(source)
        .eq_any_ignore_ascii_case_str(&["to"])
        && phrase.end == phrase.start + 2
        && tokens[phrase.start + 1].kind.is_hyphen()
        && phrase.head.kind.is_verb_lemma()
        && (phrase.first.kind.is_verb_lemma()
            || phrase
                .first
                .get_ch(source)
                .eq_any_ignore_ascii_case_str(&["re", "de", "un"]))
    {
        return false;
    }
    if phrase.first.span == phrase.head.span
        && !previous.get_ch(source).eq_any_ignore_ascii_case_str(&[
            "during",
            "throughout",
            "before",
            "after",
            "until",
        ])
    {
        return false;
    }
    if previous
        .get_ch(source)
        .eq_any_ignore_ascii_case_str(&["for"])
        && phrase
            .first
            .get_ch(source)
            .eq_any_ignore_ascii_case_str(&["good"])
        && phrase
            .head
            .get_ch(source)
            .eq_any_ignore_ascii_case_str(&["measure"])
    {
        return false;
    }
    if phrase.start >= 4
        && tokens[phrase.start - 4]
            .get_ch(source)
            .iter()
            .map(char::to_ascii_lowercase)
            .eq(phrase
                .head
                .get_ch(source)
                .iter()
                .map(char::to_ascii_lowercase))
    {
        return false;
    }
    !phrase.head.get_ch(source).eq_any_ignore_ascii_case_str(&[
        "today",
        "tomorrow",
        "yesterday",
        "command",
        "school",
        "bed",
        "church",
        "prison",
        "jail",
        "college",
        "university",
        "hospital",
    ]) && !previous
        .get_ch(source)
        .eq_any_ignore_ascii_case_str(&["as"])
}
