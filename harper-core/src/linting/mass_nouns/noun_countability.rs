use crate::linting::expr_linter::Chunk;
use crate::{
    Lrc, Span, Token, TokenStringExt,
    expr::{Expr, LongestMatchOf, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::{IndefiniteArticle, WordSet},
};

#[derive(Debug, Clone, Copy)]
pub enum Correction {
    // Drop the determiner or quantifier
    DropDQ,
    // Replace the determiner or quantifier with a string
    ReplaceDQWith(&'static str),
    // Insert a string between the determiner or quantifier and the mass noun
    InsertBetween(&'static str),
    // Replace the mass noun with a string
    ReplaceNounWith(&'static str),
}

use Correction::*;

pub struct NounCountability {
    expr: Box<dyn Expr>,
}

impl Default for NounCountability {
    fn default() -> Self {
        let quantifier = WordSet::new(&[
            "another", "both", "each", "every", "few", "fewer", "many", "multiple", "one",
            "several",
        ]);

        // A determiner or quantifier followed by a mass noun
        let detquant_mass = Lrc::new(
            SequenceExpr::any_of(vec![
                Box::new(IndefiniteArticle::default()),
                Box::new(quantifier),
            ])
            .then_whitespace()
            .then_mass_noun_only(),
        );

        let detauant_mass_then_hyphen =
            Lrc::new(SequenceExpr::with(detquant_mass.clone()).then_hyphen());

        let detquant_mass_following_context = Lrc::new(
            SequenceExpr::with(detquant_mass.clone())
                .then_whitespace()
                // If we don't get the word, this won't be the longest match
                .then_any_word(),
        );

        Self {
            expr: Box::new(LongestMatchOf::new(vec![
                Box::new(detquant_mass),
                Box::new(detauant_mass_then_hyphen),
                Box::new(detquant_mass_following_context),
            ])),
        }
    }
}

impl ExprLinter for NounCountability {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let toks_chars = toks.span()?.get_content(src);

        // 4 tokens means the phrase was followed by a hyphen
        if toks.len() == 4 {
            return None;
        }
        // 3 tokens means the phrase was at the end of a chunk/sentence.
        // 5 tokens means the phrase was in the middle of a chunk/sentence.
        // If it's in the middle then we check if the next word token is a noun or OOV.
        // Since the last token of our phrase is the mass noun, this would make it part of a compound noun.
        if toks.len() == 5 && (toks.last()?.kind.is_noun() || toks.last()?.kind.is_oov()) {
            return None;
        }

        // the determiner or quantifier
        let dq = toks[0].span.get_content_string(src).to_lowercase();

        // the mass noun
        let noun = toks[2].span.get_content_string(src).to_lowercase();

        let synonym_corrections: &'static [Correction] = match (noun.as_str(), dq.as_str()) {
            ("advice", "a" | "an" | "another" | "each" | "every" | "one") => &[
                ReplaceNounWith("tip"),
                ReplaceNounWith("suggestion"),
                ReplaceNounWith("recommendation"),
            ],
            ("advice", "both" | "many" | "multiple" | "several") => &[
                ReplaceNounWith("tips"),
                ReplaceNounWith("suggestions"),
                ReplaceNounWith("recommendations"),
            ],
            ("clothing", "a" | "an" | "another" | "each" | "every" | "one") => {
                &[ReplaceNounWith("garment")]
            }
            ("clothing", "both" | "many" | "multiple" | "several") => {
                &[ReplaceNounWith("garments")]
            }
            ("luggage", "a" | "an" | "another" | "each" | "every" | "one") => {
                &[ReplaceNounWith("suitcase"), ReplaceNounWith("bag")]
            }
            ("luggage", "both" | "many" | "multiple" | "several") => {
                &[ReplaceNounWith("suitcases"), ReplaceNounWith("bags")]
            }
            ("punctuation", "a" | "an" | "another" | "each" | "every" | "one") => {
                &[ReplaceNounWith("punctuation mark")]
            }
            ("punctuation", "both" | "many" | "multiple" | "several") => {
                &[ReplaceNounWith("punctuation marks")]
            }
            ("software", "a") => &[
                ReplaceNounWith("program"),
                ReplaceNounWith("software package"),
                ReplaceNounWith("software tool"),
            ],
            ("software", "an" | "another" | "each" | "every" | "one") => &[
                ReplaceNounWith("app"),
                ReplaceNounWith("application"),
                ReplaceNounWith("program"),
                ReplaceNounWith("software package"),
                ReplaceNounWith("software tool"),
            ],
            ("software", "both" | "many" | "multiple" | "several") => &[
                ReplaceNounWith("apps"),
                ReplaceNounWith("applications"),
                ReplaceNounWith("programs"),
                ReplaceNounWith("software packages"),
                ReplaceNounWith("software tools"),
            ],
            _ => &[],
        };

        let no_piece = matches!(noun.as_str(), "punctuation" | "traffic");

        let basic_corrections: &'static [Correction] = match (dq.as_str(), no_piece) {
            ("a" | "an", true) => &[DropDQ, ReplaceDQWith("some")],
            ("a" | "an", false) => &[DropDQ, ReplaceDQWith("some"), ReplaceDQWith("a piece of")],
            ("another" | "each" | "every" | "one", true) => &[],
            ("another" | "each" | "every" | "one", false) => &[InsertBetween("piece of")],
            ("both" | "multiple" | "several", true) => &[],
            ("both" | "multiple" | "several", false) => &[InsertBetween("pieces of")],
            ("few", true) => &[ReplaceDQWith("little")],
            ("few", false) => &[ReplaceDQWith("little"), InsertBetween("pieces of")],
            ("fewer", true) => &[ReplaceDQWith("less")],
            ("fewer", false) => &[ReplaceDQWith("less"), InsertBetween("pieces of")],
            ("many", true) => &[ReplaceDQWith("much"), ReplaceDQWith("a lot of")],
            ("many", false) => &[
                ReplaceDQWith("much"),
                ReplaceDQWith("a lot of"),
                InsertBetween("pieces of"),
            ],
            _ => &[],
        };

        let mut suggestions = Vec::new();

        for correction in synonym_corrections {
            let parts = match correction {
                ReplaceNounWith(w) => &[&dq, *w],
                _ => return None,
            };
            suggestions.push(Suggestion::replace_with_match_case(
                parts.join(" ").chars().collect(),
                toks_chars,
            ));
        }

        suggestions.extend(basic_corrections.iter().map(|correction| {
            let parts: &[&str] = match correction {
                DropDQ => &[&noun],
                ReplaceDQWith(w) => &[w, &noun],
                InsertBetween(w) => &[&dq, w, &noun],
                ReplaceNounWith(w) => &[&dq, w],
            };
            Suggestion::replace_with_match_case(parts.join(" ").chars().collect(), toks_chars)
        }));

        Some(Lint {
            span: Span::new(toks[0].span.start, toks[2].span.end),
            lint_kind: LintKind::Agreement,
            suggestions,
            message: format!("`{noun}` is a mass noun."),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Correct mass nouns that are preceded by the wrong determiners or quantifiers."
    }
}

#[cfg(test)]
mod tests {
    use super::NounCountability;
    use crate::linting::tests::{assert_lint_count, assert_top3_suggestion_result};

    #[test]
    fn corrects_a() {
        assert_top3_suggestion_result(
            "If the unit turns out to be noisy, can I expect a firmware with phase ...",
            NounCountability::default(),
            "If the unit turns out to be noisy, can I expect some firmware with phase ...",
        );
    }

    #[test]
    #[ignore = "replace_with_match_case matches by index, not by lower vs title vs upper"]
    fn corrects_a_title_case() {
        assert_top3_suggestion_result(
            "Simple POC of a Ransomware.",
            NounCountability::default(),
            "Simple POC of a piece of Ransomware.",
        );
    }

    #[test]
    fn corrects_an() {
        assert_top3_suggestion_result(
            "The PlaySEM platform provides an infrastructure for playing and rendering sensory effects in multimedia applications.",
            NounCountability::default(),
            "The PlaySEM platform provides infrastructure for playing and rendering sensory effects in multimedia applications.",
        );
    }

    #[test]
    #[ignore = "replace_with_match_case matches by index, not by lower vs title vs upper"]
    fn corrects_an_title_case() {
        assert_top3_suggestion_result(
            "An Infrastructure for Integrated EDA.",
            NounCountability::default(),
            "Infrastructure for Integrated EDA.",
        );
    }

    #[test]
    fn corrects_another() {
        assert_top3_suggestion_result(
            "Another ransomware made by me for fun.",
            NounCountability::default(),
            "Another piece of ransomware made by me for fun.",
        );
    }

    #[test]
    fn corrects_both() {
        assert_top3_suggestion_result(
            "Make a terminal show both information of your CPU and GPU!",
            NounCountability::default(),
            "Make a terminal show both pieces of information of your CPU and GPU!",
        );
    }

    #[test]
    // "piece of traffic" sounds very weird
    fn can_correct_each_with_traffic() {
        assert_top3_suggestion_result(
            "Beside each traffic there is also a pedestrian traffic light.",
            NounCountability::default(),
            "Beside each traffic there is also a pedestrian traffic light.",
        );
    }

    #[test]
    fn corrects_every() {
        assert_top3_suggestion_result(
            "Capacitor plugin to get access to every info about the device software and hardware.",
            NounCountability::default(),
            "Capacitor plugin to get access to every piece of info about the device software and hardware.",
        );
    }

    #[test]
    fn corrects_few() {
        assert_top3_suggestion_result(
            "Displays a few information to help you rotating through your spells.",
            NounCountability::default(),
            "Displays a few pieces of information to help you rotating through your spells.",
        );
    }

    #[test]
    fn corrects_many() {
        assert_top3_suggestion_result(
            "It shows clearly how many information about objects you can get with old search ...",
            NounCountability::default(),
            "It shows clearly how much information about objects you can get with old search ...",
        );
    }

    #[test]
    fn corrects_one() {
        assert_top3_suggestion_result(
            "For example, it only makes sense to compare global protein q-value filtering in one software with that in another.",
            NounCountability::default(),
            "For example, it only makes sense to compare global protein q-value filtering in one application with that in another.",
        );
    }

    #[test]
    #[ignore = "'in' = noun because conflated with 'IN' (Indiana)"]
    fn corrects_several() {
        assert_top3_suggestion_result(
            "The program takes in input a single XML file and outputs several info in different files.",
            NounCountability::default(),
            "The program takes in input a single XML file and outputs several pieces of info in different files.",
        );
    }

    #[test]
    fn dont_correct_many_compound() {
        assert_lint_count(
            "Additionally, many software development platforms also provide access to a community of developers.",
            NounCountability::default(),
            0,
        );
    }

    #[test]
    #[ignore]
    fn dont_correct_first_do_correct_second() {
        assert_top3_suggestion_result(
            "A advice description is required for each advice.",
            NounCountability::default(),
            "A advice description is required for each piece of advice.",
        );
    }

    #[test]
    fn corrects_an_advice() {
        assert_top3_suggestion_result(
            "Origin will not always provide the right method when an advice is applied to a bridged method.",
            NounCountability::default(),
            "Origin will not always provide the right method when an tip is applied to a bridged method.",
        );
    }

    #[test]
    fn corrects_one_advice() {
        assert_top3_suggestion_result(
            "Is it possible to use more than one advice on the same method?",
            NounCountability::default(),
            "Is it possible to use more than one tip on the same method?",
        );
    }

    #[test]
    fn corrects_every_advice() {
        assert_top3_suggestion_result(
            "Ideally every advice would have a unique identifier.",
            NounCountability::default(),
            "Ideally every tip would have a unique identifier.",
        );
    }

    #[test]
    fn corrects_a_advice() {
        assert_top3_suggestion_result(
            "Hello! I need a advice.",
            NounCountability::default(),
            "Hello! I need a tip.",
        );
    }

    #[test]
    fn corrects_a_software() {
        assert_top3_suggestion_result(
            "HGroup-DIA, a software for analyzing multiple DIA data files.",
            NounCountability::default(),
            "HGroup-DIA, a software package for analyzing multiple DIA data files.",
        );
    }

    #[test]
    fn corrects_a_luggage() {
        assert_top3_suggestion_result(
            "A luggage with a little engine, sensors (gps, ultrasounds, etc...) and bluetooth connection that will follow you everywhere.",
            NounCountability::default(),
            "A suitcase with a little engine, sensors (gps, ultrasounds, etc...) and bluetooth connection that will follow you everywhere.",
        );
    }

    #[test]
    fn corrects_multiple_advice() {
        assert_top3_suggestion_result(
            "Update Advice API doc for event and data params, multiple advice.",
            NounCountability::default(),
            "Update Advice API doc for event and data params, multiple suggestions.",
        );
    }

    #[test]
    fn corrects_every_software() {
        assert_top3_suggestion_result(
            "Rewrite every software known to man in Rust.",
            NounCountability::default(),
            "Rewrite every application known to man in Rust.",
        );
    }

    #[test]
    fn corrects_each_furniture() {
        assert_top3_suggestion_result(
            "the position (x, y) and size (height, width, length) of each furniture",
            NounCountability::default(),
            "the position (x, y) and size (height, width, length) of each piece of furniture",
        );
    }

    #[test]
    fn corrects_one_clothing() {
        assert_top3_suggestion_result(
            "Each list element represents one clothing based on weather conditions.",
            NounCountability::default(),
            "Each list element represents one garment based on weather conditions.",
        );
    }

    #[test]
    fn dont_flag_compound_nouns() {
        assert_lint_count(
            "Fill in the blanks following the creation of each Furniture class instance.",
            NounCountability::default(),
            0,
        );
        assert_lint_count(
            "This project is a clothing shop that let users buy and pay for they purchases.",
            NounCountability::default(),
            0,
        );
        assert_lint_count(
            "Yet another software router.",
            NounCountability::default(),
            0,
        );
        assert_lint_count(
            "Calculate a rate for every software component.",
            NounCountability::default(),
            0,
        );
    }

    #[test]
    fn corrects_fewer() {
        assert_top3_suggestion_result(
            "Why do my packages have fewer information?",
            NounCountability::default(),
            "Why do my packages have less information?",
        );
    }

    #[test]
    fn dont_flag_fewer_in_compound_noun() {
        assert_lint_count(
            "Additionally, less traffic leads to fewer traffic jams, resulting in a more fluent, thus more efficient, trip.",
            NounCountability::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_mass_noun_part_of_hyphenated_compound() {
        assert_lint_count(
            "Internally, we have a hardware-in-the-loop Jenkins test suite that builds and unit tests the various processes.",
            NounCountability::default(),
            0,
        );
    }

    #[test]
    fn corrects_punctuation() {
        assert_top3_suggestion_result(
            "Not in this form because it currently works with one punctuation with one letter either side.",
            NounCountability::default(),
            "Not in this form because it currently works with one punctuation mark with one letter either side.",
        );
    }
}
