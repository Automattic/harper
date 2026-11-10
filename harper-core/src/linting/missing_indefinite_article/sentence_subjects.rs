use super::phrase::{ArticlePhrase, lint_phrases};
use crate::{
    CharStringExt, Dialect, Document, Token,
    linting::{Lint, Linter},
};
use harper_brill::UPOS;

/// Checks initial subjects with a finite predicate, not headings or passive labels.
pub(super) struct SentenceSubjects(pub Dialect);

impl Linter for SentenceSubjects {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        lint_phrases(document, self.0, |tokens, phrase, source| {
            phrase.start == 0
                && phrase.first.span != phrase.head.span
                && has_finite_predicate(tokens, phrase, source)
        })
    }
    fn description(&self) -> &'static str {
        "Checks missing articles in sentence subjects."
    }
}

/// Requires a finite predicate, excluding reduced passive labels without an object.
fn has_finite_predicate(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    let Some(verb) = tokens.get(phrase.end + 2) else {
        return false;
    };
    if !(verb.kind.is_upos(UPOS::VERB) || verb.kind.is_upos(UPOS::AUX))
        || verb.kind.is_verb_progressive_form()
    {
        return false;
    }
    if tokens
        .get(phrase.end + 4)
        .is_some_and(|next| next.get_ch(source).eq_any_ignore_ascii_case_str(&["by"]))
    {
        return false;
    }
    if verb.kind.is_verb_past_participle_form() || verb.kind.is_verb_past_form() {
        return (verb.kind.is_verb_simple_past_form() || verb.kind.is_verb_past_form())
            && tokens.get(phrase.end + 4).is_none_or(|next| {
                next.kind.is_sentence_terminator()
                    || ((next.kind.is_upos(UPOS::DET)
                        || next.kind.is_upos(UPOS::NOUN)
                        || next.kind.is_upos(UPOS::PRON))
                        && !next.kind.is_preposition())
            });
    }
    true
}
