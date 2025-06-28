use crate::{
    Dialect::{self, American, Australian, British, Canadian},
    Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, FixedPhrase},
    linting::{Lint, LintKind, Suggestion},
};

use super::ExprLinter;

#[derive(PartialEq)]
enum CanFlag {
    Flag,
    DontFlag,
}

use CanFlag::*;

#[derive(PartialEq)]
enum Concept {
    AubergineEggplant,
    CatsupKetchupTomatoSauce,
    CellPhoneMobilePhone,
    CoolboxCoolerEsky,
    ChipsCrisps,
    CilantroCoriander,
    DummyPacifier,
    FaucetTap,
    FlashlightTorch,
    FootballSoccer,
    FootpathPavementSidewalk,
    GasolinePetrol,
    JumperSweater,
    LightBulbLightGlobe,
    LorryTruck,
    PickupUte,
    SpannerWrench,
    StationWagonEstate,
}

use Concept::*;

/// Represents a single entry in our regional terms database
struct Term<'a> {
    /// The term (e.g., "light globe", "sidewalk")
    term: &'a str,
    /// Whether to flag this term or only suggest it
    flag: CanFlag,
    /// The dialect(s) this term is associated with
    dialects: &'a [Dialect],
    /// The concept this term is associated with
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
        flag: DontFlag,
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
        flag: DontFlag,
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
        term: "dummy",
        flag: DontFlag,
        dialects: &[Australian],
        concept: DummyPacifier,
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
        flag: DontFlag,
        dialects: &[British],
        concept: StationWagonEstate,
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
        flag: DontFlag,
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
        term: "gasoline",
        flag: Flag,
        dialects: &[American],
        concept: GasolinePetrol,
    },
    Term {
        term: "jumper",
        flag: DontFlag,
        dialects: &[Australian],
        concept: JumperSweater,
    },
    Term {
        term: "ketchup",
        flag: Flag,
        dialects: &[American],
        concept: CatsupKetchupTomatoSauce,
    },
    Term {
        term: "light bulb",
        flag: DontFlag,
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
        term: "pacifier",
        flag: Flag,
        dialects: &[American],
        concept: DummyPacifier,
    },
    Term {
        term: "pavement",
        flag: DontFlag,
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
        term: "pickup truck",
        flag: Flag,
        dialects: &[American],
        concept: PickupUte,
    },
    Term {
        term: "sidewalk",
        flag: DontFlag,
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
        term: "sweater",
        flag: Flag,
        dialects: &[American],
        concept: JumperSweater,
    },
    Term {
        term: "tap",
        flag: DontFlag,
        dialects: &[Australian, British],
        concept: FaucetTap,
    },
    Term {
        term: "tomato sauce",
        flag: DontFlag,
        dialects: &[Australian],
        concept: CatsupKetchupTomatoSauce,
    },
    Term {
        term: "torch",
        flag: DontFlag,
        dialects: &[Australian, British],
        concept: FlashlightTorch,
    },
    Term {
        term: "truck",
        flag: DontFlag,
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
        term: "wrench",
        flag: Flag,
        dialects: &[American],
        concept: SpannerWrench,
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

        let concept = &REGIONAL_TERMS
            .iter()
            .find(|row| row.term == flagged_term_string)
            .unwrap()
            .concept;

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
                "`{flagged_term_string}` isn't used in {linter_dialect}. Use `{}` instead.",
                other_terms[0]
            )
        } else {
            format!("`{flagged_term_string}` isn't used in {linter_dialect}.")
        };

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
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
    use crate::linting::tests::assert_top3_suggestion_result;

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
}
