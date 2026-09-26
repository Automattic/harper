//! The German determiner paradigms.
//!
//! Determiners are a closed class of a few hundred forms, so they are written
//! out here rather than derived from `dictionary.dict`. Two reasons:
//!
//! * A dictionary entry carries an [`Agreement`], whose three axes are
//!   independent sets. That cannot express a determiner: *der* is nominative
//!   masculine singular, dative feminine singular, genitive feminine singular
//!   and genitive plural, but it is never *nominative feminine*. Storing
//!   `case = {NOM, DAT, GEN}` beside `gender = {M, F}` would admit exactly that
//!   combination. What is needed is a set of fully specified readings — the
//!   same shape LanguageTool intersects with `retainAll`.
//! * The paradigms are regular. Spelling out sixteen endings twice and naming
//!   the stems is shorter, and far easier to check, than 200 dictionary lines.

use hashbrown::HashMap;
use std::sync::LazyLock;

use crate::language::morphology::{
    Agreement, Case, CaseSet, Gender, GenderSet, Number, NumberSet, PersonSet,
};

/// One fully specified reading of a determiner form.
///
/// `gender` is `None` in the plural, where German draws no gender distinction —
/// that is a genuine absence, not an unknown, which is why it is not a
/// [`crate::language::morphology::GenderSet`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeterminerReading {
    pub case: Case,
    pub gender: Option<Gender>,
    /// Which paradigm produced this reading. A correction has to come from the
    /// same one: the fix for *wegen dem* is *wegen des*, never *wegen eines*.
    pub paradigm: Paradigm,
}

impl DeterminerReading {
    pub fn number(&self) -> Number {
        match self.gender {
            Some(_) => Number::Singular,
            None => Number::Plural,
        }
    }

    /// The reading as an [`Agreement`], for comparison with a dictionary entry.
    ///
    /// A plural reading gets every gender rather than none: German draws no
    /// gender distinction in the plural, so *die Männer* and *die Frauen* are
    /// the same form, and an empty set would read as "unknown" instead.
    pub fn agreement(&self) -> Agreement {
        Agreement {
            case: self.case.into(),
            gender: self.gender.map(GenderSet::from).unwrap_or(GenderSet::all()),
            number: self.number().into(),
            // A determiner has no person of its own; the noun phrase it heads
            // is third person, and the verb rule reads that from the phrase
            // rather than from here.
            person: PersonSet::empty(),
        }
    }
}

/// The inflection class a form belongs to, used to keep corrections inside one
/// paradigm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Paradigm(&'static str);

impl Paradigm {
    /// The dictionary form the paradigm is named after.
    pub fn stem(&self) -> &'static str {
        self.0
    }
}

/// A cell of a paradigm table: which case and gender an ending realizes.
/// `None` for gender is the plural column.
type Cell = (Case, Option<Gender>, &'static str);

const M: Option<Gender> = Some(Gender::Masculine);
const F: Option<Gender> = Some(Gender::Feminine);
const N: Option<Gender> = Some(Gender::Neuter);
const PL: Option<Gender> = None;

/// The strong (*der*-word) endings: *dieser, diese, dieses, diesem, diesen…*
const STRONG: &[Cell] = &[
    (Case::Nominative, M, "er"),
    (Case::Accusative, M, "en"),
    (Case::Dative, M, "em"),
    (Case::Genitive, M, "es"),
    (Case::Nominative, F, "e"),
    (Case::Accusative, F, "e"),
    (Case::Dative, F, "er"),
    (Case::Genitive, F, "er"),
    (Case::Nominative, N, "es"),
    (Case::Accusative, N, "es"),
    (Case::Dative, N, "em"),
    (Case::Genitive, N, "es"),
    (Case::Nominative, PL, "e"),
    (Case::Accusative, PL, "e"),
    (Case::Dative, PL, "en"),
    (Case::Genitive, PL, "er"),
];

/// The mixed (*ein*-word) endings. They differ from [`STRONG`] in exactly the
/// three cells where the determiner carries no ending at all — which is why
/// *ein Mann* and *ein Kind* look alike while *dieser Mann* and *dieses Kind*
/// do not.
const MIXED: &[Cell] = &[
    (Case::Nominative, M, ""),
    (Case::Accusative, M, "en"),
    (Case::Dative, M, "em"),
    (Case::Genitive, M, "es"),
    (Case::Nominative, F, "e"),
    (Case::Accusative, F, "e"),
    (Case::Dative, F, "er"),
    (Case::Genitive, F, "er"),
    (Case::Nominative, N, ""),
    (Case::Accusative, N, ""),
    (Case::Dative, N, "em"),
    (Case::Genitive, N, "es"),
    (Case::Nominative, PL, "e"),
    (Case::Accusative, PL, "e"),
    (Case::Dative, PL, "en"),
    (Case::Genitive, PL, "er"),
];

/// A regular paradigm to expand.
struct Regular {
    /// The name of the paradigm, and the stem the endings attach to.
    stem: &'static str,
    /// The unsuffixed form, when it is safe to claim. `None` drops the three
    /// endingless cells of [`MIXED`]: bare *ihr* is overwhelmingly the personal
    /// pronoun (*mit ihr*, dative) rather than the possessive, and reading it
    /// as a nominative determiner would flag every *mit ihr* as a case error.
    bare: Option<&'static str>,
    endings: &'static [Cell],
    /// *ein* has no plural. Generating one would give *einen* a dative plural
    /// reading and silence *mit einen Freund*, the commonest case error there
    /// is.
    has_plural: bool,
}

const REGULARS: &[Regular] = &[
    // ein-words.
    Regular {
        stem: "ein",
        bare: Some("ein"),
        endings: MIXED,
        has_plural: false,
    },
    Regular {
        stem: "kein",
        bare: Some("kein"),
        endings: MIXED,
        has_plural: true,
    },
    Regular {
        stem: "mein",
        bare: Some("mein"),
        endings: MIXED,
        has_plural: true,
    },
    Regular {
        stem: "dein",
        bare: Some("dein"),
        endings: MIXED,
        has_plural: true,
    },
    // Bare *sein* is the infinitive *to be*, which follows *zu* constantly:
    // *zu sein* would otherwise be a nominative after a dative preposition.
    Regular {
        stem: "sein",
        bare: None,
        endings: MIXED,
        has_plural: true,
    },
    Regular {
        stem: "ihr",
        bare: None,
        endings: MIXED,
        has_plural: true,
    },
    Regular {
        stem: "unser",
        bare: Some("unser"),
        endings: MIXED,
        has_plural: true,
    },
    // *euer* loses the stem `e` as soon as an ending follows: euer, but eure.
    Regular {
        stem: "eur",
        bare: Some("euer"),
        endings: MIXED,
        has_plural: true,
    },
    // der-words. None has an endingless cell, so `bare` stays `None`
    // throughout: *manch ein Mann* leaves *manch* uninflected and outside the
    // paradigm.
    Regular {
        stem: "dies",
        bare: None,
        endings: STRONG,
        has_plural: true,
    },
    Regular {
        stem: "jen",
        bare: None,
        endings: STRONG,
        has_plural: true,
    },
    Regular {
        stem: "jed",
        bare: None,
        endings: STRONG,
        has_plural: false,
    },
    Regular {
        stem: "jeglich",
        bare: None,
        endings: STRONG,
        has_plural: false,
    },
    Regular {
        stem: "welch",
        bare: None,
        endings: STRONG,
        has_plural: true,
    },
    Regular {
        stem: "manch",
        bare: None,
        endings: STRONG,
        has_plural: true,
    },
    Regular {
        stem: "solch",
        bare: None,
        endings: STRONG,
        has_plural: true,
    },
    Regular {
        stem: "sämtlich",
        bare: None,
        endings: STRONG,
        has_plural: true,
    },
    Regular {
        stem: "all",
        bare: None,
        endings: STRONG,
        has_plural: true,
    },
];

/// A form of the definite article and the case/gender pairs it realizes.
type DefiniteForm = (&'static str, &'static [(Case, Option<Gender>)]);

/// The definite article, which is too irregular to generate. Read across: each
/// form is followed by every reading it has.
const DEFINITE: &[DefiniteForm] = &[
    (
        "der",
        &[
            (Case::Nominative, M),
            (Case::Dative, F),
            (Case::Genitive, F),
            (Case::Genitive, PL),
        ],
    ),
    (
        "die",
        &[
            (Case::Nominative, F),
            (Case::Accusative, F),
            (Case::Nominative, PL),
            (Case::Accusative, PL),
        ],
    ),
    ("das", &[(Case::Nominative, N), (Case::Accusative, N)]),
    ("den", &[(Case::Accusative, M), (Case::Dative, PL)]),
    ("dem", &[(Case::Dative, M), (Case::Dative, N)]),
    ("des", &[(Case::Genitive, M), (Case::Genitive, N)]),
    // The dative plural of the demonstrative/relative pronoun. Included because
    // it is unambiguous and shares the article's paradigm, so *für denen* can
    // be corrected to *für die*.
    ("denen", &[(Case::Dative, PL)]),
];

/// The paradigm of the definite article.
pub const DEFINITE_ARTICLE: Paradigm = Paradigm("der");

/// Forms that stand in for a noun phrase instead of introducing one. They are
/// listed so their case can be read, but they are never offered as a
/// correction: *mit die Leute* becomes *mit den Leuten*, not *mit denen*.
const PRONOUN_ONLY: &[&str] = &["denen"];

// A `LazyLock` in a `static` hands out `&'static` references, so the owned
// `String` keys can be borrowed for the lifetime of the process and the lookups
// stay allocation-free.
static FORMS: LazyLock<HashMap<String, Vec<DeterminerReading>>> = LazyLock::new(|| {
    let mut forms: HashMap<String, Vec<DeterminerReading>> = HashMap::new();

    let mut push = |word: String, reading: DeterminerReading| {
        let readings = forms.entry(word).or_default();
        if !readings.contains(&reading) {
            readings.push(reading);
        }
    };

    for (word, readings) in DEFINITE {
        for (case, gender) in *readings {
            push(
                (*word).to_string(),
                DeterminerReading {
                    case: *case,
                    gender: *gender,
                    paradigm: DEFINITE_ARTICLE,
                },
            );
        }
    }

    for regular in REGULARS {
        let paradigm = Paradigm(regular.stem);
        for (case, gender, ending) in regular.endings {
            if gender.is_none() && !regular.has_plural {
                continue;
            }

            let word = if ending.is_empty() {
                match regular.bare {
                    Some(bare) => bare.to_string(),
                    None => continue,
                }
            } else {
                format!("{}{ending}", regular.stem)
            };

            push(
                word,
                DeterminerReading {
                    case: *case,
                    gender: *gender,
                    paradigm,
                },
            );
        }
    }

    forms
});

/// Every reading of `word` as a determiner, or `None` if it is not one.
///
/// Matching is case-insensitive, so a sentence-initial *Der* is found.
pub fn determiner_readings(word: &str) -> Option<&'static [DeterminerReading]> {
    FORMS
        .get(word.to_lowercase().as_str())
        .map(|readings| readings.as_slice())
}

/// The cases `word` can be read in as a determiner.
pub fn determiner_cases(word: &str) -> Option<CaseSet> {
    determiner_readings(word).map(|readings| {
        readings
            .iter()
            .fold(CaseSet::empty(), |set, reading| set | reading.case.into())
    })
}

/// The forms of the same paradigm that keep `word`'s gender and number but land
/// in one of `wanted`.
///
/// This is what turns a detected mismatch into a correction: *dem* is dative
/// masculine or neuter singular, so in the genitive it becomes *des* — and only
/// *des*, because the gender and number have to survive the change.
pub fn forms_in_case(word: &str, wanted: CaseSet) -> Vec<&'static str> {
    match determiner_readings(word) {
        Some(readings) => forms_for_readings(readings, wanted),
        None => Vec::new(),
    }
}

/// The readings of a determiner that the noun after it allows.
///
/// This is the step that makes the noun worth reading at all. *den* is
/// accusative masculine singular or dative plural, and nothing about the word
/// itself says which; *Freund* is a singular, so the dative plural reading
/// cannot stand and *mit den Freund* has no dative left.
///
/// **Only a singular noun narrows anything, and only the number is read.** Both
/// restrictions were forced by measurement, and each has a reason:
///
/// * *Gender is not trustworthy.* `Leber`, `Mauer`, `Dauer`, `Nummer` and
///   `Schulter` are all feminine and all recorded masculine, and `Tier` and
///   `Heer` are neuter and recorded masculine. Narrowing by gender turned
///   *"in der Leber"* into an error. Reading gender here again is the last step
///   of the gender audit, not the first.
/// * *A plural marking does not rule out the singular.* German weak masculines
///   — `Mensch`, `Philosoph`, `Patient`, `Laie`, `Gedanke` — spell the
///   accusative, dative and genitive singular exactly like the plural, and the
///   dictionary records `Menschen` as a plural only. Narrowing by it reported
///   *"für den Menschen"*, 44 times in one corpus.
///
/// A singular marking has neither problem: it comes from a base entry, which is
/// a nominative singular by construction.
///
/// The noun's *spelling* says more than its entry does; see
/// [`readings_allowed_by_spelling`].
pub fn readings_allowed_by(
    readings: &[DeterminerReading],
    noun: &Agreement,
) -> Vec<DeterminerReading> {
    if noun.number != NumberSet::SINGULAR {
        return readings.to_vec();
    }

    keep(readings, |reading| reading.number() == Number::Singular)
}

/// The readings that survive the **spelling** of the noun after them.
///
/// German has one inflectional ending left that is exceptionless: the dative
/// plural takes `-n`. *den Freunden*, *den Kindern*, *den Lehrern* — every
/// dative plural in the language ends in `-n`, and the only nouns exempt are
/// those whose plural is `-s` (*den Autos*) or a Latin or Greek form (*den
/// Korpora*, *den Termini*).
///
/// So a noun that ends in none of those cannot be a dative plural, whatever the
/// dictionary does or does not know about it. That is what makes *mit den
/// Freund*, *mit den Lehrer* and *bei den Bäcker* reportable: `den` is
/// accusative singular or dative plural, the noun rules the second out, and
/// *mit* wants a dative. No entry has to carry a number, a gender or a case —
/// which is as well, since `bruder` and `zug` carry none of the three.
pub fn readings_allowed_by_spelling(
    readings: &[DeterminerReading],
    noun: &str,
) -> Vec<DeterminerReading> {
    if could_be_a_dative_plural(noun) {
        return readings.to_vec();
    }

    keep(readings, |reading| {
        !(reading.case == Case::Dative && reading.number() == Number::Plural)
    })
}

/// Whether `noun` is spelled the way a German dative plural has to be.
pub fn could_be_a_dative_plural(noun: &str) -> bool {
    // An acronym inflects for nothing: *bei den NSAR*, *mit den AGB*.
    if noun.chars().all(|c| !c.is_lowercase()) {
        return true;
    }

    let lower = noun.to_lowercase();

    // A Latin plural in `-ae`: *bei den Mimiviridae*. Spelled out because `-e`
    // on its own is one of the commonest German singular endings.
    if lower.ends_with("ae") {
        return true;
    }

    // `-n` is the ending itself; `-s` is the `-s` plural, which takes none; and
    // `-a` and `-i` are the Latin and Greek plurals, which take none either.
    lower.ends_with(['n', 's', 'a', 'i'])
}

/// Determiner forms that stand on their own as freely as they introduce a noun.
///
/// *Zu diesen zählen Annegray, Luxeuil und St. Gallen* — `diesen` is the whole
/// phrase, and the capitalized word after it belongs to what follows. The
/// article forms are not like this: `den` and `dem` are pronouns only in a
/// relative clause, which a comma announces. Reading a noun after one of these
/// is guesswork, so the spelling rule is not applied to them.
const STANDS_ALONE: &[&str] = &["diesen", "jenen", "welchen", "solchen", "manchen"];

/// Whether `word` is one of the freely pronominal forms. See [`STANDS_ALONE`].
pub fn stands_alone(word: &str) -> bool {
    STANDS_ALONE.contains(&word.to_lowercase().as_str())
}

/// Filter, but never down to nothing.
///
/// An empty result means the determiner and the noun disagree outright — *mit
/// die Mann* — which is a different mistake, and narrowing to nothing would
/// make the caller describe it wrongly.
fn keep(
    readings: &[DeterminerReading],
    allowed: impl Fn(&DeterminerReading) -> bool,
) -> Vec<DeterminerReading> {
    let narrowed: Vec<DeterminerReading> = readings
        .iter()
        .copied()
        .filter(|reading| allowed(reading))
        .collect();

    if narrowed.is_empty() {
        readings.to_vec()
    } else {
        narrowed
    }
}

/// The same, for readings that have already been narrowed by the noun.
///
/// *mit den Freund* leaves `den` with only its accusative masculine singular
/// reading, because *Freund* is singular. Correcting from that one reading gives
/// *dem* and nothing else; correcting from every reading `den` has would also
/// offer the plural *denen*.
pub fn forms_for_readings(readings: &[DeterminerReading], wanted: CaseSet) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for (candidate, candidate_readings) in FORMS.iter() {
        if PRONOUN_ONLY.contains(&candidate.as_str()) {
            continue;
        }
        for candidate_reading in candidate_readings {
            if !wanted.contains(candidate_reading.case.into()) {
                continue;
            }
            let matches_source = readings.iter().any(|reading| {
                reading.paradigm == candidate_reading.paradigm
                    && reading.gender == candidate_reading.gender
            });
            if matches_source && !out.contains(&candidate.as_str()) {
                out.push(candidate.as_str());
            }
        }
    }

    // The map iterates in hash order, and a lint's suggestions are shown in the
    // order they are given.
    out.sort_unstable();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cases(word: &str) -> CaseSet {
        determiner_cases(word).unwrap_or_else(|| panic!("{word} is not a determiner"))
    }

    #[test]
    fn the_definite_article_has_all_of_its_readings() {
        assert_eq!(
            cases("der"),
            CaseSet::NOMINATIVE | CaseSet::DATIVE | CaseSet::GENITIVE
        );
        assert_eq!(cases("dem"), CaseSet::DATIVE);
        assert_eq!(cases("des"), CaseSet::GENITIVE);
        assert_eq!(cases("den"), CaseSet::ACCUSATIVE | CaseSet::DATIVE);
        assert_eq!(cases("das"), CaseSet::NOMINATIVE | CaseSet::ACCUSATIVE);
        assert_eq!(cases("die"), CaseSet::NOMINATIVE | CaseSet::ACCUSATIVE);
    }

    /// *der* is nominative masculine and dative feminine, but never nominative
    /// feminine. A per-axis set would claim it is; a list of readings does not.
    #[test]
    fn readings_are_joint_not_a_cross_product() {
        let der = determiner_readings("der").unwrap();
        assert!(der.contains(&DeterminerReading {
            case: Case::Nominative,
            gender: M,
            paradigm: DEFINITE_ARTICLE
        }));
        assert!(!der.contains(&DeterminerReading {
            case: Case::Nominative,
            gender: F,
            paradigm: DEFINITE_ARTICLE
        }));
    }

    #[test]
    fn a_sentence_initial_article_is_found() {
        assert_eq!(cases("Dem"), CaseSet::DATIVE);
    }

    #[test]
    fn the_strong_paradigm_is_generated() {
        assert_eq!(cases("diesem"), CaseSet::DATIVE);
        assert_eq!(
            cases("dieses"),
            CaseSet::NOMINATIVE | CaseSet::ACCUSATIVE | CaseSet::GENITIVE
        );
        assert_eq!(cases("jedem"), CaseSet::DATIVE);
        assert_eq!(cases("welchen"), CaseSet::ACCUSATIVE | CaseSet::DATIVE);
    }

    #[test]
    fn the_mixed_paradigm_is_generated() {
        assert_eq!(cases("einem"), CaseSet::DATIVE);
        assert_eq!(cases("keines"), CaseSet::GENITIVE);
        assert_eq!(cases("meiner"), CaseSet::DATIVE | CaseSet::GENITIVE);
        assert_eq!(cases("unserem"), CaseSet::DATIVE);
    }

    /// *euer* drops its stem vowel before an ending.
    #[test]
    fn euer_is_spelled_correctly_in_both_shapes() {
        assert_eq!(cases("euer"), CaseSet::NOMINATIVE | CaseSet::ACCUSATIVE);
        assert_eq!(cases("eurem"), CaseSet::DATIVE);
        assert!(determiner_readings("euerem").is_none());
    }

    /// *ein* has no plural, so *einen* must stay accusative-only. Otherwise
    /// *mit einen Freund* would look like a dative plural and go unreported.
    #[test]
    fn ein_has_no_plural() {
        assert_eq!(cases("einen"), CaseSet::ACCUSATIVE);
        assert_eq!(cases("keinen"), CaseSet::ACCUSATIVE | CaseSet::DATIVE);
    }

    /// Bare *ihr* is the personal pronoun far more often than the possessive.
    /// Bare *ihr* is the personal pronoun and bare *sein* the infinitive.
    /// Both would otherwise turn a correct *mit ihr* or *zu sein* into a case
    /// error.
    #[test]
    fn verb_and_pronoun_homographs_are_not_determiners() {
        assert!(determiner_readings("ihr").is_none());
        assert!(determiner_readings("sein").is_none());
        assert_eq!(cases("ihrem"), CaseSet::DATIVE);
        assert_eq!(cases("seinem"), CaseSet::DATIVE);
    }

    #[test]
    fn corrections_keep_gender_and_number() {
        assert_eq!(forms_in_case("dem", CaseSet::GENITIVE), ["des"]);
        assert_eq!(forms_in_case("dem", CaseSet::ACCUSATIVE), ["das", "den"]);
        assert_eq!(forms_in_case("das", CaseSet::DATIVE), ["dem"]);
        assert_eq!(forms_in_case("die", CaseSet::DATIVE), ["den", "der"]);
        // *denen* is a pronoun, not a replacement article.
        assert!(!forms_in_case("die", CaseSet::DATIVE).contains(&"denen"));
    }

    /// A correction never leaves the paradigm it started in.
    #[test]
    fn corrections_stay_within_the_paradigm() {
        assert_eq!(
            forms_in_case("einem", CaseSet::ACCUSATIVE),
            ["ein", "einen"]
        );
        assert_eq!(forms_in_case("diesem", CaseSet::GENITIVE), ["dieses"]);
        assert_eq!(forms_in_case("denen", CaseSet::ACCUSATIVE), ["die"]);
    }

    /// The whole point of reading the noun: *den* keeps only its accusative
    /// singular reading beside a singular noun.
    #[test]
    fn a_singular_noun_narrows_the_readings() {
        let den = determiner_readings("den").unwrap();
        let singular = Agreement {
            number: Number::Singular.into(),
            ..Default::default()
        };

        let allowed = readings_allowed_by(den, &singular);
        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed[0].case, Case::Accusative);
    }

    /// A plural marking narrows nothing, deliberately. German weak masculines
    /// spell the oblique singular exactly like the plural — *den Menschen* is
    /// both — and the dictionary records only the plural, so trusting it turns
    /// *"für den Menschen"* into an error.
    #[test]
    fn a_plural_noun_narrows_nothing() {
        let den = determiner_readings("den").unwrap();
        let plural = Agreement {
            number: Number::Plural.into(),
            ..Default::default()
        };

        assert_eq!(readings_allowed_by(den, &plural).len(), den.len());
    }

    /// Gender is not read at all. Too much of it is wrong: *Leber*, *Mauer* and
    /// *Nummer* are feminine and recorded masculine, and narrowing by that made
    /// *"in der Leber"* an error.
    #[test]
    fn gender_is_not_read() {
        let der = determiner_readings("der").unwrap();
        let masculine_singular = Agreement {
            gender: Gender::Masculine.into(),
            number: Number::Singular.into(),
            ..Default::default()
        };

        // Only the plural reading goes; the feminine singular ones survive.
        let allowed = readings_allowed_by(der, &masculine_singular);
        assert_eq!(allowed.len(), 3);
        assert!(allowed.iter().all(|r| r.number() == Number::Singular));
    }

    /// A noun the dictionary says nothing about narrows nothing.
    #[test]
    fn a_noun_without_features_narrows_nothing() {
        let der = determiner_readings("der").unwrap();
        assert_eq!(
            readings_allowed_by(der, &Agreement::default()).len(),
            der.len()
        );
    }

    /// *dem* is singular in both its readings, so a singular noun leaves both
    /// and the correction still offers each gender.
    #[test]
    fn narrowing_that_removes_nothing_keeps_every_correction() {
        let dem = determiner_readings("dem").unwrap();
        let singular = Agreement {
            number: Number::Singular.into(),
            ..Default::default()
        };

        let allowed = readings_allowed_by(dem, &singular);
        assert_eq!(allowed.len(), dem.len());
        assert_eq!(
            forms_for_readings(&allowed, CaseSet::ACCUSATIVE),
            ["das", "den"]
        );
    }

    /// A correction built from the narrowed readings offers one form, not the
    /// whole paradigm.
    #[test]
    fn corrections_follow_the_narrowed_readings() {
        let den = determiner_readings("den").unwrap();
        let singular = Agreement {
            number: Number::Singular.into(),
            ..Default::default()
        };

        let narrowed = readings_allowed_by(den, &singular);
        assert_eq!(forms_for_readings(&narrowed, CaseSet::DATIVE), ["dem"]);
        // Without the noun, the dative plural reading survives and `den` is
        // offered as a correction of itself.
        assert_eq!(forms_in_case("den", CaseSet::DATIVE), ["dem", "den"]);
    }

    /// German has one inflectional ending left that is exceptionless: the
    /// dative plural takes `-n`. A noun without it cannot be one, and no
    /// dictionary entry is needed to see that.
    #[test]
    fn the_spelling_rules_out_a_dative_plural() {
        let den = determiner_readings("den").unwrap();

        let beside_freund = readings_allowed_by_spelling(den, "Freund");
        assert_eq!(beside_freund.len(), 1);
        assert_eq!(beside_freund[0].case, Case::Accusative);

        // *Freunden* could be one, so nothing is ruled out.
        assert_eq!(
            readings_allowed_by_spelling(den, "Freunden").len(),
            den.len()
        );
    }

    #[test]
    fn the_endings_a_dative_plural_may_have() {
        for word in [
            "Freunden",
            "Autos",
            "Korpora",
            "Termini",
            "Mimiviridae",
            "NSAR",
        ] {
            assert!(could_be_a_dative_plural(word), "{word}");
        }
        for word in ["Freund", "Lehrer", "Bruder", "Zug", "Bäcker", "Worte"] {
            assert!(!could_be_a_dative_plural(word), "{word}");
        }
    }

    #[test]
    fn the_freely_pronominal_forms_are_named() {
        assert!(stands_alone("diesen"));
        assert!(stands_alone("Jenen"));
        assert!(!stands_alone("den"));
        assert!(!stands_alone("dem"));
    }

    #[test]
    fn a_noun_is_not_a_determiner() {
        assert!(determiner_readings("Haus").is_none());
        assert!(determiner_readings("laufen").is_none());
    }
}
