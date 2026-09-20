//! The words German derives, and the word list did not have.
//!
//! Every gap here showed up the same way: a compound with the missing word at
//! the end was reported as a misspelling. `Achslastverschiebung` failed because
//! `Verschiebung` was not a word, `Arbeitsverhältnissen` because `Verhältnisse`
//! was not, `aktualisiertes` and `bearbeitbaren` because the participle and the
//! `-bar` adjective had no affix class at all.
//!
//! Three repairs, all decided per entry by `hunspell -m`:
//!
//! * `-nis` doubles its `s` before an ending (`X`, `Y` and `0` grew a rule),
//!   see `scripts/fix_german_noun_forms.py --matching 'nis$'`.
//! * `-ung` nouns, the declined past participle (`n`) and the `-bar` adjective
//!   (`x`) come from `scripts/add_german_verb_derivations.py`.
//! * The genitive `-s` from `scripts/fix_german_noun_forms.py --flags H`.

#[cfg(test)]
mod tests {
    use crate::language::german::spell::curated_german_dictionary;
    use crate::spell::Dictionary;

    fn known(word: &str) -> bool {
        curated_german_dictionary()
            .get_word_metadata_str(word)
            .is_some()
    }

    /// A noun in `-nis` doubles the `s`: `das Ergebnis`, `die Ergebnisse`.
    #[test]
    fn nis_nouns_double_their_s() {
        for word in [
            "Ergebnisse",
            "Ergebnissen",
            "Ergebnisses",
            "Verhältnisse",
            "Verständnisses",
            "Bekenntnissen",
        ] {
            assert!(known(word), "'{word}' is a form of a -nis noun");
        }
    }

    /// And the undoubled form is not a word, which is what the three spelled-out
    /// conditions on `X`, `Y` and `0` are for.
    #[test]
    fn the_undoubled_nis_form_is_not_a_word() {
        for word in ["Ergebnise", "Ergebnisen", "Ergebnises", "Verhältnise"] {
            assert!(!known(word), "'{word}' is not German");
        }
    }

    /// The genitive of the nouns that do not end in `-nis` is untouched.
    #[test]
    fn ordinary_s_nouns_keep_their_endings() {
        for word in ["Hauses", "Kreise", "Häuser", "Glases"] {
            assert!(known(word), "'{word}' must survive the -nis rule");
        }
    }

    /// The `-ung` noun of a verb.
    #[test]
    fn verbs_have_their_ung_noun() {
        for word in [
            "Verschiebung",
            "Verschiebungen",
            "Absperrung",
            "Abstimmungen",
        ] {
            assert!(known(word), "'{word}' is an -ung noun");
        }
    }

    /// And the compounds built on it, which need the decomposition and so the
    /// compound-aware dictionary rather than the base one.
    #[test]
    fn compounds_headed_by_an_ung_noun_are_words() {
        use crate::language::german::spell::combined_german_dictionary;

        let dictionary = combined_german_dictionary();
        for word in [
            "Achslastverschiebung",
            "Bedeutungsverschiebungen",
            "Blattverschiebung",
            "Arbeitsverhältnissen",
            "Abhängigkeitsverhältnisse",
        ] {
            assert!(
                dictionary.contains_word(&word.chars().collect::<Vec<_>>()),
                "'{word}' is a compound of a derived word"
            );
        }
    }

    /// The suffix is not productive for every verb, so the oracle's refusals
    /// have to hold.
    #[test]
    fn the_ung_noun_is_not_invented() {
        for word in ["Wissung", "Gehung", "Sagung"] {
            assert!(!known(word), "'{word}' is not German");
        }
    }

    /// The declined past participle, flag `n`.
    #[test]
    fn past_participles_decline() {
        for word in [
            "automatisierter",
            "automatisiertes",
            "aktualisiertes",
            "beobachtetem",
        ] {
            assert!(known(word), "'{word}' is a declined past participle");
        }
    }

    /// The `-bar` adjective, flag `x`.
    #[test]
    fn verbs_have_their_bar_adjective() {
        for word in [
            "beeinflussbar",
            "bearbeitbaren",
            "adressierbarer",
            "lesbare",
        ] {
            assert!(known(word), "'{word}' is a -bar adjective");
        }
    }

    /// The `-chen` diminutive, flag `z`, including the stems that umlaut.
    #[test]
    fn nouns_have_their_diminutive() {
        for word in [
            "Röhrchen",
            "Körperchen",
            "Körperchens",
            "Fässchen",
            "Küsschen",
        ] {
            assert!(known(word), "'{word}' is a diminutive");
        }
    }

    /// An adjective declines, and the `-el` and `-uer` endings contract when
    /// it does. The single `.` rule this replaced built the forms on the right.
    #[test]
    fn adjectives_decline_with_their_syncope() {
        for word in [
            "akzeptable",
            "akzeptabler",
            "akzeptables",
            "irreversibler",
            "edler",
            "teure",
            "böses",
            "helle",
            "klare",
        ] {
            assert!(known(word), "'{word}' is a declined adjective");
        }

        for word in ["akzeptabele", "teuere", "irreversibeles"] {
            assert!(!known(word), "'{word}' is not German");
        }
    }

    /// The adjectives that hunspell declines and Harper could not: the entries
    /// were there, the declension flags were not.
    #[test]
    fn corpus_mined_adjectives_decline_too() {
        for word in [
            "abundanter",
            "abwechslungsarmes",
            "berufener",
            "achtmonatiger",
        ] {
            assert!(known(word), "'{word}' is a declined adjective");
        }
    }

    /// The plurals that igerman98 lists as headwords of their own, which the
    /// strict form of the oracle refused for every `-tät`, `-ion` and `-keit`.
    #[test]
    fn derived_feminines_have_their_plural() {
        for word in [
            "Quantitäten",
            "Kontinuitäten",
            "Komorbiditäten",
            "Gesetzlichkeiten",
            "Vergangenheiten",
            "Adaptationen",
            "Appositionen",
        ] {
            assert!(known(word), "'{word}' is the plural of a derived feminine");
        }
    }
}
