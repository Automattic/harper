use super::phrase::{ArticlePhrase, previous};
use crate::{CharStringExt, Token};
use harper_brill::UPOS;

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
        "quarter",
        "session",
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
        || is_name_or_title(tokens, phrase, source)
        || is_temporal(tokens, phrase, source)
        || is_bare_relation(tokens, phrase, source)
        || is_telegraphic(tokens, phrase, source)
        || is_quantified_of_phrase(tokens, phrase, source)
        || is_access_modifier(tokens, phrase, source)
        || is_numbered_label(tokens, phrase, source)
        // The tagger can read the noun "hit" as a finite verb, truncating
        // "cache hit rates" at "cache". Do not insert into that compound.
        || (tokens.get(phrase.end + 2).is_some_and(|t| t.get_ch(source).eq_any_ignore_ascii_case_str(&["hit"]))
            && tokens.get(phrase.end + 4).is_some_and(|t| t.get_ch(source).eq_any_ignore_ascii_case_str(&["rate", "rates", "ratio", "ratios"])))
        || has_shared_nominal_head(tokens, phrase)
        // An unrecognized continuation is not evidence that a noun phrase ends
        // here ("active volunteer firefighters"). Do not lint a truncated head.
        || tokens.get(phrase.end + 2).is_some_and(|t| {
            t.kind.is_oov()
                || (t.kind.is_word() && !t.kind.is_noun() && !t.kind.is_verb()
                    && t.get_ch(source).last() == Some(&'s'))
        })
        || relative_time
        || season
        || meat
        || repeated
        || temporal_suffix
        || (head.eq_any_ignore_ascii_case_str(&["term"])
            && phrase.end >= phrase.start + 4
            && tokens[phrase.end - 1].kind.is_hyphen()
            && tokens[phrase.end - 2].get_ch(source).eq_any_ignore_ascii_case_str(&["long", "short", "mid"]))
        || is_template_slot(tokens, phrase, source)
}

/// Recognizes dates and relative times without treating every duration as bare.
/// "An entire month" is nominal; "early next month" and "was tomorrow" are not.
fn is_temporal(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    let head = phrase.head.get_ch(source);
    let governor = previous(tokens, phrase).map(|t| t.get_ch(source));
    let single = phrase.first.span == phrase.head.span;
    ((single
        || phrase.first.kind.is_verb_past_form()
        || phrase.first.kind.is_verb_past_participle_form()
        || phrase.first.kind.is_verb_progressive_form()
        || governor.is_some_and(|g| {
            g.eq_any_ignore_ascii_case_str(&[
                "am", "is", "are", "was", "were", "be", "been", "being",
            ])
        }))
        && head.eq_any_ignore_ascii_case_str(&[
            "today",
            "tomorrow",
            "yesterday",
            "tonight",
            "midnight",
            "noon",
            "sunrise",
            "sunset",
        ]))
        || (single
            && head.eq_any_ignore_ascii_case_str(&["early", "late"])
            && tokens.get(phrase.end + 2).is_some_and(|t| {
                t.get_ch(source)
                    .eq_any_ignore_ascii_case_str(&["next", "last", "this"])
            }))
        || (head.eq_any_ignore_ascii_case_str(&["grade"])
            && phrase.first.get_ch(source).eq_any_ignore_ascii_case_str(&[
                "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth",
                "ninth", "tenth",
            ])
            && governor.is_some_and(|g| g.eq_any_ignore_ascii_case_str(&["in", "into", "include"])))
        || phrase
            .first
            .get_ch(source)
            .eq_any_ignore_ascii_case_str(&["next", "last", "this"])
        || (head.eq_any_ignore_ascii_case_str(&["season"])
            && phrase
                .first
                .get_ch(source)
                .eq_any_ignore_ascii_case_str(&["hunting", "storm"]))
}

/// Allows relational classifications and established bare prepositional idioms.
/// The same heads remain countable elsewhere: "need a new file type", "buy a swing".
fn is_bare_relation(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    let Some(governor) = previous(tokens, phrase) else {
        return false;
    };
    let head = phrase.head.get_ch(source);
    let first = phrase.first.get_ch(source);
    let g = governor.get_ch(source);
    let classification = head.eq_any_ignore_ascii_case_str(&[
        "type", "level", "role", "name", "size", "region", "method", "rate",
    ]) && g.eq_any_ignore_ascii_case_str(&[
        "by",
        "on",
        "of",
        "in",
        "with",
        "for",
        "to",
        "include",
        "including",
        "monitor",
        "measure",
        "indicate",
        "indicating",
        "consider",
        "considering",
    ]);
    let idiom = (g.eq_any_ignore_ascii_case_str(&["in"])
        && ((first.eq_any_ignore_ascii_case_str(&["equal"])
            && head.eq_any_ignore_ascii_case_str(&["measure"]))
            || (first.eq_any_ignore_ascii_case_str(&["full"])
                && head.eq_any_ignore_ascii_case_str(&["swing"]))
            || (first.eq_any_ignore_ascii_case_str(&["short", "medium", "long"])
                && head.eq_any_ignore_ascii_case_str(&["term"]))))
        || (g.eq_any_ignore_ascii_case_str(&["at"])
            && ((first.eq_any_ignore_ascii_case_str(&["first"])
                && head.eq_any_ignore_ascii_case_str(&["glance"]))
                || (first.eq_any_ignore_ascii_case_str(&["high", "low"])
                    && head.eq_any_ignore_ascii_case_str(&["tide"]))))
        || (g.eq_any_ignore_ascii_case_str(&["take", "takes", "took"])
            && first.eq_any_ignore_ascii_case_str(&["center", "centre"])
            && head.eq_any_ignore_ascii_case_str(&["stage"]))
        || (g.eq_any_ignore_ascii_case_str(&["on", "without", "with"])
            && first.eq_any_ignore_ascii_case_str(&["short", "prior", "adequate"])
            && head.eq_any_ignore_ascii_case_str(&["notice"]));
    let instrument = head.eq_any_ignore_ascii_case_str(&["guitar", "mandolin", "piano", "bass"])
        && g.eq_any_ignore_ascii_case_str(&["play", "plays", "played", "playing", "on", "with"]);
    let process = (head.eq_any_ignore_ascii_case_str(&["delay"])
        && g.eq_any_ignore_ascii_case_str(&["with", "without"]))
        || (head.eq_any_ignore_ascii_case_str(&["arrangement"])
            && first.eq_any_ignore_ascii_case_str(&["prior", "advance"]))
        || (head.eq_any_ignore_ascii_case_str(&["consequence"])
            && first.eq_any_ignore_ascii_case_str(&["minor", "little"])
            && g.eq_any_ignore_ascii_case_str(&["of"]))
        || (head.eq_any_ignore_ascii_case_str(&["term"])
            && first.eq_any_ignore_ascii_case_str(&["medium"])
            && g.eq_any_ignore_ascii_case_str(&["to"]));
    let compound = (head.eq_any_ignore_ascii_case_str(&["matrix"])
        && first.eq_any_ignore_ascii_case_str(&["soil"]))
        || (head.eq_any_ignore_ascii_case_str(&["game"])
            && first.eq_any_ignore_ascii_case_str(&["wild"]))
        || (head.eq_any_ignore_ascii_case_str(&["rule"])
            && first.eq_any_ignore_ascii_case_str(&["imperial", "colonial"]))
        || (head.eq_any_ignore_ascii_case_str(&["leaf"])
            && first.eq_any_ignore_ascii_case_str(&["loose"]))
        || (head.eq_any_ignore_ascii_case_str(&["effect", "island"])
            && tokens[phrase.start..phrase.end]
                .iter()
                .any(|t| t.get_ch(source).eq_any_ignore_ascii_case_str(&["heat"])))
        || (head.eq_any_ignore_ascii_case_str(&["toe"])
            && first.eq_any_ignore_ascii_case_str(&["head"])
            && phrase.end == phrase.start + 4)
        || (first.eq_any_ignore_ascii_case_str(&["mid"])
            && phrase.end == phrase.start + 2
            && tokens[phrase.start + 1].kind.is_hyphen());
    let metric = first.eq_any_ignore_ascii_case_str(&["time"])
        && phrase.end == phrase.start + 4
        && tokens[phrase.start + 1].kind.is_hyphen()
        && tokens[phrase.start + 2]
            .get_ch(source)
            .eq_any_ignore_ascii_case_str(&["to"])
        && tokens[phrase.start + 3].kind.is_hyphen();
    classification || idiom || instrument || process || compound || metric
}

/// Names and title-name appositions permit bare roles. Require actual name
/// evidence, rather than globally excluding role nouns such as "consultant".
fn is_name_or_title(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    let capitalized = |t: &Token| t.get_ch(source).first().is_some_and(|c| c.is_uppercase());
    let named_prefix = tokens[phrase.start..=phrase.end]
        .iter()
        .filter(|t| t.kind.is_word())
        .take(2)
        .filter(|t| capitalized(t))
        .count()
        == 2;
    let following = phrase.end + 1;
    let name_start = if tokens.get(following).is_some_and(|t| t.kind.is_comma()) {
        following + 2
    } else {
        following + 1
    };
    let name_start = if tokens.get(name_start).is_some_and(|t| t.kind.is_quote()) {
        name_start + 1
    } else {
        name_start
    };
    let role = phrase.head.get_ch(source).eq_any_ignore_ascii_case_str(&[
        "consultant",
        "expert",
        "engineer",
        "artist",
        "musician",
        "architect",
        "therapist",
        "firm",
        "company",
        "band",
        "newcomer",
        "woodworker",
        "expressionist",
        "ceramicist",
        "critic",
        "chef",
        "competitor",
        "developer",
        "supplier",
        "contractor",
        "author",
        "teacher",
        "archivist",
        "designer",
        "baron",
        "anchor",
        "enthusiast",
        "businesswoman",
    ]);
    let title_name = role
        && tokens.get(name_start).is_some_and(|t| {
            capitalized(t)
                && (t.kind.is_upos(UPOS::PROPN)
                    || t.kind.is_oov()
                    || t.get_ch(source).eq_any_ignore_ascii_case_str(&[
                        "mr", "mrs", "ms", "dr", "prof", "mr.", "mrs.", "ms.", "dr.", "prof.",
                    ])
                    || tokens.get(name_start + 2).is_some_and(capitalized))
        });
    let preceded_by_name = phrase.start.checked_sub(4).is_some_and(|i| {
        tokens[i].kind.is_noun()
            && capitalized(&tokens[i])
            && tokens[i + 2].kind.is_noun()
            && capitalized(&tokens[i + 2])
    });
    let software_name = phrase
        .first
        .get_ch(source)
        .eq_any_ignore_ascii_case_str(&["vim"])
        && phrase
            .head
            .get_ch(source)
            .eq_any_ignore_ascii_case_str(&["plug"])
        && phrase.end == phrase.start + 2
        && tokens[phrase.start + 1].kind.is_hyphen();
    named_prefix || title_name || preceded_by_name || software_name
}

/// An `of` complement after a quantifier or classifier does not independently
/// require an indefinite article: "95% of users report", "a kind of clear toy".
fn is_quantified_of_phrase(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    if !previous(tokens, phrase)
        .is_some_and(|t| t.get_ch(source).eq_any_ignore_ascii_case_str(&["of"]))
    {
        return false;
    }
    phrase
        .start
        .checked_sub(4)
        .and_then(|i| tokens.get(i))
        .is_some_and(|t| {
            t.kind.is_number()
                || t.get_ch(source).eq_any_ignore_ascii_case_str(&[
                    "%",
                    "percent",
                    "percentage",
                    "number",
                    "amount",
                    "kind",
                    "type",
                    "sort",
                    "range",
                    "variety",
                    "collection",
                    "set",
                    "combination",
                    "form",
                ])
        })
}

/// Coordinated role names can share a bare access/permission head:
/// "administrator or editor access". Other alternatives remain independent
/// noun phrases, so "an administrator or a consultant" is still checked.
fn is_access_modifier(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    if phrase.first.span != phrase.head.span
        || !tokens.get(phrase.end + 2).is_some_and(|t| {
            t.get_ch(source)
                .eq_any_ignore_ascii_case_str(&["or", "and"])
        })
    {
        return false;
    }
    super::phrase::phrase(tokens, phrase.end + 4).is_some_and(|tail| {
        tail.first.span != tail.head.span
            && tail.head.get_ch(source).eq_any_ignore_ascii_case_str(&[
                "access",
                "privileges",
                "permissions",
                "rights",
                "credentials",
            ])
    })
}

/// Postposed section or cycle identifiers act as determiners, including Roman
/// numerals. An ordinary unnumbered "diagnostic cycle" still requires an article.
fn is_numbered_label(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    if phrase
        .head
        .get_ch(source)
        .eq_any_ignore_ascii_case_str(&["percent"])
        && phrase.first.get_ch(source).eq_any_ignore_ascii_case_str(&[
            "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
            "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
            "hundred",
        ])
    {
        return true;
    }
    let roman = |text: &[char]| {
        !text.is_empty()
            && text
                .iter()
                .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
    };
    if phrase
        .first
        .get_ch(source)
        .eq_any_ignore_ascii_case_str(&["section", "chapter", "item", "appendix"])
        && roman(phrase.head.get_ch(source))
    {
        return true;
    }
    let Some(next) = tokens.get(phrase.end + 2) else {
        return false;
    };
    next.kind.is_number()
        || (phrase
            .head
            .get_ch(source)
            .eq_any_ignore_ascii_case_str(&["section", "chapter", "item", "appendix"])
            && roman(next.get_ch(source)))
}

/// Coordinated noun modifiers can share a final plural head: "bus and train
/// routes", "short answer and essay questions". Require a second noun modifier,
/// not an independent "short answer and a diagram" or "old bus and new trains".
fn has_shared_nominal_head(tokens: &[Token], phrase: &ArticlePhrase<'_>) -> bool {
    let Some(conjunction) = tokens.get(phrase.end + 2) else {
        return false;
    };
    if !conjunction.kind.is_upos(UPOS::CCONJ) {
        return false;
    }
    super::phrase::phrase(tokens, phrase.end + 4).is_some_and(|tail| {
        !tail.determined
            && tail.first.span != tail.head.span
            && tail.first.kind.is_upos(UPOS::NOUN)
            && tail.head.kind.is_plural_noun()
    })
}

/// Skips Markdown headings, table cells, and verbless label fragments, not
/// complete example sentences after labels. Raw source is needed because the
/// Markdown parser removes some layout punctuation before phrase scanning.
fn is_telegraphic(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    let start = phrase.first.span.start;
    let line_start = source[..start]
        .iter()
        .rposition(|c| *c == '\n')
        .map_or(0, |i| i + 1);
    let line_end = source[start..]
        .iter()
        .position(|c| *c == '\n')
        .map_or(source.len(), |i| start + i);
    let line: String = source[line_start..line_end].iter().collect();
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') || trimmed.starts_with('|') {
        return true;
    }
    let prefix = &source[line_start..start];
    let list_label = trimmed.split_once(':').is_some_and(|(label, _)| {
        let label = label.trim_matches(|c: char| c.is_whitespace() || matches!(c, '*' | '-'));
        label.eq_ignore_ascii_case("materials")
            || label.eq_ignore_ascii_case("deliverables include")
    });
    let stage_direction = (trimmed.starts_with("**(") || trimmed.starts_with("*("))
        && !prefix.contains(&')')
        && !tokens[..phrase.start]
            .iter()
            .any(|t| t.kind.is_upos(UPOS::PRON) || t.kind.is_upos(UPOS::AUX));
    if list_label || stage_direction {
        return true;
    }
    let label = prefix
        .iter()
        .rposition(|c| *c == ':')
        .map(|i| line_start + i);
    label.is_some_and(|colon| {
        !tokens[..phrase.start]
            .iter()
            .filter(|t| t.span.start > colon)
            .any(|t| {
                t.kind.is_upos(UPOS::PRON)
                    || t.kind.is_upos(UPOS::AUX)
                    || (t.kind.is_upos(UPOS::VERB)
                        && !t.kind.is_verb_past_participle_form()
                        && !t.kind.is_verb_progressive_form()
                        && !t.kind.is_plural_noun())
            })
    })
}

/// Bracketed insert/describe directives and verbless template slots are not
/// ordinary prose. Complete bracketed sentences and bare imperatives remain checked.
fn is_template_slot(tokens: &[Token], phrase: &ArticlePhrase<'_>, source: &[char]) -> bool {
    // Template directives can contain sentence punctuation ("e.g.") which
    // places the opening bracket outside this sentence's token slice.
    if let Some(open) = source[..phrase.first.span.start]
        .iter()
        .rposition(|c| matches!(c, '[' | ']'))
        && source[open] == '['
        && source[phrase.head.span.end..].contains(&']')
    {
        let prefix: String = source[open + 1..phrase.first.span.start].iter().collect();
        let words: Vec<_> = prefix.split_whitespace().take(2).collect();
        if words.iter().any(|w| {
            ["insert", "describe", "mention", "outline", "link"]
                .iter()
                .any(|directive| w.eq_ignore_ascii_case(directive))
        }) {
            return true;
        }
    }
    let Some(open) = tokens[..phrase.start]
        .iter()
        .rposition(|t| t.get_ch(source) == ['['] || t.get_ch(source) == [']'])
    else {
        return false;
    };
    if tokens[open].get_ch(source) != ['['] {
        return false;
    }
    let Some(close) = tokens[phrase.end + 1..]
        .iter()
        .position(|t| t.get_ch(source) == [']'])
    else {
        return false;
    };
    let slot = &tokens[open + 1..phrase.end + 1 + close];
    let instruction = slot.iter().filter(|t| t.kind.is_word()).take(2).any(|t| {
        t.get_ch(source)
            .eq_any_ignore_ascii_case_str(&["insert", "describe", "mention", "outline", "link"])
    });
    instruction
        || !slot.iter().any(|t| {
            t.kind.is_upos(UPOS::PRON) || t.kind.is_upos(UPOS::VERB) || t.kind.is_upos(UPOS::AUX)
        })
}
