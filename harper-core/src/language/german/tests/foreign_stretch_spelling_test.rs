//! Spelling inside a stretch of another language.
//!
//! German academic prose quotes constantly and mostly not in German: a book
//! title, a journal name, a line of a bibliography. A German dictionary has
//! nothing useful to say about any of it, and `GermanSpellCheck` used to report
//! every word of it. On edited German prose that was the single largest source
//! of wrong reports — larger than every other German rule put together.
//!
//! `german_foreign_stretch` already existed for the capitalization rules and
//! asks for the same evidence here: function words no German sentence contains,
//! counted in a window of word tokens either side.
//!
//! The price is named in `a_german_typo_inside_an_english_run_is_missed`: a
//! German misspelling that happens to stand among English words is no longer
//! reported. That is the right way round — the reports removed are on prose
//! people actually write, the missed ones need a German typo inside an English
//! sentence.

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::linting::new_curated_german;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::curated_german_dictionary;
    use crate::linting::{LintKind, Linter};

    fn misspelled(text: &str) -> Vec<String> {
        let dict = curated_german_dictionary();
        let mut linter = new_curated_german(GermanDialect::Standard, dict.clone());
        let document = Document::new(text, &PlainGerman, &dict);
        linter
            .lint(&document)
            .into_iter()
            .filter(|lint| lint.lint_kind == LintKind::Spelling)
            .map(|lint| document.get_span_content_str(&lint.span))
            .collect()
    }

    /// The shape that dominates a German bibliography.
    #[test]
    fn an_english_title_is_not_spell_checked() {
        let reported = misspelled("Edward Sapir: An introduction to the study of speech, 1921.");

        assert!(
            reported.is_empty(),
            "an English title is not German; reported: {reported:?}"
        );
    }

    /// A second one, with the publisher's English in it.
    #[test]
    fn an_english_citation_is_not_spell_checked() {
        let reported = misspelled(
            "Bruce Alberts: Molecular biology of the cell, in the fifth edition of that work.",
        );

        assert!(
            reported.is_empty(),
            "an English citation is not German; reported: {reported:?}"
        );
    }

    /// French counts too — the list is not English-only.
    #[test]
    fn a_french_title_is_not_spell_checked() {
        let reported = misspelled(
            "Georg Bossong: Le marquage différentiel de l'objet dans les langues d'Europe.",
        );

        assert!(
            reported.is_empty(),
            "a French title is not German; reported: {reported:?}"
        );
    }

    /// An English clause inside a German sentence.
    #[test]
    fn an_english_clause_inside_german_is_not_spell_checked() {
        let reported =
            misspelled("Der Autor schrieb over the years about the history of the Zelle.");

        assert!(
            reported.is_empty(),
            "the English run is not German; reported: {reported:?}"
        );
    }

    /// The whole point: a German misspelling next to an English run is still
    /// reported, as long as it stands outside the run.
    #[test]
    fn a_german_typo_beside_an_english_run_is_still_reported() {
        let reported = misspelled("Er las die Studie of the Pacific und fand darin einen Fehlr.");

        assert!(
            reported.iter().any(|w| w == "Fehlr"),
            "'Fehlr' is a German misspelling; reported: {reported:?}"
        );
        assert!(
            !reported.iter().any(|w| w == "Pacific"),
            "'Pacific' is inside the English run; reported: {reported:?}"
        );
    }

    /// The price, stated rather than hidden: inside the run the rule is silent,
    /// German typo or not.
    #[test]
    fn a_german_typo_inside_an_english_run_is_missed() {
        let reported =
            misspelled("Die Studie of the cell in the year of the report nennt Aspekkte.");

        assert!(
            reported.is_empty(),
            "known limitation: inside the run nothing is checked; reported: {reported:?}"
        );
    }

    /// An ordinary German sentence is untouched.
    #[test]
    fn an_ordinary_german_typo_is_reported() {
        let reported = misspelled("Er schreibt einen Breif an seine Mutter.");

        assert!(
            reported.iter().any(|w| w == "Breif"),
            "'Breif' is a misspelling of 'Brief'; reported: {reported:?}"
        );
    }

    /// And a second one, with no foreign word anywhere near it.
    #[test]
    fn a_german_typo_in_a_long_german_sentence_is_reported() {
        let reported = misspelled("Der Breif kam gestern an und lag auf dem Tisch im Flur.");

        assert!(
            reported.iter().any(|w| w == "Breif"),
            "'Breif' is a misspelling of 'Brief'; reported: {reported:?}"
        );
    }

    /// A Latin borrowing in a German sentence has no foreign function words
    /// around it and stays reported.
    #[test]
    fn a_lone_foreign_word_in_german_is_still_reported() {
        let reported = misspelled("Durch die Gabe von Reserpin kommt es zu einer Verminderung.");

        assert!(
            reported.iter().any(|w| w == "Reserpin"),
            "one unknown word is not a foreign stretch; reported: {reported:?}"
        );
    }

    /// Correct German is silent, which is what the whole rule is for.
    #[test]
    fn correct_german_reports_nothing() {
        let reported = misspelled("Die Untersuchung der Zelle ergab ein klares Bild.");

        assert!(reported.is_empty(), "reported: {reported:?}");
    }

    /// A German determiner in front settles it the other way: the sentence is
    /// German and merely names something foreign.
    #[test]
    fn a_word_behind_a_german_determiner_is_still_checked() {
        let reported = misspelled("Er kaufte die Zeitung The Guardian in Berlin.");

        assert!(
            reported.iter().any(|w| w == "Guardian"),
            "'die Zeitung The Guardian' is a German sentence; reported: {reported:?}"
        );
    }

    /// One foreign function word is not a stretch.
    #[test]
    fn a_single_foreign_function_word_is_not_enough() {
        let reported = misspelled("Der Titel lautet The Origin of Species.");

        assert!(
            !reported.is_empty(),
            "two words of English do not silence the rule; reported: {reported:?}"
        );
    }

    /// A misspelling at the very start of a German sentence.
    #[test]
    fn a_typo_early_in_the_sentence_is_reported() {
        let reported = misspelled("Das Manuscript lag seit Wochen auf dem Schreibtisch.");

        assert!(
            reported.iter().any(|w| w == "Manuscript"),
            "'Manuscript' is a misspelling of 'Manuskript'; reported: {reported:?}"
        );
    }

    /// German with an English book title in the middle: the German half keeps
    /// its spell check.
    #[test]
    fn german_around_an_english_title_keeps_its_spell_check() {
        let reported = misspelled(
            "Das Buch erschien in the series of the Cambridge studies of language. Es gilt als ein Fehlr.",
        );

        assert!(
            reported.iter().any(|w| w == "Fehlr"),
            "the German sentence after it is still checked; reported: {reported:?}"
        );
    }

    /// An English run split across a colon still counts as one.
    #[test]
    fn an_english_run_after_a_colon_is_one_stretch() {
        let reported =
            misspelled("Vergleiche dazu: the relationship between the language and the thought.");

        assert!(
            reported.is_empty(),
            "punctuation must not eat the window; reported: {reported:?}"
        );
    }

    /// A German sentence full of proper names the dictionary does not know is
    /// not a foreign stretch — unknown words alone never were enough.
    #[test]
    fn unknown_german_names_are_not_a_foreign_stretch() {
        let reported =
            misspelled("Der Physiker Grubbenkamp erklärte dem Kollegen Vosswinkel den Vorgangg.");

        assert!(
            reported.iter().any(|w| w == "Vorgangg"),
            "unknown names are not evidence of another language; reported: {reported:?}"
        );
    }
}
