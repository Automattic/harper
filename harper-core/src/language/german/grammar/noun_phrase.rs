//! Chunking a German sentence into noun phrases.
//!
//! German noun phrases are rigid — a determiner or preposition, then any number
//! of attributive adjectives, then the head noun — which is why a rule can do
//! here what English needs a trained chunker for. Three linters want the same
//! answer from it and each used to guess differently:
//!
//! * **capitalization** asks which token is the head, because only a head may
//!   be a miscapitalized noun;
//! * **preposition case** asks what noun a determiner governs, to read its
//!   gender and number;
//! * **subject–verb agreement** asks where a noun-phrase subject ends, because
//!   the finite verb stands directly behind it.
//!
//! Each private answer was wrong in its own way. The capitalization rule grew
//! the only version that survived contact with a corpus, and the other two paid
//! for their own: the preposition rule's head-finder crossed a clause boundary
//! and produced thirty-seven false reports the moment gender narrowing was
//! switched on, and the agreement rule's produced 1229.
//!
//! [`phrases`] is the shared answer. [`roles`] is the same information keyed by
//! token, which is what the capitalization rule reads.
//!
//! # What the shape buys
//!
//! ```text
//! die   wesentliche   Frage      der   große     schöne     hund
//! ^open ^modifier     ^head      ^open ^modifier ^modifier  ^head
//! ```
//!
//! The head is always **last**: everything in front of it is an attributive
//! adjective, which German writes lower case. So a capitalized token ends the
//! phrase — it *is* the head. Without that, *in Munitionsfabriken eingesetzt*
//! runs on past `Munitionsfabriken` and makes the participle the head.
//!
//! Everything here reads only the metadata `Document::parse` already attached
//! to each token. Calling `Dictionary::get_word_metadata` per token would take
//! `CompoundAwareDictionary`'s global mutex and attempt a compound
//! decomposition on every miss.

use crate::language::german::spell::lexical_classes::{NUMERALS, UNIT_ABBREVIATIONS};
use crate::{Document, Punctuation, Token, TokenKind};

/// A token's position in a German noun phrase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The head of the phrase — the noun (*die wesentliche **Frage***).
    Head,
    /// An attributive modifier before the head (*die **wesentliche** Frage*).
    Modifier,
    /// Not inside a noun phrase at all.
    Outside,
}

/// One chunked noun phrase, as indices into the token slice it was found in.
///
/// `open` is the determiner or preposition, `head` the noun, and `end` is
/// exclusive. `open + 1 == head` is the bare *die Frage* case; anything wider
/// holds attributive material.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phrase {
    pub open: usize,
    pub head: usize,
    pub end: usize,
}

/// Chunk a sentence into noun phrases.
///
/// `tokens` must have its whitespace filtered out; the indices in the result
/// are into that slice. Phrases are returned in order and never overlap.
pub fn phrases(tokens: &[&Token], document: &Document) -> Vec<Phrase> {
    chunk(tokens, document)
}

/// The phrase whose **head** is the token at `index`, if there is one.
pub fn phrase_headed_by(phrases: &[Phrase], index: usize) -> Option<&Phrase> {
    phrases.iter().find(|phrase| phrase.head == index)
}

/// The phrase **opened** by the token at `index`, if there is one.
pub fn phrase_opened_by(phrases: &[Phrase], index: usize) -> Option<&Phrase> {
    phrases.iter().find(|phrase| phrase.open == index)
}

/// Label every token with its role, for callers that ask per token rather than
/// per phrase.
pub fn roles(tokens: &[&Token], document: &Document) -> Vec<Role> {
    let mut roles = vec![Role::Outside; tokens.len()];
    for phrase in chunk(tokens, document) {
        for role in roles.iter_mut().take(phrase.head).skip(phrase.open + 1) {
            *role = Role::Modifier;
        }
        roles[phrase.head] = Role::Head;
    }
    roles
}

/// Words whose lower-case reading is not the noun one.
///
/// This used to be 265 words, and the reason given was that "the dictionary
/// actively mistags them". That is no longer true:
/// `harper-core/src/language/german/scripts/strip_german_noun_readings.py` took the noun reading off every
/// lower-case entry igerman98 has no capitalized form for, and 230 of these
/// words stopped reading as nouns with it.
///
/// What is left is the part a dictionary cannot settle. Each of these really is
/// a noun when it is capitalized -- `die Frage`, `die Waren`, `das Gut`, `die
/// Wegen` -- so the entry is right to carry the reading, and the lower-case
/// occurrence still has to be let through. A handful are not entries at all.
///
/// Check the list against the dictionary before adding to it:
///
/// ```bash
/// just language-meta-text german "wegen trotz ist waren"
/// ```
pub(crate) const GERMAN_NON_NOUNS: &[&str] = &[
    "alt", "arbeite", "darin", "denke", "dgl", "dürfen", "ebd", "etc", "frage", "gebe", "gibe",
    "groß", "gut", "habe", "heute", "hin", "ist", "klein", "kurz", "können", "lang", "langsam",
    "neu", "sehe", "sollen", "sondern", "teils", "trotz", "versuche", "viel", "waren", "wegen",
    "wollen", "worden", "wäre",
];

/// Words that, standing immediately to the left of a candidate, mark it as the
/// head or a modifier of a noun phrase: articles, other determiners,
/// possessives, demonstratives, quantifiers and the common prepositions
/// (including the usual contracted forms). Kept as an explicit surface list
/// because the German dictionary mislabels many of these forms.
pub(crate) const NOUN_PHRASE_LICENSORS: &[&str] = &[
    // definite / indefinite articles, all cases
    "der",
    "die",
    "das",
    "dem",
    "den",
    "des",
    "ein",
    "eine",
    "einen",
    "einem",
    "einer",
    "eines",
    "kein",
    "keine",
    "keinen",
    "keinem",
    "keiner",
    "keines",
    // possessives
    "mein",
    "meine",
    "meinen",
    "meinem",
    "meiner",
    "meines",
    "dein",
    "deine",
    "deinen",
    "deinem",
    "deiner",
    "deines",
    "sein",
    "seine",
    "seinen",
    "seinem",
    "seiner",
    "seines",
    "ihr",
    "ihre",
    "ihren",
    "ihrem",
    "ihrer",
    "ihres",
    "unser",
    "unsere",
    "unseren",
    "unserem",
    "unserer",
    "unseres",
    "euer",
    "eure",
    "euren",
    "eurem",
    "eurer",
    "eures",
    // demonstratives / relatives / quantifiers
    "dies",
    "dieser",
    "diese",
    "dieses",
    "diesen",
    "diesem",
    "jen",
    "jener",
    "jene",
    "jenes",
    "jenen",
    "jenem",
    "jed",
    "jeder",
    "jede",
    "jedes",
    "jeden",
    "jedem",
    "manch",
    "mancher",
    "manche",
    "manches",
    "manchen",
    "manchem",
    "solch",
    "solcher",
    "solche",
    "solches",
    "solchen",
    "solchem",
    "welch",
    "welcher",
    "welche",
    "welches",
    "welchen",
    "welchem",
    "all",
    "alle",
    "allen",
    "aller",
    "alles",
    "allem",
    "beide",
    "beiden",
    "beider",
    "sämtliche",
    "sämtlichen",
    "jegliche",
    "jeglichen",
    "mehrere",
    "mehreren",
    "einige",
    "einigen",
    "einiger",
    "viele",
    "vielen",
    "wenige",
    "wenigen",
    // prepositions fused with an article: these *are* a determiner ("im
    // Freien", "zum Guten"), which is why they are on this side of the split.
    "im",
    "ins",
    "am",
    "ans",
    "aufs",
    "beim",
    "vom",
    "zum",
    "zur",
    "übers",
    "unters",
    "durchs",
    "fürs",
    "ums",
];

/// Prepositions that open a noun phrase without supplying a determiner.
///
/// German nominalizes an adjective only under a determiner, and the nominalized
/// form always carries a declension ending: *das Braune*, *im Freien*, *ein
/// Kahler*. A bare preposition in front of a **base-form** adjective is
/// therefore never a nominalization — *"weiß bis braun"*, *"von gelb zu weiß"*
/// are predicative. Kept apart from [`NOUN_PHRASE_LICENSORS`] for exactly that
/// distinction; both open a phrase.
pub(crate) const NP_BARE_PREPOSITIONS: &[&str] = &[
    "in",
    "an",
    "auf",
    "aus",
    "bei",
    "mit",
    "nach",
    "von",
    "vor",
    "zu",
    "über",
    "unter",
    "durch",
    "für",
    "gegen",
    "ohne",
    "um",
    "seit",
    "während",
    "wegen",
    "trotz",
    "statt",
    "anstatt",
    "innerhalb",
    "außerhalb",
    "oberhalb",
    "unterhalb",
    "entlang",
    "gegenüber",
    "bis",
    "per",
    "pro",
    "via",
    "samt",
    "nebst",
    "laut",
    "gemäß",
    "mittels",
    "anhand",
    "aufgrund",
    "infolge",
    "zwischen",
    "neben",
    "hinter",
];

/// Words that join coordinated attributive adjectives inside one noun phrase:
/// *"in britische, französische **und** niederländische Kolonien"*.
/// Function words of the languages German quotes without translating, chosen so
/// that **none of them is also a German word**. `des`, `in`, `da`, `so` and `e`
/// are deliberately absent for that reason, even though they are frequent in
/// French, Italian and Latin.
/// Forms that are a relative pronoun as readily as an article or determiner.
/// Only [`GermanNounCapitalization::opens_relative_clause`] uses this, and only
/// straight after a comma or an opening bracket.
pub(crate) const RELATIVE_PRONOUNS: &[&str] = &[
    "der", "die", "das", "dem", "den", "dessen", "deren", "denen", "welcher", "welche", "welches",
    "welchen", "welchem", "wer", "wen", "wem", "was",
];

pub(crate) const COORDINATORS: &[&str] = &[
    "und",
    "oder",
    "sowie",
    "beziehungsweise",
    "bzw",
    // Contrastive: "der milde, **aber** wenig angenehme Pilz". They join two
    // attributive adjectives exactly the way "und" does, and stopping on one
    // promoted the adjective in front of it to head.
    "aber",
    "jedoch",
    "sondern",
];

/// Degree words that grade the adjective following them. They stand inside a
/// noun phrase without being a modifier or the head of it.
pub(crate) const DEGREE_MODIFIERS: &[&str] = &[
    "etwas",
    "sehr",
    "ziemlich",
    "recht",
    "besonders",
    "eher",
    "noch",
    "weit",
    "weitaus",
    "deutlich",
    "leicht",
    "kaum",
    "durchaus",
    "überaus",
    "äußerst",
    "höchst",
    "relativ",
    "vergleichsweise",
    "zunehmend",
    "teilweise",
    "meist",
    "vorwiegend",
    "überwiegend",
    // Quantity words used as degree: "der milde, aber **wenig** angenehme Pilz".
    "wenig",
    "viel",
    "fast",
    "nahezu",
    "annähernd",
    "ausgesprochen",
    "vorwiegend",
];

/// Does this token open a noun phrase — an article, another determiner, a
/// possessive, a preposition or a spelled-out number?
pub(crate) fn opens_noun_phrase(token: &Token, document: &Document) -> bool {
    // Punctuation, numbers, whitespace and symbols never open a noun
    // phrase. Only a genuine word can.
    if !matches!(token.kind, TokenKind::Word(_)) {
        return false;
    }

    if token.kind.is_preposition() || token.kind.is_determiner() {
        return true;
    }

    let lower = lowercase_of(token, document);
    NOUN_PHRASE_LICENSORS.contains(&lower.as_str())
        || NP_BARE_PREPOSITIONS.contains(&lower.as_str())
        || NUMERALS.contains(&lower)
}

/// Does this token supply a **determiner**, rather than merely open a phrase?
///
/// The distinction only matters for nominalized adjectives, which German
/// licenses with a determiner and writes with a declension ending: *das
/// Braune*, *im Freien*. A bare preposition or a numeral supplies neither, so
/// *"weiß bis braun"*, *"von gelb zu weiß"* and *"davon sind vier unbewohnt"*
/// are predicative adjectives — not noun phrases whose head happens to be
/// lower case.
pub(crate) fn supplies_determiner(token: &Token, document: &Document) -> bool {
    if !matches!(token.kind, TokenKind::Word(_)) {
        return false;
    }
    if token.kind.is_determiner() {
        return true;
    }
    NOUN_PHRASE_LICENSORS.contains(&lowercase_of(token, document).as_str())
}

/// Is this adjective in its undeclined base form?
///
/// German nominalizes an adjective *with* a declension ending — "für
/// **Deutsche**", "das **Gute**", "im **Freien**" — so a declined form after
/// a bare preposition is a real nominalization and stays a candidate. The
/// base form never is one: "weiß bis **braun**" and "von **gelb** zu weiß"
/// are predicative, and the noun spelling would be a separate lexeme ("das
/// Braun") rather than this word.
pub(crate) fn is_base_form_adjective(token: &Token, document: &Document) -> bool {
    let lower = lowercase_of(token, document);
    !(lower.ends_with('e')
        || lower.ends_with("en")
        || lower.ends_with("er")
        || lower.ends_with("es")
        || lower.ends_with("em"))
}

/// Can this token sit *inside* a noun phrase — as an attributive adjective,
/// as the head noun, or as an unknown word standing in for one?
///
/// Deliberately reads only the metadata already attached to the token by
/// `Document::parse`. Calling `Dictionary::get_word_metadata` once per token
/// would take `CompoundAwareDictionary`'s global mutex and attempt a
/// compound decomposition on every miss.
pub(crate) fn continues_noun_phrase(token: &Token, document: &Document) -> bool {
    if !matches!(token.kind, TokenKind::Word(_)) {
        return false;
    }

    let chars = document.get_span_content(&token.span);
    if chars.is_empty() || !chars.iter().all(|c| c.is_alphabetic()) {
        return false;
    }

    // Closed-class words close the phrase: "die Zeit **im** Büro" is two
    // noun phrases, not one.
    if token.kind.is_determiner()
        || token.kind.is_preposition()
        || token.kind.is_pronoun()
        || token.kind.is_conjunction()
    {
        return false;
    }

    // A capital letter mid-sentence marks the head noun (or a proper name),
    // and it wins over every part-of-speech reading below. The dictionary
    // hands out spurious adverb and verb readings freely — "Band" is tagged
    // an adverb — and rejecting the token on one of those truncates the
    // phrase and promotes the attributive adjective before it to head.
    // `GERMAN_NON_NOUNS` suppresses lints on *lowercase* verb forms, several
    // of which are perfectly good nouns when written with a capital — "die
    // Frage", "die Sage", "die Suche".
    if chars.first().is_some_and(|c| c.is_uppercase()) {
        return true;
    }

    // Adverbs close the phrase. Only adjectives stand between the determiner
    // and the head, so "das Thema **oftmals** behandelt" ends the phrase at
    // "Thema" instead of making the adverb its head. Words that are both —
    // most German adjectives double as adverbs — keep going.
    if token.kind.is_adverb() && !token.kind.is_adjective() {
        return false;
    }

    let lower = lowercase_of(token, document);
    if GERMAN_NON_NOUNS.contains(&lower.as_str()) {
        return false;
    }

    // An adjective reading marks an attributive modifier, a noun reading a
    // (miscapitalized) head. An out-of-vocabulary word is most likely one of
    // the two, and treating it as phrase-internal keeps the head from being
    // mistaken for the word before it. So is a word the dictionary lists
    // with no part of speech at all — `diversifizierte` is in there carrying
    // nothing, and ending the phrase on it made "eine **reiche** und
    // diversifizierte Tierwelt" a capitalization error.
    token.kind.is_noun()
        || token.kind.is_adjective()
        || token.kind.is_oov()
        || has_no_pos_reading(token)
}

/// Is the token a word the dictionary knows but gives no part of speech?
///
/// Distinct from [`TokenKind::is_oov`], which is the *missing* entry. An
/// entry with every reading empty carries no evidence either way, and the
/// chunker has to treat it the same as an unknown word rather than as a
/// phrase boundary.
fn has_no_pos_reading(token: &Token) -> bool {
    match &token.kind {
        TokenKind::Word(Some(metadata)) => {
            metadata.noun.is_none()
                && metadata.verb.is_none()
                && metadata.adjective.is_none()
                && metadata.adverb.is_none()
                && metadata.pronoun.is_none()
                && metadata.determiner.is_none()
                && metadata.conjunction.is_none()
                && !metadata.preposition
        }
        _ => false,
    }
}

/// Is the token at `index` a relative pronoun rather than an article?
///
/// German spells them the same — `der`, `die`, `das`, `dem`, `den` are both
/// — and the difference decides what follows: an article introduces a noun
/// phrase, a relative pronoun a verb-final clause. The comma is the reliable
/// surface signal, because German punctuates every relative clause:
///
/// ```text
/// der SV Rödinghausen, der zuletzt 2019 den Pokal gewinnen konnte
///                      ^relative pronoun — "zuletzt" is not its head noun
/// ```
///
/// Reading it as an article crowns whatever comes next, which in a relative
/// clause is an adverb or a finite verb: `unterging`, `angibt`, `verstreut`
/// and `zuletzt` were all reported as miscapitalized nouns this way.
///
/// The cost is a lint inside *"das Haus, das große fenster hat"* — a noun
/// phrase really does follow there. It stays unflagged, which is the same
/// trade the rule makes everywhere else: a missed lint over a false one.
pub(crate) fn opens_relative_clause(tokens: &[&Token], index: usize, document: &Document) -> bool {
    if !RELATIVE_PRONOUNS.contains(&lowercase_of(tokens[index], document).as_str()) {
        return false;
    }

    index
        .checked_sub(1)
        .map(|previous| tokens[previous])
        .is_some_and(|previous| {
            matches!(
                previous.kind,
                TokenKind::Punctuation(Punctuation::Comma | Punctuation::OpenRound)
            )
        })
}

pub(crate) fn lowercase_of(token: &Token, document: &Document) -> String {
    document
        .get_span_content(&token.span)
        .iter()
        .map(|c| c.to_lowercase().next().unwrap_or(*c))
        .collect()
}

/// Is this token an opening quote or bracket, by its characters?
///
/// The lexer is uneven about them: German's closing `“` arrives as
/// `Punctuation::Quote`, but the opening `„` as `Unlintable`. Reading the
/// character sidesteps the classification, and matching `Unlintable`
/// wholesale is not an option — it is also what inline code blocks get, and
/// those really do end a noun phrase.
fn is_opening_mark(token: &Token, document: &Document) -> bool {
    let chars = document.get_span_content(&token.span);
    chars.len() == 1 && matches!(chars[0], '„' | '“' | '»' | '«' | '‚' | '‘' | '›' | '‹')
}

/// How far an extended attribute reaches, when the preposition at `index`
/// opens one.
///
/// German lets an attribute grow a phrase of its own in front of the
/// adjective: *die einzige **in Mitteleuropa** heimische Pflanzenart*, *die ganze
/// **nach links hinten** verlagerte Last*, *die medizinische und **in der
/// Regel** laienhafte Beurteilung*. Stopping at the preposition crowns the
/// adjective in front of it and reports *einzige*, *ganze*, *medizinische*
/// as nouns that lost their capital.
///
/// Returning the index of the adjective the attribute ends on rather than a
/// bare yes keeps the head where it belongs: skipping one token at a time
/// would stop at the capitalized *Mitteleuropa* inside the attribute and
/// make that the head.
///
/// **Two things keep this safe**, and both are needed. The word in front of
/// the preposition has to be an adjective, because that is the whole
/// difference between an attribute and a postmodifier: *die einzige **in
/// Mitteleuropa** heimische Pflanzenart* carries on to its head, *die Blume
/// **in dem großen Garten*** is finished, and the two are the same shape
/// from here on. And the attribute has to close the way an attribute must —
/// a lower-case adjective with a capitalized word directly behind it.
/// Without either of them the phrase ends at the preposition, as before.
fn extended_attribute_before_head(
    tokens: &[&Token],
    index: usize,
    document: &Document,
) -> Option<usize> {
    /// How far past the preposition the closing adjective may sit.
    const REACH: usize = 8;

    let capitalized = |token: &Token| {
        matches!(token.kind, TokenKind::Word(_))
            && document
                .get_span_content(&token.span)
                .first()
                .is_some_and(|c| c.is_uppercase())
    };

    for at in index + 1..(index + 1 + REACH).min(tokens.len().saturating_sub(1)) {
        // A clause boundary is the end of the attribute and of the search.
        if matches!(
            tokens[at].kind,
            TokenKind::Punctuation(
                Punctuation::Period | Punctuation::Semicolon | Punctuation::Colon
            )
        ) {
            return None;
        }

        if !capitalized(tokens[at])
            && continues_noun_phrase(tokens[at], document)
            && capitalized(tokens[at + 1])
        {
            return Some(at);
        }
    }

    None
}

/// Does the phrase pick up again after the joiner or degree word at `index`?
///
/// Several of them may stack — "eine große, aber **noch** **recht** junge
/// Sammlung" — so the skip repeats until a token either continues the phrase
/// or ends it.
fn phrase_resumes_after(tokens: &[&Token], index: usize, document: &Document) -> bool {
    // Inside quotation marks a capitalized word is a title, whatever its
    // part of speech: *das neue „**Wir**“* is a noun phrase, not a pronoun
    // ending one.
    let quoted = is_opening_mark(tokens[index], document);

    let mut next = index + 1;

    while next < tokens.len() {
        if quoted
            && matches!(tokens[next].kind, TokenKind::Word(_))
            && document
                .get_span_content(&tokens[next].span)
                .first()
                .is_some_and(|c| c.is_uppercase())
        {
            return true;
        }

        if continues_noun_phrase(tokens[next], document) {
            return true;
        }

        // The joiner may be followed by an extended attribute rather than
        // by the adjective itself: *die medizinische und **in der Regel**
        // laienhafte Beurteilung*. The preposition is the start of one, not
        // the end of the phrase, whenever the attribute closes on an
        // adjective in front of the head.
        if tokens[next].kind.is_preposition()
            && tokens[index - 1].kind.is_adjective()
            && extended_attribute_before_head(tokens, next, document).is_some()
        {
            return true;
        }

        if !skippable_inside_phrase(tokens[next], tokens.get(next - 1).copied(), document) {
            return false;
        }

        next += 1;
    }

    false
}

/// May this token stand between an attributive adjective and the head?
///
/// German puts a surprising amount here: a grading adverb (*"eine große,
/// aber **noch** recht junge Sammlung"*), a numeral or an ordinal (*"die
/// ehemalige **84.** Oberschule"*, *"eine große, **1671** gefertigte Uhr"*,
/// *"der lange **0,9 m** breite Gang"*), and further joiners when several
/// stack (*"reich, **aber** **auch** vielfältig"*). None of them is a
/// modifier or the head; stopping on one crowns the adjective in front of it
/// and reports a capitalization error on a perfectly ordinary attributive.
fn skippable_inside_phrase(token: &Token, prev: Option<&Token>, document: &Document) -> bool {
    if matches!(
        token.kind,
        TokenKind::Number(_) | TokenKind::Decade | TokenKind::Punctuation(Punctuation::Comma)
    ) {
        return true;
    }

    // The full stop of an ordinal, and only that one — a sentence-final
    // period is followed by a capitalized word, which `continues_noun_phrase`
    // accepts, so skipping it would run one phrase into the next sentence.
    if matches!(token.kind, TokenKind::Punctuation(Punctuation::Period))
        && prev.is_some_and(|p| matches!(p.kind, TokenKind::Number(_) | TokenKind::Decade))
    {
        return true;
    }

    let lower = lowercase_of(token, document);
    DEGREE_MODIFIERS.contains(&lower.as_str())
        || COORDINATORS.contains(&lower.as_str())
        || UNIT_ABBREVIATIONS.contains(&lower)
}

/// The scan itself. See the module header for the shape it relies on.
///
/// Looking only at the token to the left, as the capitalization rule once did,
/// flags `wesentliche` in the first phrase of the diagram above and both
/// `große` and `schöne` in the second, while missing `hund` — the word that
/// actually needs a capital.
fn chunk(tokens: &[&Token], document: &Document) -> Vec<Phrase> {
    let mut found = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        if !opens_noun_phrase(tokens[i], document) || opens_relative_clause(tokens, i, document) {
            i += 1;
            continue;
        }

        let mut end = i + 1;
        loop {
            if end >= tokens.len() {
                break;
            }

            // Coordinated attributive adjectives are joined by a comma or a
            // conjunction: "eine neue, radikalere Welle", "in britische,
            // französische und niederländische Kolonien". Step over the
            // joiner so the adjectives stay modifiers instead of each
            // becoming a phrase of its own.
            let joins_coordination =
                matches!(tokens[end].kind, TokenKind::Punctuation(Punctuation::Comma))
                    || COORDINATORS.contains(&lowercase_of(tokens[end], document).as_str());

            // A degree word may sit in front of any of those adjectives:
            // "der gerade oder **etwas** gekrümmte Griffel". It is neither a
            // modifier nor the head, but stopping on it would leave the
            // adjective before it standing as the head.
            let grades_next_adjective =
                DEGREE_MODIFIERS.contains(&lowercase_of(tokens[end], document).as_str());

            // So can a numeral or an opening bracket or quote: "das
            // beginnende **19.** Jahrhundert", "die britische
            // **4x100-Meter-**Mannschaft", "eine eigene **„**Baumnorm“",
            // "die deutsche **(**Wieder-)Besiedlung".
            let interrupts_phrase = matches!(
                tokens[end].kind,
                TokenKind::Number(_)
                    | TokenKind::Decade
                    | TokenKind::Punctuation(
                        Punctuation::Quote(_) | Punctuation::OpenRound | Punctuation::OpenSquare
                    )
            ) || is_opening_mark(tokens[end], document);

            if (joins_coordination || grades_next_adjective || interrupts_phrase)
                && end > i + 1
                && phrase_resumes_after(tokens, end, document)
            {
                end += 1;
                continue;
            }

            // German hangs a whole phrase in front of the adjective it
            // modifies — the *erweitertes Attribut*: "die einzige **in
            // Mitteleuropa** heimische Pflanzenart", "die ganze **nach
            // links hinten** verlagerte Last". Its preposition looks like
            // the end of the noun phrase and is the middle of it.
            let opens_extended_attribute = end > i + 1
                && tokens[end].kind.is_preposition()
                && tokens[end - 1].kind.is_adjective();
            if let Some(resumes_at) = opens_extended_attribute
                .then(|| extended_attribute_before_head(tokens, end, document))
                .flatten()
            {
                end = resumes_at;
                continue;
            }

            if !continues_noun_phrase(tokens[end], document) {
                break;
            }

            let capitalized = document
                .get_span_content(&tokens[end].span)
                .first()
                .is_some_and(|c| c.is_uppercase());
            end += 1;
            if capitalized {
                break;
            }
        }
        // A trailing joiner is not part of the phrase.
        while end > i + 1
            && (matches!(
                tokens[end - 1].kind,
                TokenKind::Punctuation(Punctuation::Comma)
            ) || COORDINATORS.contains(&lowercase_of(tokens[end - 1], document).as_str()))
        {
            end -= 1;
        }

        // A lone adjective under something that is not a determiner is
        // predicative, not a nominalization: "die Markzone ist weiß bis
        // **braun**", "von **gelb** zu weiß", "davon sind vier
        // **unbewohnt**". German needs a determiner for the nominal reading,
        // and then writes the declension ending with it — "das Braune". The
        // phrase is left headless rather than crowning the adjective.
        let predicative = end == i + 2
            && tokens[end - 1].kind.is_adjective()
            && is_base_form_adjective(tokens[end - 1], document)
            && !supplies_determiner(tokens[i], document);

        // An ordinal ends the *sentence* as far as the segmenter is
        // concerned, so "das sowjetische 170. | Regiment" arrives here cut in
        // half and the adjective is the last thing left. Everything after the
        // phrase being a numeral and a full stop is the signature of that
        // split; the head is in the next sentence, not on the adjective.
        let split_at_ordinal = tokens[end..].iter().all(|t| {
            matches!(
                t.kind,
                TokenKind::Number(_)
                    | TokenKind::Decade
                    | TokenKind::Punctuation(Punctuation::Period)
            )
        }) && tokens[end..]
            .iter()
            .any(|t| matches!(t.kind, TokenKind::Number(_) | TokenKind::Decade));

        if end > i + 1 && !predicative && !(split_at_ordinal && tokens[end - 1].kind.is_adjective())
        {
            found.push(Phrase {
                open: i,
                head: end - 1,
                end,
            });
        }

        // Resume at the phrase end; that token may itself open the next one.
        i = end.max(i + 1);
    }

    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TokenStringExt;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;

    /// The words of a sentence, and the chunking of it, side by side.
    fn chunked(text: &str) -> (Vec<String>, Vec<Phrase>, Vec<Role>) {
        let dictionary = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dictionary);
        let sentence = document
            .iter_sentences()
            .next()
            .expect("the test sentences are not empty");
        let tokens: Vec<&Token> = sentence
            .iter()
            .filter(|token| !token.kind.is_whitespace())
            .collect();
        let words = tokens
            .iter()
            .map(|token| document.get_span_content(&token.span).iter().collect())
            .collect();
        (
            words,
            phrases(&tokens, &document),
            roles(&tokens, &document),
        )
    }

    /// The heads of every phrase, as written.
    fn heads(text: &str) -> Vec<String> {
        let (words, found, _) = chunked(text);
        found.iter().map(|p| words[p.head].clone()).collect()
    }

    #[test]
    fn a_bare_phrase_has_the_noun_as_its_head() {
        assert_eq!(heads("Die Frage ist offen."), vec!["Frage"]);
        assert_eq!(heads("Der Hund schläft."), vec!["Hund"]);
    }

    #[test]
    fn the_head_sits_behind_its_adjectives() {
        assert_eq!(heads("Die wesentliche Frage ist offen."), vec!["Frage"]);
        assert_eq!(heads("Der große schöne Hund schläft."), vec!["Hund"]);
    }

    #[test]
    fn every_phrase_in_a_sentence_is_found() {
        assert_eq!(
            heads("Die Kinder spielen in dem großen Garten."),
            vec!["Kinder", "Garten"]
        );
    }

    #[test]
    fn coordinated_adjectives_stay_inside_one_phrase() {
        assert_eq!(heads("Eine neue, radikalere Welle kam."), vec!["Welle"]);
        assert_eq!(
            heads("Sie fuhren in britische, französische und niederländische Kolonien."),
            vec!["Kolonien"]
        );
    }

    #[test]
    fn an_extended_attribute_does_not_end_the_phrase() {
        assert_eq!(
            heads("Die einzige in Mitteleuropa heimische Pflanzenart blüht."),
            vec!["Pflanzenart"]
        );
        assert_eq!(
            heads("Der kluge mit vielen Büchern ausgestattete Raum gefiel ihm."),
            vec!["Raum"]
        );
    }

    /// The adjective in front of the preposition is what separates an
    /// attribute from a postmodifier, so without one the phrase ends at the
    /// preposition and the attribute becomes a phrase of its own. *Die in
    /// Mitteleuropa heimische Pflanzenart* is therefore not chunked as one
    /// phrase, and *die Blume in dem großen Garten* must not be either.
    #[test]
    fn an_attribute_without_an_adjective_in_front_is_not_read_as_one() {
        assert_eq!(
            heads("Die in Mitteleuropa heimische Pflanzenart blüht."),
            vec!["Mitteleuropa"]
        );
        assert_eq!(
            heads("Die Blume in dem großen Garten blüht."),
            vec!["Blume", "Garten"]
        );
    }

    #[test]
    fn a_relative_pronoun_opens_no_phrase() {
        // *der* behind a comma is a relative pronoun; crowning *zuletzt* is the
        // mistake the comma test exists to prevent.
        let found = heads("Der Verein, der zuletzt gewann, spielt heute.");
        assert!(!found.iter().any(|head| head == "zuletzt"), "{found:?}");
    }

    #[test]
    fn a_predicative_adjective_is_not_a_head() {
        assert!(
            heads("Die Markzone ist weiß bis braun.")
                .iter()
                .all(|h| h != "braun")
        );
        assert!(
            heads("Davon sind vier unbewohnt.")
                .iter()
                .all(|h| h != "unbewohnt")
        );
    }

    #[test]
    fn a_phrase_reports_its_own_extent() {
        let (words, found, _) = chunked("Die wesentliche Frage ist offen.");
        let phrase = found[0];
        assert_eq!(words[phrase.open], "Die");
        assert_eq!(words[phrase.head], "Frage");
        assert_eq!(phrase.end, phrase.head + 1);
    }

    #[test]
    fn the_lookups_find_the_phrase_by_either_end() {
        let (words, found, _) = chunked("Die wesentliche Frage ist offen.");
        let open = words.iter().position(|w| w == "Die").unwrap();
        let head = words.iter().position(|w| w == "Frage").unwrap();
        assert_eq!(phrase_opened_by(&found, open), Some(&found[0]));
        assert_eq!(phrase_headed_by(&found, head), Some(&found[0]));
        assert_eq!(phrase_headed_by(&found, open), None);
    }

    #[test]
    fn the_roles_agree_with_the_phrases() {
        let (words, found, labels) = chunked("Die wesentliche Frage ist offen.");
        for (index, word) in words.iter().enumerate() {
            let expected = match word.as_str() {
                "Frage" => Role::Head,
                "wesentliche" => Role::Modifier,
                _ => Role::Outside,
            };
            assert_eq!(labels[index], expected, "{word}");
        }
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn phrases_never_overlap() {
        let (_, found, _) = chunked(
            "Die Kinder spielen in dem großen Garten, und der alte Hund schläft unter einem Baum.",
        );
        for pair in found.windows(2) {
            assert!(pair[0].end <= pair[1].open, "{pair:?}");
        }
    }

    #[test]
    fn a_capitalized_word_ends_the_phrase() {
        // Otherwise *in Munitionsfabriken eingesetzt* runs on and makes the
        // participle the head.
        assert_eq!(
            heads("Sie wurden in Munitionsfabriken eingesetzt."),
            vec!["Munitionsfabriken"]
        );
    }

    #[test]
    fn a_sentence_without_a_noun_phrase_yields_nothing() {
        let (_, found, labels) = chunked("Er schläft tief und fest.");
        assert!(found.is_empty(), "{found:?}");
        assert!(labels.iter().all(|role| *role == Role::Outside));
    }

    /// An ordinal ends the *sentence* as far as the segmenter is concerned,
    /// so *das beginnende 19. | Jahrhundert* arrives cut in half. The chunker
    /// leaves the fragment headless rather than crowning the adjective.
    #[test]
    fn an_ordinal_splits_the_sentence_and_the_phrase_stays_headless() {
        let (_, found, _) = chunked("Das beginnende 19. Jahrhundert brachte Veränderungen.");
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn a_degree_word_before_an_adjective_is_stepped_over() {
        assert_eq!(
            heads("Eine große, aber noch recht junge Sammlung entstand."),
            vec!["Sammlung"]
        );
    }

    #[test]
    fn a_preposition_opens_a_phrase_of_its_own() {
        assert_eq!(
            heads("Sie arbeitet mit dem neuen Verfahren."),
            vec!["Verfahren"]
        );
    }
}
