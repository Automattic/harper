use crate::{
    CharStringExt, Span, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, Lint, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_hyphen, followed_by_word},
    },
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
    expr: SequenceExpr,
}

impl Default for NounCountability {
    fn default() -> Self {
        let quantifier = WordSet::new(&[
            "another", "both", "each", "every", "few", "fewer", "many", "multiple", "one",
            "several",
        ]);

        Self {
            expr: SequenceExpr::any_of(vec![
                Box::new(IndefiniteArticle::default()),
                Box::new(quantifier) as Box<dyn Expr>,
            ])
            .then_whitespace()
            .then_mass_noun_only(),
        }
    }
}

impl ExprLinter for NounCountability {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let toks_chars = toks.span()?.get_content(src);

        if toks.len() != 3
            || followed_by_hyphen(ctx)
            || followed_by_word(ctx, |t| {
                t.kind.is_noun() || t.kind.is_oov() || t.get_ch(src).eq_str("and")
            })
        {
            return None;
        }

        // the determiner or quantifier
        let dq = toks[0].get_str(src).to_lowercase();

        // the mass noun
        let noun = toks[2].get_str(src).to_lowercase();

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
    use crate::linting::tests::{
        assert_lint_count, assert_suggestion_result, assert_suggestion_result_with_language,
    };

    #[test]
    fn corrects_a() {
        assert_suggestion_result_with_language(
            "If the unit turns out to be noisy, can I expect a firmware with phase ...",
            NounCountability::default(),
            "If the unit turns out to be noisy, can I expect some firmware with phase ...",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    #[ignore = "replace_with_match_case matches by index, not by lower vs title vs upper"]
    fn corrects_a_title_case() {
        assert_suggestion_result_with_language(
            "Simple POC of a Ransomware.",
            NounCountability::default(),
            "Simple POC of a piece of Ransomware.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_an() {
        assert_suggestion_result_with_language(
            "The PlaySEM platform provides an infrastructure for playing and rendering sensory effects in multimedia applications.",
            NounCountability::default(),
            "The PlaySEM platform provides infrastructure for playing and rendering sensory effects in multimedia applications.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    #[ignore = "replace_with_match_case matches by index, not by lower vs title vs upper"]
    fn corrects_an_title_case() {
        assert_suggestion_result_with_language(
            "An Infrastructure for Integrated EDA.",
            NounCountability::default(),
            "Infrastructure for Integrated EDA.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_another() {
        assert_suggestion_result_with_language(
            "Another ransomware made by me for fun.",
            NounCountability::default(),
            "Another piece of ransomware made by me for fun.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_both() {
        assert_suggestion_result_with_language(
            "Make a terminal show both information of your CPU and GPU!",
            NounCountability::default(),
            "Make a terminal show both pieces of information of your CPU and GPU!",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    // "piece of traffic" sounds very weird
    fn can_correct_each_with_traffic() {
        assert_suggestion_result_with_language(
            "Beside each traffic there is also a pedestrian traffic light.",
            NounCountability::default(),
            "Beside each traffic there is also a pedestrian traffic light.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_every() {
        assert_suggestion_result_with_language(
            "Capacitor plugin to get access to every info about the device software and hardware.",
            NounCountability::default(),
            "Capacitor plugin to get access to every piece of info about the device software and hardware.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_few() {
        assert_suggestion_result_with_language(
            "Displays a few information to help you rotating through your spells.",
            NounCountability::default(),
            "Displays a few pieces of information to help you rotating through your spells.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_many() {
        assert_suggestion_result_with_language(
            "It shows clearly how many information about objects you can get with old search ...",
            NounCountability::default(),
            "It shows clearly how much information about objects you can get with old search ...",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_one() {
        assert_suggestion_result_with_language(
            "For example, it only makes sense to compare global protein q-value filtering in one software with that in another.",
            NounCountability::default(),
            "For example, it only makes sense to compare global protein q-value filtering in one application with that in another.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    #[ignore = "'in' = noun because conflated with 'IN' (Indiana)"]
    fn corrects_several() {
        assert_suggestion_result_with_language(
            "The program takes in input a single XML file and outputs several info in different files.",
            NounCountability::default(),
            "The program takes in input a single XML file and outputs several pieces of info in different files.",
            crate::languages::LanguageFamily::English,
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
        assert_suggestion_result_with_language(
            "A advice description is required for each advice.",
            NounCountability::default(),
            "A advice description is required for each piece of advice.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_an_advice() {
        assert_suggestion_result_with_language(
            "Origin will not always provide the right method when an advice is applied to a bridged method.",
            NounCountability::default(),
            "Origin will not always provide the right method when an tip is applied to a bridged method.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_one_advice() {
        assert_suggestion_result_with_language(
            "Is it possible to use more than one advice on the same method?",
            NounCountability::default(),
            "Is it possible to use more than one tip on the same method?",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_every_advice() {
        assert_suggestion_result_with_language(
            "Ideally every advice would have a unique identifier.",
            NounCountability::default(),
            "Ideally every tip would have a unique identifier.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_a_advice() {
        assert_suggestion_result_with_language(
            "Hello! I need a advice.",
            NounCountability::default(),
            "Hello! I need a tip.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_a_software() {
        assert_suggestion_result_with_language(
            "HGroup-DIA, a software for analyzing multiple DIA data files.",
            NounCountability::default(),
            "HGroup-DIA, a software package for analyzing multiple DIA data files.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_a_luggage() {
        assert_suggestion_result_with_language(
            "A luggage with a little engine, sensors (gps, ultrasounds, etc...) and bluetooth connection that will follow you everywhere.",
            NounCountability::default(),
            "A suitcase with a little engine, sensors (gps, ultrasounds, etc...) and bluetooth connection that will follow you everywhere.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_multiple_advice() {
        assert_suggestion_result_with_language(
            "Update Advice API doc for event and data params, multiple advice.",
            NounCountability::default(),
            "Update Advice API doc for event and data params, multiple suggestions.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_every_software() {
        assert_suggestion_result_with_language(
            "Rewrite every software known to man in Rust.",
            NounCountability::default(),
            "Rewrite every application known to man in Rust.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_each_furniture() {
        assert_suggestion_result_with_language(
            "the position (x, y) and size (height, width, length) of each furniture",
            NounCountability::default(),
            "the position (x, y) and size (height, width, length) of each piece of furniture",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn corrects_one_clothing() {
        assert_suggestion_result_with_language(
            "Each list element represents one clothing based on weather conditions.",
            NounCountability::default(),
            "Each list element represents one garment based on weather conditions.",
            crate::languages::LanguageFamily::English,
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
        assert_suggestion_result_with_language(
            "Why do my packages have fewer information?",
            NounCountability::default(),
            "Why do my packages have less information?",
            crate::languages::LanguageFamily::English,
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
        assert_suggestion_result_with_language(
            "Not in this form because it currently works with one punctuation with one letter either side.",
            NounCountability::default(),
            "Not in this form because it currently works with one punctuation mark with one letter either side.",
            crate::languages::LanguageFamily::English,
        );
    }
}
