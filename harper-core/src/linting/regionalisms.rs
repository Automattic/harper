use crate::{
    Dialect::{self, American, Australian, British, Canadian},
    Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, FixedPhrase},
    linting::{Lint, LintKind, Suggestion},
};

use super::ExprLinter;

#[derive(PartialEq)]
enum CanFlag {
    /// Flag this term as a regionalism that should be suggested against
    Flag,
    /// Don't flag this term because it's universally understood across dialects
    UniversalTerm,
    /// Don't flag this term because it has other common meanings that could cause false positives
    HasOtherMeanings,
}

use CanFlag::*;

/// Represents a unique concept that has different regional terms across English dialects.
/// Each is named by an alphabetical concatenation of the terms that refer to the same concept.
/// This allows us to suggest appropriate regional alternatives when a term from another dialect is detected.
#[derive(PartialEq)]
enum Concept {
    AubergineEggplant,
    // BiscuitCookie - biscuit names different foods in UK/Aus vs US; cookie has other meanings
    // BiscuitCracker - cracker also has other meanings
    BumBagFannyPack,
    BurglarizeBurgle,
    CampervanRv,
    CaravanTrailer,
    CatsupKetchupTomatoSauce,
    CellPhoneMobilePhone,
    CoolboxCoolerEsky,
    ChipsCrisps,
    CilantroCoriander,
    DiaperNappy,
    DoonaDuvet,
    DummyPacifier,
    FaucetTap,
    FlashlightTorch,
    FootballSoccer,
    FootpathPavementSidewalk,
    GasolinePetrol,
    GasStationPetrolStationServiceStation,
    // HooverVacuumCleaner - Hoover is also a surname and vacuum cleaner is universal.
    JumperSweater,
    LightBulbLightGlobe,
    LorryTruck,
    MotorhomeRv,
    PhotocopierXerox,
    PhotocopyXerox,
    PickupUte,
    PramStroller,
    SpannerWrench,
    StationWagonEstate,
    WindscreenWindshield,
}

use Concept::*;

/// Represents a single entry in our regional terms database
struct Term<'a> {
    /// The term (e.g., "light globe", "sidewalk")
    term: &'a str,
    /// Whether to flag this term or only suggest it.
    /// We don't want to flag universal terms just because they have a regional synonym.
    /// We also don't want to flag terms that are common in other senses.
    flag: CanFlag,
    /// The dialect(s) this term is associated with.
    dialects: &'a [Dialect],
    /// The concept this term is associated with.
    /// Named by concatenating all the associated terms in alphabetical order.
    concept: Concept,
}

const REGIONAL_TERMS: &[Term<'_>] = &[
    Term {
        term: "aubergine",
        flag: Flag,
        dialects: &[British],
        concept: AubergineEggplant,
    },
    Term {
        term: "bum bag",
        flag: Flag,
        dialects: &[Australian],
        concept: BumBagFannyPack,
    },
    Term {
        term: "burglarize",
        flag: Flag,
        dialects: &[American],
        concept: BurglarizeBurgle,
    },
    Term {
        term: "burgle",
        flag: Flag,
        dialects: &[British],
        concept: BurglarizeBurgle,
    },
    Term {
        term: "campervan",
        flag: Flag,
        dialects: &[Australian, British],
        concept: CampervanRv,
    },
    Term {
        term: "caravan",
        flag: UniversalTerm,
        dialects: &[Australian, British],
        concept: CaravanTrailer,
    },
    Term {
        term: "catsup",
        flag: Flag,
        dialects: &[American],
        concept: CatsupKetchupTomatoSauce,
    },
    Term {
        term: "cellphone",
        flag: Flag,
        dialects: &[American],
        concept: CellPhoneMobilePhone,
    },
    Term {
        term: "chips",
        flag: UniversalTerm,
        dialects: &[American, Australian],
        concept: ChipsCrisps,
    },
    Term {
        term: "cilantro",
        flag: Flag,
        dialects: &[American],
        concept: CilantroCoriander,
    },
    Term {
        term: "coolbox",
        flag: Flag,
        dialects: &[British],
        concept: CoolboxCoolerEsky,
    },
    Term {
        term: "cooler",
        flag: HasOtherMeanings,
        dialects: &[American, Canadian],
        concept: CoolboxCoolerEsky,
    },
    Term {
        term: "coriander",
        flag: Flag,
        dialects: &[Australian, British],
        concept: CilantroCoriander,
    },
    Term {
        term: "crisps",
        flag: Flag,
        dialects: &[British],
        concept: ChipsCrisps,
    },
    Term {
        term: "diaper",
        flag: Flag,
        dialects: &[American, Canadian],
        concept: DiaperNappy,
    },
    Term {
        term: "doona",
        flag: Flag,
        dialects: &[Australian],
        concept: DoonaDuvet,
    },
    Term {
        term: "dummy",
        flag: HasOtherMeanings,
        dialects: &[Australian],
        concept: DummyPacifier,
    },
    Term {
        term: "duvet",
        flag: Flag,
        dialects: &[Australian],
        concept: DoonaDuvet,
    },
    Term {
        term: "eggplant",
        flag: Flag,
        dialects: &[American, Australian],
        concept: AubergineEggplant,
    },
    Term {
        term: "esky",
        flag: Flag,
        dialects: &[Australian],
        concept: CoolboxCoolerEsky,
    },
    Term {
        term: "estate",
        flag: HasOtherMeanings,
        dialects: &[British],
        concept: StationWagonEstate,
    },
    Term {
        term: "fanny pack",
        flag: Flag,
        dialects: &[American, Canadian],
        concept: BumBagFannyPack,
    },
    Term {
        term: "faucet",
        flag: Flag,
        dialects: &[American],
        concept: FaucetTap,
    },
    Term {
        term: "flashlight",
        flag: Flag,
        dialects: &[American, Canadian],
        concept: FlashlightTorch,
    },
    Term {
        term: "football",
        flag: HasOtherMeanings,
        dialects: &[British],
        concept: FootballSoccer,
    },
    Term {
        term: "footpath",
        flag: Flag,
        dialects: &[Australian],
        concept: FootpathPavementSidewalk,
    },
    Term {
        term: "gas",
        flag: HasOtherMeanings,
        dialects: &[American, Canadian],
        concept: GasolinePetrol,
    },
    Term {
        term: "gas station",
        flag: Flag,
        dialects: &[American, Canadian],
        concept: GasStationPetrolStationServiceStation,
    },
    Term {
        term: "gasoline",
        flag: Flag,
        dialects: &[American],
        concept: GasolinePetrol,
    },
    Term {
        term: "jumper",
        flag: HasOtherMeanings,
        dialects: &[Australian],
        concept: JumperSweater,
    },
    Term {
        term: "ketchup",
        flag: Flag,
        dialects: &[American, Canadian],
        concept: CatsupKetchupTomatoSauce,
    },
    Term {
        term: "light bulb",
        flag: UniversalTerm,
        dialects: &[American, Australian, British, Canadian],
        concept: LightBulbLightGlobe,
    },
    Term {
        term: "light globe",
        flag: Flag,
        dialects: &[Australian],
        concept: LightBulbLightGlobe,
    },
    Term {
        term: "lorry",
        flag: Flag,
        dialects: &[British],
        concept: LorryTruck,
    },
    Term {
        term: "mobile phone",
        flag: Flag,
        dialects: &[Australian, British],
        concept: CellPhoneMobilePhone,
    },
    Term {
        term: "motorhome",
        flag: Flag,
        dialects: &[Australian, British],
        concept: MotorhomeRv,
    },
    Term {
        term: "nappy",
        flag: Flag,
        dialects: &[Australian, British],
        concept: DiaperNappy,
    },
    Term {
        term: "pacifier",
        flag: Flag,
        dialects: &[American],
        concept: DummyPacifier,
    },
    Term {
        term: "pavement",
        flag: HasOtherMeanings,
        dialects: &[British],
        concept: FootpathPavementSidewalk,
    },
    Term {
        term: "petrol",
        flag: Flag,
        dialects: &[Australian, British],
        concept: GasolinePetrol,
    },
    Term {
        term: "petrol station",
        flag: Flag,
        dialects: &[Australian, British],
        concept: GasStationPetrolStationServiceStation,
    },
    Term {
        term: "photocopier",
        flag: Flag,
        dialects: &[Australian, British, Canadian],
        concept: PhotocopierXerox,
    },
    Term {
        term: "photocopy",
        flag: Flag,
        dialects: &[Australian, British, Canadian],
        concept: PhotocopyXerox,
    },
    Term {
        term: "pickup truck",
        flag: Flag,
        dialects: &[American],
        concept: PickupUte,
    },
    Term {
        term: "pram",
        flag: Flag,
        dialects: &[Australian, British],
        concept: PramStroller,
    },
    Term {
        // Must be normalized to lowercase
        term: "rv",
        flag: Flag,
        dialects: &[American],
        concept: CampervanRv,
    },
    Term {
        term: "rv",
        flag: Flag,
        dialects: &[American],
        concept: MotorhomeRv,
    },
    Term {
        term: "sidewalk",
        flag: Flag,
        dialects: &[American, Canadian],
        concept: FootpathPavementSidewalk,
    },
    Term {
        term: "soccer",
        flag: Flag,
        dialects: &[American, Australian],
        concept: FootballSoccer,
    },
    Term {
        term: "spanner",
        flag: Flag,
        dialects: &[Australian, British],
        concept: SpannerWrench,
    },
    Term {
        term: "station wagon",
        flag: Flag,
        dialects: &[American, Australian],
        concept: StationWagonEstate,
    },
    Term {
        term: "stroller",
        flag: Flag,
        dialects: &[American, Australian],
        concept: PramStroller,
    },
    Term {
        term: "sweater",
        flag: Flag,
        dialects: &[American],
        concept: JumperSweater,
    },
    Term {
        term: "tap",
        flag: HasOtherMeanings,
        dialects: &[Australian, British],
        concept: FaucetTap,
    },
    Term {
        term: "tomato sauce",
        flag: HasOtherMeanings,
        dialects: &[Australian],
        concept: CatsupKetchupTomatoSauce,
    },
    Term {
        term: "torch",
        flag: HasOtherMeanings,
        dialects: &[Australian, British],
        concept: FlashlightTorch,
    },
    Term {
        term: "trailer",
        flag: HasOtherMeanings,
        dialects: &[American],
        concept: CaravanTrailer,
    },
    Term {
        term: "truck",
        flag: HasOtherMeanings,
        dialects: &[American, Australian, Canadian],
        concept: LorryTruck,
    },
    Term {
        term: "ute",
        flag: Flag,
        dialects: &[Australian],
        concept: PickupUte,
    },
    Term {
        term: "xerox",
        dialects: &[American],
        flag: Flag,
        concept: PhotocopierXerox,
    },
    Term {
        term: "xerox",
        flag: Flag,
        dialects: &[American],
        concept: PhotocopyXerox,
    },
    Term {
        term: "wrench",
        flag: Flag,
        dialects: &[American],
        concept: SpannerWrench,
    },
    Term {
        term: "windscreen",
        flag: Flag,
        dialects: &[British, Australian],
        concept: WindscreenWindshield,
    },
    Term {
        term: "windshield",
        flag: Flag,
        dialects: &[American, Canadian],
        concept: WindscreenWindshield,
    },
];

pub struct Regionalisms {
    expr: Box<dyn Expr>,
    dialect: Dialect,
}

impl Regionalisms {
    pub fn new(dialect: Dialect) -> Self {
        let terms: Vec<Box<dyn Expr>> = REGIONAL_TERMS
            .iter()
            .filter(|row| row.flag == Flag)
            .map(|row| Box::new(FixedPhrase::from_phrase(row.term)) as Box<dyn Expr>)
            .collect();

        Self {
            expr: Box::new(FirstMatchOf::new(terms)),
            dialect,
        }
    }
}

impl ExprLinter for Regionalisms {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.span()?;
        let flagged_term_chars = span.get_content(src);
        let flagged_term_string = span.get_content_string(src).to_lowercase();

        let linter_dialect = self.dialect;

        // If this term is used in the linter dialect, then we don't want to lint it.
        if REGIONAL_TERMS
            .iter()
            .any(|row| row.term == flagged_term_string && row.dialects.contains(&linter_dialect))
        {
            return None;
        }

        let concept = match REGIONAL_TERMS
            .iter()
            .find(|row| row.term == flagged_term_string)
        {
            Some(term) => &term.concept,
            None => return None, // No matching term found, so nothing to lint
        };

        let other_terms = REGIONAL_TERMS
            .iter()
            .filter(|row| row.concept == *concept)
            .filter_map(|row| {
                if row.dialects.contains(&linter_dialect) {
                    Some(&row.term)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if other_terms.is_empty() {
            return None;
        }

        let suggestions = other_terms
            .iter()
            .map(|term| Suggestion::replace_with_match_case_str(term, flagged_term_chars))
            .collect::<Vec<_>>();

        let message = if other_terms.len() == 1 {
            format!(
                "`{flagged_term_string}` isn't used in {linter_dialect} English. Use `{}` instead.",
                other_terms[0]
            )
        } else {
            format!("`{flagged_term_string}` isn't used in {linter_dialect} English.")
        };

        Some(Lint {
            span,
            lint_kind: LintKind::Regionalism,
            suggestions,
            message,
            priority: 64,
        })
    }

    fn description(&self) -> &str {
        "Regionalisms"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linting::tests::{assert_lint_count, assert_top3_suggestion_result};

    #[test]
    fn uk_to_us_food() {
        assert_top3_suggestion_result(
            "I can't eat aubergine or coriander, so I'll just have a bag of crisps.",
            Regionalisms::new(Dialect::American),
            "I can't eat eggplant or cilantro, so I'll just have a bag of chips.",
        );
    }

    #[test]
    fn au_to_us_phone() {
        assert_top3_suggestion_result(
            "I dropped my mobile phone in the esky and now it's covered in tomato sauce.",
            Regionalisms::new(Dialect::American),
            // Tomato sauce is valid in American English, it just means pasta sauce rather than ketchup.
            "I dropped my cellphone in the cooler and now it's covered in tomato sauce.",
        )
    }

    #[test]
    fn au_to_uk_cars() {
        assert_top3_suggestion_result(
            "Drive the station wagon onto the footpath and hand me that spanner.",
            Regionalisms::new(Dialect::British),
            "Drive the estate onto the pavement and hand me that spanner.",
        )
    }

    #[test]
    fn au_to_us_cars() {
        assert_top3_suggestion_result(
            "Drive the station wagon onto the footpath and hand me that spanner.",
            Regionalisms::new(Dialect::American),
            "Drive the station wagon onto the sidewalk and hand me that wrench.",
        )
    }

    #[test]
    fn us_to_au_baby() {
        assert_top3_suggestion_result(
            "Wash the pacifier under the faucet.",
            Regionalisms::new(Dialect::Australian),
            "Wash the dummy under the tap.",
        )
    }

    #[test]
    fn us_to_uk_fuel() {
        assert_top3_suggestion_result(
            "I needed more gasoline to drive the truck to the soccer match.",
            Regionalisms::new(Dialect::British),
            "I needed more petrol to drive the truck to the football match.",
        )
    }

    #[test]
    fn au_to_uk_light() {
        assert_top3_suggestion_result(
            "Can you sell me a light globe for this torch?",
            Regionalisms::new(Dialect::British),
            "Can you sell me a light bulb for this torch?",
        )
    }

    #[test]
    fn us_to_au_oops() {
        assert_top3_suggestion_result(
            "I spilled ketchup on my clean sweater.",
            Regionalisms::new(Dialect::Australian),
            "I spilled tomato sauce on my clean jumper.",
        )
    }

    #[test]
    fn caravan_doesnt_always_mean_trailer() {
        assert_lint_count(
            "A caravan (from Persian کاروان kârvân) is a group of people traveling together, often on a trade expedition. Caravans were used mainly in desert areas.",
            Regionalisms::new(Dialect::British),
            0,
        )
    }

    #[test]
    fn uk_to_us_windscreen() {
        assert_top3_suggestion_result(
            "Detect raindrops on vehicle windscreen by combining various region proposal algorithm with Convolutional Neural Network.",
            Regionalisms::new(Dialect::American),
            "Detect raindrops on vehicle windshield by combining various region proposal algorithm with Convolutional Neural Network.",
        )
    }
}
