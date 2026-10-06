//! The verb has to match the subject in person and number.

use std::sync::Arc;

use crate::language::german::grammar::determiners::{
    determiner_readings, is_plural_only_quantifier,
};
use crate::language::german::grammar::noun_phrase::{self, Phrase};
use crate::language::german::grammar::subjects::{
    Features, irregular_finite_verb, subject_pronoun, subordinate_subject_pronoun,
};
use crate::language::morphology::{Case, MorphologyExt, Number, NumberSet, PersonSet};
use crate::linting::{Lint, LintKind, Linter};
use crate::spell::Dictionary;
use crate::{Punctuation, Token, TokenKind, TokenStringExt, document::Document};

/// Requires a finite verb to agree with the pronoun in front of it.
///
/// German marks person and number on the verb, so *er gehen* and *du hat* are
/// wrong in a way no amount of context can rescue. This is the one agreement
/// class Harper can decide without gender, and the features come from two
/// places, both of them complete:
///
/// * the **subject** is a personal pronoun, a closed class of eight forms, in
///   `grammar/subjects.rs`.
/// * the **verb** carries person and number from its conjugation affix
///   (`annotations.json`), or, when it is irregular enough that the dictionary
///   stores the whole form, from the table beside the pronouns.
///
/// Two positions are checked, and they are the two where the finite verb can
/// be *located* without parsing the sentence:
///
/// * the **front field**. German is verb-second, so when the pronoun opens its
///   clause the finite verb is the next word. Nowhere else in a main clause is
///   that true — in *das er vergessen hat* the word behind the pronoun is a
///   participle — and checking the other pairs reported eight hundred times on
///   edited prose and was wrong every time.
/// * the **end of a subordinate clause**. A clause opened by *dass*, *weil* or
///   *wie* is verb-final, so the last word before the clause's punctuation is
///   the verb. This is the half of the language the front field cannot see, and
///   LanguageTool does not see it either: its `DE_VERBAGREEMENT` wants the two
///   words adjacent, so *Es ist nicht so wie es sein sollt* goes unreported by
///   both tools until now.
///
/// The second position is worth less than it looks, because `-en` has to be
/// thrown away there — it is the infinitive, the plural and the declined
/// adjective at once. What is left is the irregular auxiliaries and modals,
/// which is where the frequency is anyway.
///
/// A **noun-phrase subject** — *die Kinder spielt* — is not checked, and the
/// attempt is worth recording. Finding the head is easy enough, but the word
/// behind it is not reliably the verb: *die Gesellschaft bürgerlichen Rechts*
/// and *die Arten hohler Stängel* put an adjective there, and a relative clause
/// behind a comma (*…, welches Sittenwidrigkeit impliziert*) passes the
/// front-field test while being verb-final. That version reported 1229 times on
/// the same prose. It needs the noun-phrase chunker the capitalization rule
/// has, not another guard.
/// Conjunctions that put their clause in verb-final order.
///
/// Only the unambiguous ones, and only those that can be followed directly by
/// a pronoun subject. A word that is also a question word (*wie*, *wo*) is
/// still safe here, because the check needs a pronoun immediately behind it
/// and a question puts the verb there instead: *wie geht es* never matches,
/// *wie es sein soll* does.
const SUBORDINATORS: &[&str] = &[
    "dass",
    "weil",
    "wenn",
    "ob",
    "obwohl",
    "obgleich",
    "obschon",
    "sodass",
    "damit",
    "falls",
    "sobald",
    "solange",
    "seitdem",
    "nachdem",
    "bevor",
    "während",
    "zumal",
    "sofern",
    "soweit",
    "wohingegen",
    "indem",
    "wenngleich",
    "da",
    "wie",
    "wo",
];

/// Coordinating conjunctions, which end the clause for this purpose.
///
/// *da ich keine Beweise hätte und der Dieb es nicht zurückgeben wird* puts a
/// second clause with a subject of its own behind the *und*, and the word at
/// the end belongs to that one. Stopping early can only cost a detection,
/// never cause a report: what it hands back instead is a word from the middle
/// of a phrase, which the two filters below throw away.
const COORDINATORS: &[&str] = &["und", "oder", "aber", "sondern", "denn", "doch", "sowie"];

pub struct GermanSubjectVerbAgreement<T>
where
    T: Dictionary,
{
    dictionary: Arc<T>,
}

impl<T: Dictionary> GermanSubjectVerbAgreement<T> {
    pub fn new(dictionary: Arc<T>) -> Self {
        Self { dictionary }
    }

    /// The readings of a finite verb form, as *joint* person/number pairs.
    ///
    /// A list rather than two sets, and the reason is the `-t` ending: it is
    /// third person singular and second person plural, and independent axes
    /// would also admit third person plural, which is exactly the reading *die
    /// Kinder spielt* needs ruled out. The determiner table next door keeps its
    /// readings joint for the same reason.
    ///
    /// The dictionary says *whether* the word is a finite form and roughly
    /// which features it has; the ending says how they pair up. Nothing else
    /// can: an affix rule carries one metadata block for all its replacements.
    ///
    /// The irregular table wins over the dictionary: *ist*, *hat* and *sind*
    /// exist as entries of their own and carry generic verb flags, some of
    /// which are also affixes and would hand back a reading built for a
    /// different word.
    fn verb_features(&self, word: &str, token: &Token) -> Vec<Features> {
        if let Some(features) = irregular_finite_verb(word) {
            return vec![features];
        }

        // The affix-built forms. What identifies them is the person and number
        // the conjugation affix left behind, not a part of speech: the
        // preterite affixes sit on two thousand noun and adjective entries as
        // well, so they cannot declare their output a verb without turning
        // those into verbs too.
        //
        // What the part of speech does here is rule things out. The `-st`
        // affix is applied to pronoun and adjective roots, so `selbst` and
        // `möglichst` arrive carrying a second person singular; a determiner, a
        // pronoun or an adverb is not the finite verb of the clause. The
        // article `die` used to need this too, until the root that built it
        // lost the flag — see `tests/verb_person_test.rs`.
        if token.kind.is_determiner()
            || token.kind.is_pronoun()
            || token.kind.is_preposition()
            || token.kind.is_conjunction()
        {
            return Vec::new();
        }

        // A finite verb is never capitalized mid-sentence, and the pronoun in
        // front of this one guarantees we are mid-sentence. What a capital
        // marks here is an apposition — *wir Arbeiter*, *wir Deutsche* — whose
        // head is a noun that happens to carry a conjugation affix.
        if word.chars().next().is_some_and(char::is_uppercase) {
            return Vec::new();
        }

        let chars: Vec<char> = word.chars().collect();
        let Some(metadata) = self.dictionary.get_word_metadata(&chars) else {
            return Vec::new();
        };

        // An adverb is not the finite verb, whatever else the entry says. The
        // `-st` affix is applied to adjective and pronoun roots as well as to
        // verbs, so `selbst` and `möglichst` arrive here carrying a second
        // person singular; sixteen of the seventeen reports left on the prose
        // corpus were that one ending.
        if metadata.is_adverb() {
            return Vec::new();
        }

        let agreement = metadata.verb_agreement();
        if agreement.person.is_empty() || agreement.number.is_empty() {
            return Vec::new();
        }

        readings_of_ending(word)
    }

    /// Does the token at `index` stand in the front field of its clause?
    ///
    /// That means the start of the sentence, or directly behind a comma or a
    /// coordinating conjunction — the positions from which German's verb-second
    /// rule puts the finite verb next. Behind a subordinator or a relative
    /// pronoun the clause is verb-final instead and the next word is anything
    /// but the finite verb.
    fn opens_a_clause(tokens: &[&Token], index: usize, document: &Document) -> bool {
        const COORDINATORS: &[&str] = &["und", "oder", "aber", "denn", "sondern", "doch"];

        let Some(previous) = index.checked_sub(1).map(|i| tokens[i]) else {
            return true;
        };

        match previous.kind {
            TokenKind::Punctuation(
                Punctuation::Comma | Punctuation::Semicolon | Punctuation::Colon,
            ) => true,
            TokenKind::Word(_) => {
                let word: String = document
                    .get_span_content(&previous.span)
                    .iter()
                    .flat_map(|c| c.to_lowercase())
                    .collect();
                COORDINATORS.contains(&word.as_str())
            }
            _ => false,
        }
    }

    /// Is the token at `index` the last member of a coordinated subject?
    ///
    /// *Mein Bruder und ich spielen* has a plural subject, so the pronoun that
    /// follows *und* does not open a clause of its own and *ich spielen* is not
    /// an error. *Sie kam nach Hause und er gehen weg* does open one, and what
    /// tells them apart is the finite verb in front of the conjunction: a
    /// coordinated clause has already had its verb, a coordinated subject has
    /// not. A verb the dictionary cannot place counts as absent, which can only
    /// cost a detection.
    fn joins_a_subject(&self, tokens: &[&Token], index: usize, document: &Document) -> bool {
        const JOINERS: &[&str] = &["und", "oder", "sowie"];

        let Some(coordinator) = index.checked_sub(1) else {
            return false;
        };
        if !JOINERS.contains(&Self::word_at(tokens, coordinator, document).as_str()) {
            return false;
        }

        !tokens[..coordinator]
            .iter()
            .rev()
            .take_while(|token| {
                !matches!(
                    token.kind,
                    TokenKind::Punctuation(
                        Punctuation::Comma | Punctuation::Semicolon | Punctuation::Colon
                    )
                )
            })
            .filter(|token| matches!(token.kind, TokenKind::Word(_)))
            .any(|token| self.may_be_a_verb(token, document))
    }

    /// Could this token be a verb at all, finite or not?
    ///
    /// Looser than `verb_features`, which wants the person and number a
    /// conjugation affix left behind: a preterite such as *kam* has neither, and
    /// is still the verb that makes *Sie kam nach Hause und er …* a clause. A
    /// capital marks a noun, so only lower-case words count.
    fn may_be_a_verb(&self, token: &Token, document: &Document) -> bool {
        let chars = document.get_span_content(&token.span);
        let word: String = chars.iter().collect();
        if !self.verb_features(&word, token).is_empty() {
            return true;
        }

        chars.first().is_some_and(|c| c.is_lowercase())
            && !(token.kind.is_determiner()
                || token.kind.is_pronoun()
                || token.kind.is_preposition()
                || token.kind.is_conjunction())
            && self
                .dictionary
                .get_word_metadata(chars)
                .is_some_and(|metadata| metadata.is_verb() && !metadata.is_noun())
    }

    /// The word at `index`, lower-cased.
    fn word_at(tokens: &[&Token], index: usize, document: &Document) -> String {
        document
            .get_span_content(&tokens[index].span)
            .iter()
            .flat_map(|c| c.to_lowercase())
            .collect()
    }

    /// The finite verb of a subordinate clause opened at `index`, if it can be
    /// found without guessing.
    ///
    /// German puts the finite verb last in a subordinate clause, which is the
    /// one other position where it can be located by counting rather than by
    /// parsing. The clause runs from the subject to the first punctuation mark
    /// or coordinating conjunction; its last word is the verb.
    ///
    /// The exception is the *Ersatzinfinitiv* — *dass er hat kommen können* —
    /// where the auxiliary leads a cluster of two bare infinitives instead of
    /// standing last. Two `-en` forms in a row is the shape of it, and also the
    /// shape of *dass wir gelesen haben*, where the last word really is the
    /// verb; both are skipped, because for a plural subject the reading would
    /// have agreed anyway.
    fn clause_final_verb(tokens: &[&Token], index: usize, document: &Document) -> Option<usize> {
        let mut words: Vec<usize> = Vec::new();

        for position in index..tokens.len() {
            match tokens[position].kind {
                TokenKind::Word(_) => {
                    if Self::word_ends_clause(tokens, position, document) {
                        break;
                    }
                    words.push(position);
                }
                TokenKind::Punctuation(Punctuation::Hyphen | Punctuation::Apostrophe) => {}
                TokenKind::Punctuation(_) => break,
                _ => {}
            }
        }

        let last = *words.last()?;

        // `-en` is banned in this position, and it is the whole reason the
        // front-field check was written first. The ending is the infinitive,
        // the first and third person plural, and the declined adjective, all
        // three — and clause-finally German uses every one of them. *dass er
        // versucht zu schlafen* extraposes an infinitive past the finite verb,
        // *damit er handeln können* belongs to a plural subject further out,
        // and *da er es in einem konkreten, empirischen Zusammenhang sieht*
        // ends a fragment on an adjective. All twenty reports the first version
        // produced on edited prose ended in `-en`.
        //
        // In the front field the position itself guaranteed a finite verb and
        // the ending could be trusted. Here nothing does, so the check keeps
        // only the endings an infinitive cannot wear.
        if Self::word_at(tokens, last, document).ends_with("en") {
            return None;
        }

        // And an adjective is not the verb either. The `-e` affix builds
        // *positive* as readily as *lerne*, and stopping at a coordinator can
        // leave the scan standing on one: *ob wir eine positive oder negative
        // Wirkung erwarten*. The front field never needed this, because a
        // pronoun is not followed by an attributive adjective.
        if tokens[last].kind.is_adjective() {
            return None;
        }

        Some(last)
    }

    fn word_ends_clause(tokens: &[&Token], index: usize, document: &Document) -> bool {
        COORDINATORS.contains(&Self::word_at(tokens, index, document).as_str())
    }

    /// The number a noun-phrase subject imposes, when its determiner fixes it.
    ///
    /// The **determiner** decides this, not the noun. German spells most
    /// masculine and neuter nouns the same in the nominative singular and
    /// plural — *der Lehrer* and *die Lehrer*, *das Fenster* and *die Fenster*
    /// — so the noun entry cannot tell them apart and the article always can.
    /// Reading the noun instead is how the earlier attempt invented errors.
    ///
    /// Only the **nominative** readings count, because only a nominative noun
    /// phrase is a subject. That is also what keeps *die* honest: it is
    /// feminine singular or plural in the nominative, so *die Lehrer* stays
    /// ambiguous and is not checked, while *der Lehrer* is singular and *diese
    /// Lehrer* plural.
    ///
    /// `None` means the determiner allows both numbers, or is not in the table
    /// at all. Nothing is checked then — an unknown subject narrows nothing.
    fn subject_number(
        tokens: &[&Token],
        document: &Document,
        phrase: &Phrase,
    ) -> Option<NumberSet> {
        let determiner = Self::word_at(tokens, phrase.open, document);
        if is_plural_only_quantifier(&determiner) {
            return Some(NumberSet::PLURAL);
        }

        let readings = determiner_readings(&determiner)?;

        let mut number = NumberSet::empty();
        for reading in readings.iter().filter(|r| r.case == Case::Nominative) {
            number |= match reading.number() {
                Number::Singular => NumberSet::SINGULAR,
                Number::Plural => NumberSet::PLURAL,
            };
        }

        // `die` is where this stops, and it is the most common determiner in
        // the language: in the nominative it is feminine singular *and*
        // plural. The noun cannot break the tie — `Kinder`, `Bücher` and
        // `Kirche` all carry `SINGULAR | PLURAL` in the dictionary, because an
        // entry describes a lemma and the plural affix hangs off the same one.
        // Narrowing by it was tried and produced seven reports on edited
        // prose, five of them from a wrong recorded number. So *die* phrases
        // are not checked.
        (number == NumberSet::SINGULAR || number == NumberSet::PLURAL).then_some(number)
    }

    /// The third position: a noun phrase in the front field.
    ///
    /// *Die Kinder spielt im Garten*. German is verb-second, so the finite verb
    /// stands directly behind the subject phrase — and with the chunker,
    /// "directly behind the phrase" is a token index rather than a guess.
    ///
    /// The guess is what the earlier attempt got wrong, to the tune of 1229
    /// reports on edited prose. Taking the word behind the head noun put it on
    /// an adjective in *die Gesellschaft bürgerlichen Rechts* and *die Arten
    /// hohler Stängel*; the chunker keeps those inside the phrase. And a
    /// relative clause behind a comma — *…, welches Sittenwidrigkeit
    /// impliziert* — passed the front-field test while being verb-final; the
    /// chunker refuses to open a phrase on a relative pronoun.
    fn lint_noun_phrase_subjects(
        &self,
        tokens: &[&Token],
        document: &Document,
        lints: &mut Vec<Lint>,
    ) {
        for phrase in noun_phrase::phrases(tokens, document) {
            if !Self::opens_a_clause(tokens, phrase.open, document) {
                continue;
            }

            // A coordinator in front of the phrase joins it to the one before
            // rather than starting a clause, and two coordinated noun phrases
            // are a *plural* subject however singular each half is: *die
            // Kodierung und das Format hängen*, *andere Tiere und der Mensch
            // sind*. The front-field test accepts a coordinator because a
            // pronoun behind one really does open a clause; a noun phrase
            // behind one usually does not.
            if index_of_coordinator_before(tokens, phrase.open, document) {
                continue;
            }

            let Some(number) = Self::subject_number(tokens, document, &phrase) else {
                continue;
            };
            let subject = Features::new(PersonSet::THIRD, number);

            let Some(verb_token) = tokens.get(phrase.end) else {
                continue;
            };
            if !matches!(verb_token.kind, TokenKind::Word(_)) {
                continue;
            }

            // A postposed genitive attribute leaves an adjective in the verb's
            // place: *der Brennwert **reinen** Fettes beträgt*, *die Arten
            // **hohler** Stängel*. The chunker ends the phrase at the
            // capitalized head, so the attribute behind it lands here.
            if verb_token.kind.is_adjective() {
                continue;
            }

            // And `-en` is refused here too, for the third time and a third
            // reason. Behind a noun phrase it is the infinitive a modal or a
            // *zu* governs, with the phrase as its **object**: *eine Pandemie
            // auszulösen*, *ein Glas trinken*, *ein Haustier halten*. Every one
            // of the forty-five reports left at this point ended in `-en`, and
            // every one of them was this.
            //
            // The pattern across all three positions is one fact: `-en` is the
            // infinitive, the plural and the declined adjective, and only
            // directly behind a personal pronoun does verb-second guarantee
            // which of them it is. The cost is the *singular subject, plural
            // verb* direction — *das Kind spielen* goes unreported — and what
            // is kept is *die Kinder spielt*.
            if Self::word_at(tokens, phrase.end, document).ends_with("en") {
                continue;
            }

            let verb_text: String = document.get_span_content(&verb_token.span).iter().collect();
            if another_subject_follows(tokens, document, phrase.end) {
                continue;
            }

            let readings = self.verb_features(&verb_text, verb_token);
            if readings.is_empty() || readings.iter().any(|r| subject.agrees_with(r)) {
                continue;
            }

            // The copula lets the **predicate** carry the number, but only in
            // one direction: *ein weiteres Problem sind die langen Wege* is
            // correct German, while *viele Leute ist müde* is not. A singular
            // subject with a plural copula is therefore left alone, and the
            // reverse stays reportable.
            if number == NumberSet::SINGULAR
                && is_copula(&verb_text)
                && readings.iter().any(|r| r.number == NumberSet::PLURAL)
            {
                continue;
            }

            let subject_text: String = (phrase.open..=phrase.head)
                .map(|at| {
                    document
                        .get_span_content(&tokens[at].span)
                        .iter()
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join(" ");
            lints.push(Self::report(
                &subject_text,
                &subject,
                &verb_text,
                verb_token,
            ));
        }
    }

    fn report(subject_text: &str, subject: &Features, verb_text: &str, verb: &Token) -> Lint {
        Lint {
            span: verb.span,
            lint_kind: LintKind::Agreement,
            suggestions: Vec::new(),
            priority: 30,
            message: format!(
                "»{subject_text}« verlangt {}. »{verb_text}« steht in einer anderen Form.",
                Self::wanted(subject)
            ),
        }
    }

    /// The second position where the finite verb can be found by counting.
    ///
    /// A subordinate clause is verb-final, so *weil du kommen sollt* pairs the
    /// pronoun behind the conjunction with the last word of the clause. This is
    /// the half of the language the front-field check cannot see, and neither
    /// tool Harper is measured against sees it either: LanguageTool's
    /// `DE_VERBAGREEMENT` wants the two words adjacent.
    ///
    /// `es` is admitted here and nowhere else — see
    /// [`subordinate_subject_pronoun`] — but only its **person** is checked.
    /// The number comes from the predicate in *weil es meine Freunde sind*, and
    /// no third-person form is ever wrong after *es*. What is left is the
    /// second person, which *es* can never take: *wie es sein sollt*.
    fn lint_subordinate_clauses(
        &self,
        tokens: &[&Token],
        document: &Document,
        lints: &mut Vec<Lint>,
    ) {
        for index in 0..tokens.len().saturating_sub(2) {
            if !matches!(tokens[index].kind, TokenKind::Word(_))
                || !SUBORDINATORS.contains(&Self::word_at(tokens, index, document).as_str())
            {
                continue;
            }

            let subject_index = index + 1;
            if !matches!(tokens[subject_index].kind, TokenKind::Word(_)) {
                continue;
            }

            let subject_text: String = document
                .get_span_content(&tokens[subject_index].span)
                .iter()
                .collect();
            let Some(subject) = subordinate_subject_pronoun(&subject_text) else {
                continue;
            };

            let Some(verb_index) = Self::clause_final_verb(tokens, subject_index + 1, document)
            else {
                continue;
            };

            let verb_token = tokens[verb_index];
            let verb_text: String = document.get_span_content(&verb_token.span).iter().collect();
            let readings = self.verb_features(&verb_text, verb_token);
            if readings.is_empty() {
                continue;
            }

            let placeholder = subject_text.eq_ignore_ascii_case("es");
            let agrees = |reading: &Features| {
                if placeholder {
                    subject.person.agrees_with(reading.person)
                } else {
                    subject.agrees_with(reading)
                }
            };
            if readings.iter().any(agrees) {
                continue;
            }

            lints.push(Self::report(
                &subject_text,
                &subject,
                &verb_text,
                verb_token,
            ));
        }
    }

    /// Which form the subject wants, spelled out for the message.
    fn wanted(features: &Features) -> &'static str {
        use crate::language::morphology::{NumberSet, PersonSet};
        match (features.person, features.number) {
            (PersonSet::FIRST, NumberSet::SINGULAR) => "die 1. Person Singular",
            (PersonSet::SECOND, NumberSet::SINGULAR) => "die 2. Person Singular",
            (PersonSet::THIRD, NumberSet::SINGULAR) => "die 3. Person Singular",
            (PersonSet::FIRST, NumberSet::PLURAL) => "die 1. Person Plural",
            (PersonSet::SECOND, NumberSet::PLURAL) => "die 2. Person Plural",
            (PersonSet::THIRD, NumberSet::PLURAL) => "die 3. Person Plural",
            _ => "eine andere Form",
        }
    }
}

impl<T: Dictionary> Linter for GermanSubjectVerbAgreement<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let tokens: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();

            for index in 0..tokens.len().saturating_sub(1) {
                if !Self::opens_a_clause(&tokens, index, document)
                    || self.joins_a_subject(&tokens, index, document)
                {
                    continue;
                }

                let (subject_token, verb_token) = (tokens[index], tokens[index + 1]);
                if !matches!(subject_token.kind, TokenKind::Word(_))
                    || !matches!(verb_token.kind, TokenKind::Word(_))
                {
                    continue;
                }

                let subject_text: String = document
                    .get_span_content(&subject_token.span)
                    .iter()
                    .collect();
                let Some(subject) = subject_pronoun(&subject_text) else {
                    continue;
                };

                let verb_text: String =
                    document.get_span_content(&verb_token.span).iter().collect();
                let verb = self.verb_features(&verb_text, verb_token);
                if verb.is_empty() || verb.iter().any(|reading| subject.agrees_with(reading)) {
                    continue;
                }

                lints.push(Self::report(
                    &subject_text,
                    &subject,
                    &verb_text,
                    verb_token,
                ));
            }

            self.lint_subordinate_clauses(&tokens, document, &mut lints);
            self.lint_noun_phrase_subjects(&tokens, document, &mut lints);
        }

        lints
    }

    fn description(&self) -> &str {
        "Prüft, ob das Verb in Person und Numerus zum Subjekt passt."
    }
}

/// Is this a copula, whose number may come from the predicate rather than the
/// subject?
///
/// *Ein Beispiel sind die Streuobstwiesen* is correct German: with *sein*,
/// *werden* and *bleiben* the predicate nominative can carry the number, so a
/// singular subject and a plural verb is not an error. This is the same
/// concession `es` gets in a subordinate clause, for the same reason — and it
/// is only needed where the subject is a noun phrase, because a personal
/// pronoun never yields the number this way.
fn is_copula(word: &str) -> bool {
    const COPULAS: &[&str] = &[
        "ist", "sind", "war", "waren", "sei", "seien", "wäre", "wären", "bin", "bist", "seid",
        "wird", "werden", "wurde", "wurden", "würde", "würden", "bleibt", "bleiben", "blieb",
        "blieben",
    ];
    COPULAS.contains(&word.to_lowercase().as_str())
}

/// Prepositions and preposition–article contractions, which mark the phrase
/// behind them as governed and therefore never the subject.
const GOVERNORS: &[&str] = &[
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
    "innerhalb",
    "außerhalb",
    "gegenüber",
    "neben",
    "zwischen",
    "hinter",
    "im",
    "am",
    "zum",
    "zur",
    "beim",
    "vom",
    "ins",
    "ans",
    "aufs",
    "durchs",
    "fürs",
    "ums",
    "übers",
    "unters",
    "hinters",
    "vors",
];

/// Could something behind the verb be the clause's real subject?
///
/// German's front field takes **any** one constituent, not just the subject,
/// and when it takes an object the subject moves in behind the verb: *Eine
/// Rolle spielen auch regionale Unterschiede*, *Eine Ausnahme stellen einige
/// Mundarten*, *Dieser Verfolgung fielen 100.000 Frauen zum Opfer*. All three
/// are correct, and all three look exactly like a singular subject with a
/// plural verb.
///
/// Nothing on the fronted phrase itself distinguishes the two readings —
/// *eine*, *der* and *das* are each nominative *and* something else, so the
/// determiner cannot say whether the phrase is a subject. What does say so is
/// the rest of the clause: if no other word behind the verb could be the
/// subject, the fronted phrase has to be it.
///
/// A phrase behind a preposition does not count, which is what keeps *die
/// Kinder spielt im Garten* reportable: *Garten* is capitalized but governed.
fn another_subject_follows(tokens: &[&Token], document: &Document, verb_at: usize) -> bool {
    let word_at = |at: usize| -> String {
        document
            .get_span_content(&tokens[at].span)
            .iter()
            .flat_map(|c| c.to_lowercase())
            .collect()
    };
    let capitalized = |at: usize| {
        document
            .get_span_content(&tokens[at].span)
            .first()
            .is_some_and(|c| c.is_uppercase())
    };

    let mut at = verb_at + 1;
    while at < tokens.len() {
        match tokens[at].kind {
            // The clause ends; nothing further belongs to this verb.
            TokenKind::Punctuation(
                Punctuation::Comma
                | Punctuation::Semicolon
                | Punctuation::Colon
                | Punctuation::Period,
            ) => return false,
            // A bare numeral is a quantity, and a quantity can be a subject:
            // *Dieser Verfolgung fielen 100.000 Frauen zum Opfer*.
            TokenKind::Number(_) => return true,
            TokenKind::Word(_) => {
                if GOVERNORS.contains(&word_at(at).as_str()) {
                    // Skip what the preposition governs, head included. The
                    // determiner and the adjectives in between are lower case
                    // and the head is capitalized, which is enough to find the
                    // end — `continues_noun_phrase` is not, because it
                    // deliberately stops at a determiner so that *die Zeit im
                    // Büro* chunks as two phrases.
                    at += 1;
                    while at < tokens.len() && matches!(tokens[at].kind, TokenKind::Word(_)) {
                        let head = capitalized(at);
                        at += 1;
                        if head {
                            break;
                        }
                    }
                    continue;
                }

                // A determiner with no nominative reading opens an object,
                // not a subject: *einen klaren Zusammenhang*, *dem Vorfall*,
                // *des Verfahrens*. Skip what it introduces, the same way a
                // preposition's phrase is skipped.
                if let Some(readings) = determiner_readings(&word_at(at))
                    && !readings.iter().any(|r| r.case == Case::Nominative)
                {
                    at += 1;
                    while at < tokens.len() && matches!(tokens[at].kind, TokenKind::Word(_)) {
                        let head = capitalized(at);
                        at += 1;
                        if head {
                            break;
                        }
                    }
                    continue;
                }

                if subject_pronoun(&word_at(at)).is_some() || capitalized(at) {
                    return true;
                }
            }
            _ => {}
        }
        at += 1;
    }

    false
}

/// Does a coordinating conjunction sit directly in front of `index`?
fn index_of_coordinator_before(tokens: &[&Token], index: usize, document: &Document) -> bool {
    index.checked_sub(1).is_some_and(|previous| {
        matches!(tokens[previous].kind, TokenKind::Word(_))
            && COORDINATORS.contains(
                &document
                    .get_span_content(&tokens[previous].span)
                    .iter()
                    .flat_map(|c| c.to_lowercase())
                    .collect::<String>()
                    .as_str(),
            )
    })
}

/// How a conjugation ending pairs person with number.
///
/// The endings are checked longest first, because `-ten` is an `-en` and `-st`
/// is not a `-t`. Every pair here is a reading the ending genuinely has; the
/// caller has already established from the affix metadata that the word is a
/// finite form at all, which is what keeps a noun ending in `-t` out.
fn readings_of_ending(word: &str) -> Vec<Features> {
    let first_singular = Features::new(PersonSet::FIRST, NumberSet::SINGULAR);
    let third_singular = Features::new(PersonSet::THIRD, NumberSet::SINGULAR);
    let second_singular = Features::new(PersonSet::SECOND, NumberSet::SINGULAR);
    let first_plural = Features::new(PersonSet::FIRST, NumberSet::PLURAL);
    let second_plural = Features::new(PersonSet::SECOND, NumberSet::PLURAL);
    let third_plural = Features::new(PersonSet::THIRD, NumberSet::PLURAL);

    if word.ends_with("st") {
        // du lernst, du arbeitest — and the third person as well, because a
        // verb whose stem ends in a sibilant spells both the same way: *du
        // weist* and *er weist*, *du misst* and *er misst*, *du liest* and *er
        // liest*. Nothing on the surface separates `weis` + `t` from `lern` +
        // `st`, so the third person stays in. The cost is that *er lernst* goes
        // unreported; the gain is the twenty-five reports that `weist`,
        // `verweist`, `misst`, `fasst` and `liest` produced on edited prose.
        vec![second_singular, third_singular]
    } else if word.ends_with("en") {
        // wir/sie lernen, wir/sie lernten — and the infinitive, which has no
        // person at all and is why only the front field is checked.
        vec![first_plural, third_plural]
    } else if ["zt", "ßt", "xt"]
        .iter()
        .any(|ending| word.ends_with(ending))
    {
        // The mirror image of the `-st` case: a stem in `z`, `ß` or `x` takes
        // a bare `-t` for the second person too — *du nutzt*, *du heißt*, *du
        // boxt*.
        vec![second_singular, third_singular, second_plural]
    } else if word.ends_with('t') {
        // er lernt, ihr lernt
        vec![third_singular, second_plural]
    } else if word.ends_with('e') {
        // ich lerne, er lerne (Konjunktiv I), ich/er lernte
        vec![first_singular, third_singular]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::GermanSubjectVerbAgreement;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn lint_count(text: &str) -> usize {
        let dictionary = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dictionary);
        GermanSubjectVerbAgreement::new(dictionary.clone())
            .lint(&document)
            .len()
    }

    /// A noun phrase can be the subject too, and the chunker is what makes the
    /// verb findable: it stands directly behind the phrase, not behind the head
    /// noun.
    #[test]
    fn a_plural_noun_phrase_has_to_agree() {
        for text in [
            "Alle Kinder spielt im Garten.",
            "Viele Menschen ist unzufrieden.",
            "Beide Männer arbeitet dort.",
            "Mehrere Studien zeigt das.",
            "Sämtliche Akten fehlt.",
            "Wenige Leute glaubt das.",
        ] {
            assert_eq!(lint_count(text), 1, "{text}");
        }
    }

    #[test]
    fn a_correct_noun_phrase_subject_is_quiet() {
        for text in [
            "Alle Kinder spielen im Garten.",
            "Viele Menschen sind unzufrieden.",
            "Beide Männer arbeiten dort.",
            "Mehrere Studien zeigen das.",
            "Alle Teilnehmer erhalten eine Urkunde.",
            "Der Hund spielt im Garten.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// The front field holds any one constituent, not only the subject. When
    /// it holds an object the subject follows the verb, and that is what the
    /// rest of the clause is read for.
    #[test]
    fn a_fronted_object_is_not_the_subject() {
        for text in [
            "Eine Rolle spielen auch regionale Unterschiede.",
            "Eine Ausnahme stellen einige Mundarten dar.",
            "Dieser Verfolgung fielen 100.000 Frauen zum Opfer.",
            "Der Flexion stehen die Komparation und die Derivation gegenüber.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// A phrase governed by a preposition cannot be the subject, which is what
    /// keeps a fronted subject reportable although a capitalized noun follows
    /// the verb.
    #[test]
    fn a_governed_phrase_does_not_count_as_a_following_subject() {
        assert_eq!(lint_count("Alle Kinder spielt in dem großen Garten."), 1);
        assert_eq!(lint_count("Viele Gäste wartet vor dem Haus."), 1);
    }

    /// A determiner with no nominative reading opens an object, so what
    /// follows the verb there is not a competing subject.
    #[test]
    fn an_accusative_object_behind_the_verb_is_not_a_subject() {
        assert_eq!(
            lint_count("Mehrere Studien zeigt einen klaren Zusammenhang."),
            1
        );
        assert_eq!(lint_count("Mehrere Länder plant den Ausstieg."), 1);
        assert_eq!(lint_count("Viele Firmen meldet einen Rückgang."), 1);
        // …but an ambiguous one still stops the check, because *die Kommission*
        // really could be the subject of a fronted-object reading.
        assert_eq!(
            lint_count("Beide Vorschläge überzeugt die Kommission nicht."),
            0
        );
    }

    /// Two coordinated noun phrases are one plural subject, however singular
    /// each half is.
    #[test]
    fn a_coordinated_noun_phrase_is_not_checked_alone() {
        for text in [
            "Die Kodierung und das Format hängen von der Art ab.",
            "Andere Tiere und der Mensch sind nicht empfänglich.",
            "Das erste und das letzte Komma fehlen.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// `die` is feminine singular and plural in the nominative, and the noun
    /// cannot break the tie, so those phrases are left alone in both
    /// directions.
    #[test]
    fn an_ambiguous_determiner_is_not_checked() {
        assert_eq!(lint_count("Die Kinder spielt im Garten."), 0);
        assert_eq!(lint_count("Die Bücher ist teuer."), 0);
        assert_eq!(lint_count("Diese Regeln gilt überall."), 0);
    }

    /// The copula lets the predicate carry the number, but only one way round.
    #[test]
    fn the_copula_carries_the_number_in_one_direction_only() {
        assert_eq!(lint_count("Ein weiteres Problem sind die langen Wege."), 0);
        assert_eq!(lint_count("Viele Leute ist unzufrieden."), 1);
    }

    /// Behind a noun phrase, `-en` is the infinitive a modal or a *zu*
    /// governs, and the phrase is its object.
    #[test]
    fn an_infinitive_behind_a_phrase_is_not_its_verb() {
        for text in [
            "Er versucht, ein Glas zu trinken.",
            "Sie wollte eine Pandemie auslösen.",
            "Man darf hier ein Haustier halten.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// A postposed genitive attribute stands where the verb would.
    #[test]
    fn a_postposed_attribute_is_not_the_verb() {
        assert_eq!(lint_count("Der Brennwert reinen Fettes beträgt 39 kJ."), 0);
        assert_eq!(lint_count("Die Arten hohler Stängel sind selten."), 0);
    }

    /// The auxiliaries, which carry most of the German verb system and are the
    /// forms the conjugation affixes cannot build.
    #[test]
    fn an_auxiliary_has_to_match() {
        for text in [
            "Wir ist heute sehr müde.",
            "Du hat das Buch schon gelesen.",
            "Ich sind bereit.",
            "Er haben das Auto gekauft.",
            "Du bin zu spät.",
        ] {
            assert_eq!(lint_count(text), 1, "should fire on {text:?}");
        }
    }

    #[test]
    fn a_correct_auxiliary_is_quiet() {
        for text in [
            "Wir sind heute sehr müde.",
            "Du hast das Buch schon gelesen.",
            "Ich bin bereit.",
            "Er hat das Auto gekauft.",
            "Ihr seid zu spät.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// The regular endings, which come from the conjugation affixes.
    #[test]
    fn a_regular_ending_has_to_match() {
        for text in [
            "Er gehen jeden Tag nach Hause.",
            "Ich lernen Deutsch.",
            "Wir lernt Deutsch.",
            "Du spielen gern Klavier.",
        ] {
            assert_eq!(lint_count(text), 1, "should fire on {text:?}");
        }
    }

    #[test]
    fn a_correct_regular_ending_is_quiet() {
        for text in [
            "Er geht jeden Tag nach Hause.",
            "Ich lerne Deutsch.",
            "Wir lernen Deutsch.",
            "Du spielst gern Klavier.",
            "Sie spielt gern Klavier.",
            "Sie spielen gern Klavier.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// *sie* is third person in both numbers, so neither verb is wrong.
    #[test]
    fn sie_takes_either_number() {
        assert_eq!(lint_count("Sie ist müde."), 0);
        assert_eq!(lint_count("Sie sind müde."), 0);
        assert_eq!(lint_count("Sie bin müde."), 1, "but not the first person");
    }

    /// Konjunktiv I is spelled like the first person, and German prose reports
    /// speech constantly. The `-e` ending allows the third person for this.
    #[test]
    fn reported_speech_is_not_a_disagreement() {
        for text in [
            "Er sagte, er lerne viel.",
            "Sie erklärte, er komme später.",
            "Man sagte, er habe recht.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// Only the front field is checked. In a verb-final clause the word behind
    /// the pronoun is a participle or an infinitive, not the finite verb.
    #[test]
    fn a_verb_final_clause_is_left_alone() {
        for text in [
            "Das ist etwas, das er vergessen hat.",
            "Er blieb zu Hause, weil er gehen musste.",
            "Die Frage, die er stellen wollte, blieb offen.",
            "Ich weiß, dass er kommen wird.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// A coordinated main clause opens a front field of its own.
    #[test]
    fn a_coordinated_clause_is_checked() {
        assert_eq!(lint_count("Sie kam nach Hause und er gehen weg."), 1);
        assert_eq!(lint_count("Sie kam nach Hause und er ging weg."), 0);
    }

    /// The placeholder *es* is out of the table: the verb agrees with the noun
    /// behind it, not with the pronoun.
    /// *Mein Bruder und ich spielen* is a plural subject, not a clause that
    /// starts at *ich*.
    #[test]
    fn a_coordinated_subject_takes_the_plural() {
        for text in [
            "Mein Bruder und ich spielen gern Fußball.",
            "Du und ich gehen morgen ins Kino.",
            "Anna oder er kommen später.",
            "Gestern waren mein Vater und ich im Zoo.",
            "Heute fahren Peter und wir nach Berlin.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// A finite verb in front of the conjunction means the pronoun opens a
    /// clause of its own, so the check stays on.
    #[test]
    fn a_coordinated_clause_is_still_checked_after_a_verb() {
        assert_eq!(
            lint_count("Mein Bruder spielt Fußball und ich spielst Tennis."),
            1
        );
        assert_eq!(lint_count("Anna kam spät, aber er gehen früh."), 1);
    }

    #[test]
    fn the_placeholder_es_is_not_a_subject() {
        for text in [
            "Es werden fünf Klassen gebildet.",
            "Es existieren zahlreiche Ansätze.",
            "Es müssen viele Fragen geklärt werden.",
            "Es ist spät.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// `ihr` is a possessive and a dative far more often than a subject.
    #[test]
    fn ihr_is_not_read_as_a_subject() {
        for text in ["Ihr Auto ist neu.", "Ich gebe ihr Geld.", "Ihr habt recht."] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// An adverb is not the finite verb, whatever the `-st` affix produced.
    #[test]
    fn an_adverb_behind_the_pronoun_is_not_a_verb() {
        for text in [
            "Er selbst sprach von einem Paradigma.",
            "Sie selbst stellen eine wichtige Nahrung dar.",
            "Man selbst ist dafür verantwortlich.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// An oblique pronoun is not a subject, so nothing is compared.
    #[test]
    fn an_oblique_pronoun_starts_nothing() {
        for text in ["Ihn kennen wir gut.", "Mir ist kalt.", "Uns geht es gut."] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// The modals, which are the other half of the irregular table.
    #[test]
    fn the_modals_are_covered() {
        assert_eq!(lint_count("Du kann das nicht wissen."), 1);
        assert_eq!(lint_count("Du kannst das nicht wissen."), 0);
        assert_eq!(lint_count("Wir muss jetzt gehen."), 1);
        assert_eq!(lint_count("Wir müssen jetzt gehen."), 0);
    }

    /// The preterite splits by number the same way the present does.
    #[test]
    fn the_preterite_is_checked() {
        assert_eq!(lint_count("Wir lernte Deutsch."), 1);
        assert_eq!(lint_count("Wir lernten Deutsch."), 0);
        assert_eq!(lint_count("Ich lernten Deutsch."), 1);
        assert_eq!(lint_count("Ich lernte Deutsch."), 0);
    }

    /// The readings are joint pairs, not two independent sets, so the `-t`
    /// ending's two readings exclude the second person singular between them.
    #[test]
    fn the_endings_readings_are_joint() {
        assert_eq!(lint_count("Du lernt Deutsch."), 1, "-t is 3rd sg or 2nd pl");
        assert_eq!(lint_count("Ihr lernt Deutsch."), 0);
        assert_eq!(lint_count("Er lernt Deutsch."), 0);
    }

    /// A stem ending in a sibilant spells the second and third person alike,
    /// and nothing on the surface separates `weis` + `t` from `lern` + `st`.
    #[test]
    fn a_sibilant_stem_keeps_both_persons() {
        for text in [
            "Er weist darauf hin.",
            "Sie misst die Strecke.",
            "Er liest ein Buch.",
            "Du liest ein Buch.",
            "Du nutzt das Portal.",
            "Du heißt Anna.",
            "Wie du das Portal nutzt, ist egal.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
        assert_eq!(
            lint_count("Wir lernst Deutsch."),
            1,
            "the number still separates them"
        );
    }

    /// A word that is not in either table produces nothing at all.
    #[test]
    fn an_unknown_verb_is_not_guessed_at() {
        assert_eq!(lint_count("Er fnordet durch die Gegend."), 0);
    }

    /// An apposition behind the pronoun is a noun, and a capital says so.
    #[test]
    fn an_apposition_is_not_a_verb() {
        for text in [
            "Wir Deutsche reden gern über das Wetter.",
            "Wir Arbeiter haben andere Sorgen.",
            "Wir Frauen wissen das.",
        ] {
            assert_eq!(lint_count(text), 0, "should stay quiet on {text:?}");
        }
    }

    /// The subordinate clause is verb-final, which is the second position
    /// where the finite verb can be found by counting rather than parsing.
    #[test]
    fn a_verb_final_clause_has_to_agree() {
        for text in [
            "Ich hoffe, dass du kommen sollt.",
            "Er sagte, dass wir müde bist.",
            "Sie weiß, dass ich zu spät bist.",
            "Er fragt, ob du das Buch habt.",
            "Das gilt, weil du zu jung bin.",
            "Man merkt es, sobald wir bereit bist.",
        ] {
            assert_eq!(lint_count(text), 1, "{text}");
        }
    }

    /// `es` is admitted behind a conjunction and nowhere else: the placeholder
    /// that made it unusable in the front field needs a front field to stand
    /// in.
    #[test]
    fn the_placeholder_is_a_real_pronoun_in_a_subordinate_clause() {
        assert_eq!(lint_count("Es ist nicht so wie es sein sollt."), 1);
        assert_eq!(lint_count("Es ist gut, weil es so gewesen wart."), 1);
        assert_eq!(lint_count("Ich glaube, dass es genug seid."), 1);
    }

    /// …and its number still comes from the predicate, so only the person is
    /// checked. *weil es meine Freunde sind* is correct German.
    #[test]
    fn the_placeholder_takes_its_number_from_the_predicate() {
        for text in [
            "Es ist so, weil es meine Freunde sind.",
            "Ich weiß, dass es viele Möglichkeiten gibt.",
            "Das stimmt, weil es zwei Gründe waren.",
            "Er sagt, dass es fünf Klassen sind.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// A correct verb-final clause stays quiet whatever sits between the
    /// subject and the verb.
    #[test]
    fn a_correct_verb_final_clause_is_left_alone() {
        for text in [
            "Es ist nicht so, wie es sein sollte.",
            "Er meint, dass du kommen kannst.",
            "Sie sagt, dass wir gegangen sind.",
            "Ich weiß, dass er das Buch gelesen hat.",
            "Das geht, solange du vorsichtig bist.",
            "Er wartet, bevor ich die Tür öffne.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// `-en` clause-finally is the infinitive as often as it is a finite verb,
    /// and an adjective besides. Every one of these ends on one.
    #[test]
    fn an_en_ending_is_never_trusted_at_the_end_of_a_clause() {
        for text in [
            "Er wird gereizt, damit er nicht mehr richtig urteilen kann.",
            "Gabriel singt davon, dass er versucht zu schlafen.",
            "Das ist so, damit diese erkennen und wie er handeln können.",
            "Ich bin kein Idealist, da ich nicht so weit gehe zu behaupten.",
            "Es ist bekannt, dass man das Wissen erarbeiten muss.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// An attributive adjective can end a scan when a coordinator cuts the
    /// clause short. It is not the verb.
    #[test]
    fn an_adjective_is_not_the_verb_of_the_clause() {
        for text in [
            "Es hängt davon ab, ob wir eine positive oder negative Wirkung erwarten.",
            "Da er jede Theorie in einem konkreten, empirischen Zusammenhang sieht, gilt das.",
            "Man nimmt an, dass er dem frühen 9. Jahrhundert entstammt.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// A coordinator starts a clause that has a subject of its own, so the
    /// word at the end of the sentence is not this subject's verb.
    #[test]
    fn a_coordinated_clause_belongs_to_its_own_subject() {
        for text in [
            "Ich sage, dass ich keine Beweise hätte und der Dieb es nicht zurückgibt.",
            "Es gilt, sobald wir einen anderen sehen oder uns sein Leiden geschildert wird.",
            "Er weiß, dass wir bereit sind und du später kommst.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// A question word is spelled like a conjunction, but puts the verb where
    /// the check expects the subject, so it never matches.
    #[test]
    fn a_question_is_not_a_subordinate_clause() {
        for text in [
            "Wie geht es dir?",
            "Wo bist du gewesen?",
            "Er ist größer wie ich.",
        ] {
            assert_eq!(lint_count(text), 0, "{text}");
        }
    }

    /// The clause ends at its own punctuation; a word from the next one is not
    /// reached.
    #[test]
    fn the_scan_stops_at_the_end_of_the_clause() {
        assert_eq!(lint_count("Weil du das machst, geht es."), 0);
        assert_eq!(lint_count("Obwohl wir müde sind, bleibt er wach."), 0);
        assert_eq!(lint_count("Wenn er kommt, sind wir bereit."), 0);
    }
}
