use super::phrase::ArticlePhrase;
use crate::{CharStringExt, Token};

/// Recognizes bare temporal expressions, names, and explicit template slots.
/// These checks concern the complete phrase, not membership of its head in a
/// global exception list: an ordinary "old station" must remain eligible.
pub(super) fn allows_bare(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    let head = phrase.head.get_ch(source);
    let modifiers = &tokens[phrase.start..phrase.end];
    let named = phrase.first.span != phrase.head.span
        && head.first().is_some_and(|c| c.is_uppercase())
        && head.iter().any(|c| c.is_lowercase());
    let relative_time = head.eq_any_ignore_ascii_case_str(&[
        "second",
        "minute",
        "hour",
        "day",
        "week",
        "month",
        "year",
        "decade",
        "century",
        "morning",
        "afternoon",
        "evening",
        "night",
        "spring",
        "summer",
        "autumn",
        "winter",
    ]) && modifiers.iter().any(|t| {
        t.get_ch(source)
            .eq_any_ignore_ascii_case_str(&["last", "next", "this", "that"])
    });
    let season = head.eq_any_ignore_ascii_case_str(&["season"])
        && modifiers.iter().any(|t| {
            t.get_ch(source)
                .eq_any_ignore_ascii_case_str(&["peak", "off", "high", "low", "breeding", "mating"])
        });
    // Meat cuts can name food as a material; the same body-part noun outside
    // a culinary compound still requires an article ("injured shoulder").
    let meat = head.eq_any_ignore_ascii_case_str(&["shoulder", "breast"])
        && modifiers.iter().any(|t| {
            t.get_ch(source)
                .eq_any_ignore_ascii_case_str(&["pork", "duck", "chicken", "lamb", "beef", "veal"])
        });
    // Repeated-noun compounds act adverbially: "side-by-side", "eye-to-eye".
    let repeated = phrase.end >= phrase.start + 4
        && tokens[phrase.end - 1].kind.is_hyphen()
        && tokens[phrase.end - 2].kind.is_preposition()
        && tokens[phrase.end - 3].kind.is_hyphen()
        && tokens[phrase.end - 4].get_ch(source) == head;
    // A temporal suffix can follow an already complete nominal or participle:
    // "feedback post-launch", "evaluated pre-deployment". It is not the head.
    let temporal_suffix = phrase.end >= phrase.start + 4
        && tokens[phrase.end - 1].kind.is_hyphen()
        && tokens[phrase.end - 2]
            .get_ch(source)
            .eq_any_ignore_ascii_case_str(&["pre", "post"])
        && (tokens[phrase.end - 4].kind.is_mass_noun()
            || tokens[phrase.end - 4].kind.is_plural_noun()
            || tokens[phrase.end - 4].kind.is_verb_past_participle_form()
            || tokens[phrase.end - 4].kind.is_verb_past_form());
    named
        || relative_time
        || season
        || meat
        || repeated
        || temporal_suffix
        || is_template_slot(tokens, phrase, source)
}

/// Only bracketed insert directives are placeholders. Ordinary bracketed prose
/// is still checked, and an unbracketed imperative can require an article.
fn is_template_slot(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    let Some(open) = tokens[..phrase.start]
        .iter()
        .rposition(|t| t.get_ch(source) == ['['] || t.get_ch(source) == [']'])
    else {
        return false;
    };
    tokens[open].get_ch(source) == ['[']
        && tokens[open + 1..]
            .iter()
            .find(|t| !t.kind.is_whitespace())
            .is_some_and(|t| t.get_ch(source).eq_any_ignore_ascii_case_str(&["insert"]))
        && tokens[phrase.end + 1..]
            .iter()
            .any(|t| t.get_ch(source) == [']'])
}
