use harper_brill::UPOS;

use crate::{
    CharStringExt, Dialect, Document, Span, Token, TokenStringExt,
    indefinite_article::{InitialSound, starts_with_vowel},
    linting::{Lint, LintKind, Suggestion},
};

/// A complete nominal phrase, including an optional determiner or possessive.
/// Indices are inclusive. `first` excludes the determiner and opening quote and
/// supplies the pronunciation used when choosing an article.
pub(super) struct ArticlePhrase<'a> {
    pub first: &'a Token,
    pub head: &'a Token,
    pub start: usize,
    pub end: usize,
    pub determined: bool,
}

/// Returns the governor immediately before a space-separated phrase.
/// Punctuation is not treated as an implicit grammatical connection.
pub(super) fn previous<'a>(tokens: &'a [Token], phrase: &ArticlePhrase<'_>) -> Option<&'a Token> {
    let index = phrase.start.checked_sub(2)?;
    tokens
        .get(phrase.start - 1)
        .filter(|t| t.kind.is_whitespace())?;
    tokens.get(index)
}

/// Runs a context-specific check over complete noun phrases. Determiners and
/// gerund complements protect their interiors even when no lint is emitted.
/// The callback owns grammatical-context decisions, not phrase scanning.
pub(super) fn lint_phrases(
    document: &Document,
    dialect: Dialect,
    accepts: impl Fn(&[Token], &ArticlePhrase<'_>, &[char]) -> bool,
) -> Vec<Lint> {
    let source = document.get_source();
    let mut lints = Vec::new();
    for sentence in document.iter_sentences() {
        let start = sentence
            .iter()
            .position(|t| !t.kind.is_whitespace())
            .unwrap_or(0);
        let tokens = &sentence[start..];
        let mut covered_until = 0;
        for (index, token) in tokens.iter().enumerate() {
            if index < covered_until || token.kind.is_whitespace() {
                continue;
            }
            let Some(phrase) = phrase(tokens, index) else {
                continue;
            };
            if phrase.determined
                || is_gerund_complement(tokens, &phrase, source)
                || super::bare_uses::allows_bare(tokens, &phrase, source)
            {
                covered_until = phrase.end + 1;
                continue;
            }
            if !is_head(
                phrase.head,
                source,
                token.kind.is_quote(),
                has_count_sense(tokens, &phrase, source),
                phrase.head.kind.is_upos(UPOS::VERB)
                    && !phrase.head.kind.is_adjective()
                    && is_nominal_fallback(tokens, phrase.start, phrase.end),
            ) || phrase.first.kind.is_quantifier()
                || phrase
                    .first
                    .get_ch(source)
                    .eq_any_ignore_ascii_case_str(&["such", "right"])
                || tokens
                    .get(phrase.end + 2)
                    .is_some_and(|t| t.get_ch(source).eq_any_ignore_ascii_case_str(&["enough"]))
                || (tokens
                    .get(phrase.end + 2)
                    .is_some_and(|t| t.kind.is_preposition())
                    && tokens
                        .get(phrase.end + 4)
                        .is_some_and(|t| t.get_ch(source) == phrase.head.get_ch(source)))
            {
                continue;
            }
            if accepts(tokens, &phrase, source)
                && let Some(lint) = suggest_article(tokens, &phrase, source, dialect)
            {
                covered_until = phrase.end + 1;
                lints.push(lint);
            }
        }
    }
    lints
}

/// Builds a dialect-aware insertion, preserving quotes and source spacing.
/// An initial subject instead needs a replacement to transfer capitalization.
pub(super) fn suggest_article(
    tokens: &[Token],
    phrase: &ArticlePhrase<'_>,
    source: &[char],
    dialect: Dialect,
) -> Option<Lint> {
    let first = phrase.first;
    if phrase.start != 0
        && first
            .get_ch(source)
            .first()
            .is_some_and(|c| c.is_uppercase())
        && !first.kind.is_upos(UPOS::ADJ)
    {
        return None;
    }
    let articles: &[&str] = match starts_with_vowel(first.get_ch(source), dialect)? {
        InitialSound::Vowel => &["an "],
        InitialSound::Consonant => &["a "],
        InitialSound::Either => &["a ", "an "],
    };
    Some(Lint {
        span: if phrase.start == 0 {
            first.span
        } else {
            Span::empty(tokens[phrase.start].span.start)
        },
        lint_kind: LintKind::Grammar,
        suggestions: articles
            .iter()
            .map(|article| {
                if phrase.start == 0 {
                    let mut replacement: Vec<char> = article.chars().collect();
                    replacement[0] = replacement[0].to_ascii_uppercase();
                    let mut word = first.get_ch(source).to_vec();
                    if first.kind.is_lowercase() {
                        word[0] = word[0].to_ascii_lowercase();
                    }
                    replacement.extend(word);
                    Suggestion::ReplaceWith(replacement)
                } else {
                    Suggestion::InsertAfter(article.chars().collect())
                }
            })
            .collect(),
        message: "This singular count noun may need `a` or `an`.".to_owned(),
        priority: 30,
    })
}

/// Checks noun metadata and established count senses without mistaking participles for heads.
fn is_head(
    token: &Token,
    source: &[char],
    quoted: bool,
    count_sense: bool,
    inferred: bool,
) -> bool {
    (token.kind.is_upos(UPOS::NOUN) || ((quoted || inferred) && token.kind.is_noun()))
        && token.kind.is_singular_noun()
        && !token.kind.is_plural_noun()
        && token.kind.is_countable_noun()
        && !token.kind.is_comparative_adjective()
        // Mixed count/mass nouns ("work", "water") also permit bare uses.
        && (!token.kind.is_mass_noun() || count_sense)
        && (!token.kind.is_proper_noun() || (quoted && token.get_ch(source).iter().all(|c| c.is_lowercase())))
        && !token.kind.is_possessive_noun()
        // Tagging without an article can misread a predicate participle as
        // a noun ("is likely intended", "has finished").
        && !token.kind.is_verb_past_form()
        && !token.kind.is_verb_past_participle_form()
        && !token.kind.is_verb_progressive_form()
        // Roles and certain idioms permit a bare singular noun: "became
        // president", "is home", "had lunch", "have trouble".
        && !token.get_ch(source).eq_any_ignore_ascii_case_str(&[
            "president", "chair", "chairman", "captain", "king", "queen", "head",
            "chief", "mayor", "director", "manager", "treasurer", "secretary", "owner",
            "home", "breakfast", "lunch", "dinner", "supper", "luncheon", "trouble", "part",
            "judge", "jury", "nephew", "niece", "mother", "father", "brother", "sister",
            "son", "daughter", "cousin",
            // These words have bare or predicate uses that are not
            // reliably distinguished by the dictionary and POS tagger.
            "plenty", "practice", "silence", "property", "alert", "operative", "subject",
            "kind", "permission", "flu", "rest", "courage", "exchange", "mode",
        ])
}

/// Reads a complete nominal phrase, preserving determiner and possessive boundaries.
pub(super) fn phrase<'a>(tokens: &'a [Token], start: usize) -> Option<ArticlePhrase<'a>> {
    let mut cursor = start;
    let determined =
        tokens.get(cursor)?.kind.is_determiner() || tokens[cursor].kind.is_possessive_nominal();
    if determined {
        if !tokens.get(cursor + 1)?.kind.is_whitespace() {
            return None;
        }
        cursor += 2;
    }
    if tokens.get(cursor)?.kind.is_quote() {
        cursor += 1;
    }
    let first = tokens.get(cursor)?;
    loop {
        let token = tokens.get(cursor)?;
        if token.kind.is_upos(UPOS::ADV)
            && !tokens.get(cursor + 2).is_some_and(|t| {
                t.kind.is_upos(UPOS::ADJ)
                    || (determined
                        && (t.kind.is_adjective() || t.kind.is_verb_past_participle_form()))
            })
        {
            return None;
        }
        let nominal = token.kind.is_upos(UPOS::NOUN)
            || (token.kind.is_upos(UPOS::PROPN) && token.kind.is_noun())
            || token.kind.is_upos(UPOS::ADJ)
            || token.kind.is_upos(UPOS::ADV)
            || token.kind.is_verb_past_participle_form()
            || (token.kind.is_verb_past_form() && token.kind.is_adjective())
            || token.kind.is_verb_progressive_form()
            || is_nominal_fallback(tokens, start, cursor);
        if !nominal
            || token.kind.is_determiner()
            || token.kind.is_possessive_noun()
            || token.kind.is_comparative_adjective()
            || (!determined
                && token.kind.is_proper_noun()
                && !token.kind.is_upos(UPOS::ADJ)
                && !is_hyphen_modifier(tokens, cursor))
        {
            return None;
        }
        // Commas may separate modifiers within one determined phrase. Do not
        // treat a noun list ("a dog, cat") as a shared determiner construction.
        if determined
            && (token.kind.is_upos(UPOS::ADJ) || token.kind.is_verb_past_participle_form())
            && tokens.get(cursor + 1).is_some_and(|t| t.kind.is_comma())
            && tokens
                .get(cursor + 2)
                .is_some_and(|t| t.kind.is_whitespace())
            && tokens
                .get(cursor + 3)
                .is_some_and(|t| t.kind.is_adjective() || t.kind.is_verb_past_participle_form())
        {
            cursor += 3;
            continue;
        }
        let next = cursor + 2;
        let continues = tokens
            .get(cursor + 1)
            .is_some_and(|t| t.kind.is_whitespace() || t.kind.is_hyphen())
            && tokens.get(next).is_some_and(|t| {
                t.kind.is_upos(UPOS::NOUN)
                    || (is_nominal_fallback(tokens, start, next)
                        && !(t.kind.is_preposition()
                            && tokens[cursor + 1].kind.is_whitespace()
                            && !tokens
                                .get(next + 4)
                                .is_some_and(|after| after.kind.is_upos(UPOS::NOUN))))
                    || t.kind.is_upos(UPOS::ADJ)
                    || t.kind.is_upos(UPOS::PROPN)
                    || (token.kind.is_upos(UPOS::ADV)
                        && (t.kind.is_adjective() || t.kind.is_verb_past_participle_form()))
            });
        if !continues {
            // Do not truncate a hyphenated modifier at an uncertain token.
            if tokens.get(cursor + 1).is_some_and(|t| t.kind.is_hyphen()) {
                return None;
            }
            return Some(ArticlePhrase {
                first,
                head: token,
                start,
                end: cursor,
                determined,
            });
        }
        cursor = next;
    }
}

/// A preposition can be part of a hyphenated adjective ("built-in", "on-call")
/// without turning a following noun into a separate prepositional object.
fn is_hyphen_modifier(tokens: &[Token], index: usize) -> bool {
    (tokens[index].kind.is_adjective() || tokens[index].kind.is_preposition())
        && (index
            .checked_sub(1)
            .and_then(|i| tokens.get(i))
            .is_some_and(|t| t.kind.is_hyphen())
            || tokens.get(index + 1).is_some_and(|t| t.kind.is_hyphen()))
}

/// Recovers dictionary-backed compound members missed by the POS tagger.
/// Verb-tagged words require a preceding noun in a governed object, not a
/// sentence subject. Base verbs are only recovered at the end of a sentence;
/// this distinguishes "need latency monitor" from "latency affects users".
fn is_nominal_fallback(tokens: &[Token], start: usize, index: usize) -> bool {
    let Some(token) = tokens.get(index) else {
        return false;
    };
    if is_hyphen_modifier(tokens, index)
        || (token.kind.is_upos(UPOS::VERB) && token.kind.is_adjective() && !token.kind.is_verb())
    {
        return true;
    }
    if token
        .kind
        .as_word()
        .and_then(|m| m.as_ref())
        .is_some_and(|m| m.pos_tag.is_none())
    {
        return index > start && (token.kind.is_noun() || token.kind.is_adjective());
    }
    if !token.kind.is_upos(UPOS::VERB)
        || !token.kind.is_noun()
        || !token.kind.is_verb()
        || token.kind.is_auxiliary_verb()
        || token.kind.is_preposition()
        || index <= start
    {
        return false;
    }
    let governed = start
        .checked_sub(2)
        .and_then(|i| tokens.get(i))
        .is_some_and(|t| {
            t.kind.is_upos(UPOS::VERB) || t.kind.is_upos(UPOS::AUX) || t.kind.is_upos(UPOS::ADP)
        });
    let previous_noun = index
        .checked_sub(2)
        .and_then(|i| tokens.get(i))
        .is_some_and(|t| t.kind.is_singular_noun() && !t.kind.is_plural_noun());
    let terminal = tokens
        .get(index + 1)
        .is_none_or(|t| t.kind.is_sentence_terminator());
    let nominal_modifier = index
        .checked_sub(2)
        .and_then(|i| tokens.get(i))
        .is_some_and(|t| !t.kind.is_verb_progressive_form() && !t.kind.is_mass_noun());
    governed
        && previous_noun
        && ((nominal_modifier && token.kind.is_plural_noun())
            || token.kind.is_verb_progressive_form()
            || (terminal && token.kind.is_verb_lemma()))
}

/// Establishes selected count senses from the governor and modifiers, not countability defaults alone.
fn has_count_sense(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    let Some(previous) = phrase.start.checked_sub(2).and_then(|i| tokens.get(i)) else {
        return false;
    };
    let head = phrase.head.get_ch(source);
    let modifier = phrase.first.get_ch(source);
    let governor = previous.get_ch(source);
    if super::count_senses::selected(head, governor, modifier) {
        return true;
    }
    let nominal_predicate = governor
        .eq_any_ignore_ascii_case_str(&["am", "is", "are", "was", "were", "be", "been", "being"])
        && phrase.first.kind.is_upos(UPOS::ADJ);
    // A copular "waste of ..." names an instance, unlike material waste.
    if head.eq_any_ignore_ascii_case_str(&["waste"])
        && governor.eq_any_ignore_ascii_case_str(&[
            "am", "is", "are", "was", "were", "be", "been", "being",
        ])
        && tokens
            .get(phrase.end + 2)
            .is_some_and(|t| t.get_ch(source).eq_any_ignore_ascii_case_str(&["of"]))
    {
        return true;
    }
    if phrase.first.span == phrase.head.span {
        return false;
    }
    // Event-selecting verbs distinguish an individual activity or ceremony
    // from aggregate activity and the provision of a public service.
    if head.eq_any_ignore_ascii_case_str(&["activity", "service"]) {
        return governor.eq_any_ignore_ascii_case_str(&[
            "plan",
            "plans",
            "planned",
            "schedule",
            "schedules",
            "scheduled",
            "organize",
            "organizes",
            "organized",
            "organise",
            "organises",
            "organised",
            "attend",
            "attends",
            "attended",
        ]);
    }
    if head.eq_any_ignore_ascii_case_str(&["room", "place"]) {
        return phrase.first.kind.is_verb_progressive_form()
            || governor.eq_any_ignore_ascii_case_str(&["in", "inside", "within"]);
    }
    if head.eq_any_ignore_ascii_case_str(&["creation"]) {
        return nominal_predicate
            || governor.eq_any_ignore_ascii_case_str(&[
                "buy",
                "buys",
                "bought",
                "display",
                "displays",
                "displayed",
            ]);
    }
    if head.eq_any_ignore_ascii_case_str(&["engagement"]) {
        return governor.eq_any_ignore_ascii_case_str(&["attend", "attends", "attended"])
            || governor.eq_any_ignore_ascii_case_str(&["have", "has", "had"])
                && modifier.eq_any_ignore_ascii_case_str(&[
                    "prior",
                    "previous",
                    "scheduled",
                    "upcoming",
                ]);
    }
    head.eq_any_ignore_ascii_case_str(&["definition"])
        && phrase.first.kind.is_upos(UPOS::ADJ)
        && (nominal_predicate
            || governor.eq_any_ignore_ascii_case_str(&[
                "need", "needs", "needed", "want", "wants", "wanted", "provide", "provides",
                "provided",
            ]))
}

/// Distinguishes gerund complements from nominal compounds using their governor and phrase membership.
fn is_gerund_complement(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    if !phrase.first.kind.is_verb_progressive_form() || phrase.first.span == phrase.head.span {
        return false;
    }
    let Some(previous) = phrase.start.checked_sub(2).and_then(|i| tokens.get(i)) else {
        return false;
    };
    if previous.get_ch(source).eq_any_ignore_ascii_case_str(&[
        "consider",
        "considers",
        "considered",
        "considering",
        "enjoy",
        "enjoys",
        "enjoyed",
        "enjoying",
        "avoid",
        "avoids",
        "avoided",
        "avoiding",
        "finish",
        "finishes",
        "finished",
        "finishing",
    ]) {
        return true;
    }
    previous.kind.is_preposition()
        && (!phrase.first.kind.is_noun() || !phrase.head.kind.is_np_member())
}
