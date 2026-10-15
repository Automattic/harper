//! The five adjective declension flags `OQRST` belong on the base form, which is
//! where `kleine`, `kleinem`, `kleinen`, `kleiner` and `kleines` come from. Some
//! entries were themselves one of those five and carried the flags anyway, so
//! they declined a second time and put `kleineree`, `vielee` and a couple of
//! hundred more non-words into the dictionary — each one a plausible typo of the
//! real form.
//!
//! Nine of them had lost a `#` and swallowed their own comment:
//! `höherer/~~Jq - comprtve jectveOQRST`. Since `c`, `e`, `j`, `m`, `o`, `p`,
//! `r`, `t` and `v` are all real flags, a comparative adjective read as a noun, a
//! verb in two tenses, and an adverb.
//!
//! `harper-core/src/language/german/scripts/fix_german_double_declension.py` took the flags off wherever the
//! forms survive elsewhere, moving the readings they carried onto `J` and `r`,
//! which generate nothing.

#[cfg(test)]
mod tests {
    use crate::language::german::spell::curated_german_dictionary;
    use crate::spell::Dictionary;

    /// The comparatives are still words, and the base declension still works.
    #[test]
    fn declined_comparatives_survive() {
        let dict = curated_german_dictionary();

        for word in [
            "höherer",
            "höhere",
            "höherem",
            "höheres",
            "besserer",
            "kürzerer",
            "längerer",
            "stärkerer",
            "größerer",
            "kleinerer",
            "breiterer",
            "minderer",
            "kleine",
            "kleinem",
            "kleinen",
            "kleines",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is a real German form and must stay in the dictionary"
            );
        }
    }

    /// Declining an already-declined form is not a word.
    #[test]
    fn double_declensions_are_gone() {
        let dict = curated_german_dictionary();

        for word in [
            "höherere",
            "höhererem",
            "vielee",
            "achtmonatigee",
            "validee",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_none(),
                "'{word}' declines an already-declined form and is not a word"
            );
        }
    }

    /// The nine mangled entries read as a single part of speech again.
    #[test]
    fn mangled_comparatives_are_adjectives_only() {
        let dict = curated_german_dictionary();

        for word in [
            "höherer",
            "besserer",
            "kürzerer",
            "längerer",
            "stärkerer",
            "minderer",
        ] {
            let meta = dict.get_word_metadata_str(word).unwrap();
            assert!(
                meta.is_adjective(),
                "'{word}' is a comparative adjective and should read as one"
            );
            assert!(
                !meta.is_noun() && !meta.is_verb(),
                "'{word}' picked up its own comment as flags; it is not a noun or a verb"
            );
        }
    }

    /// Taking `OQRST` off must not take the part of speech with it: `O`, `Q`, `S`
    /// and `T` carry an adjective reading and `R` an adverb one, so `J` and `r`
    /// have to stand in wherever they were removed.
    #[test]
    fn readings_moved_rather_than_lost() {
        let dict = curated_german_dictionary();

        for word in ["abstrakter", "absurder", "abhängiger"] {
            let meta = dict.get_word_metadata_str(word).unwrap();
            assert!(
                meta.is_adjective(),
                "'{word}' lost the adjective reading its declension flags carried"
            );
        }
    }
}
