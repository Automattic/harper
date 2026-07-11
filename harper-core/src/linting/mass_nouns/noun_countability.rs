use crate::{
    CharStringExt, Span, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, Lint, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_hyphen, followed_by_word},
    },
    patterns::{IndefiniteArticle, Pattern, WordSet},
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

/// Matches a qualified phrase only when `software` is its final noun.
///
/// Applying this to every mass noun produces false positives for words that also have valid
/// countable senses.
struct QualifiedSoftware;

impl Pattern for QualifiedSoftware {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        let mut cursor = 0;
        let mut last_noun_end = None;
        let mut software_end = None;

        loop {
            let token = tokens.get(cursor)?;

            if !is_software_phrase_word(token, source) {
                break;
            }

            if token.kind.is_noun() || token.kind.is_oov() {
                last_noun_end = Some(cursor + 1);

                if token.get_ch(source).eq_str("software") {
                    software_end = Some(cursor + 1);
                }
            }

            let Some(separator) = tokens.get(cursor + 1) else {
                break;
            };

            if !separator.kind.is_whitespace() && !separator.kind.is_hyphen() {
                break;
            }

            let Some(next) = tokens.get(cursor + 2) else {
                break;
            };

            if !is_software_phrase_word(next, source) {
                break;
            }

            cursor += 2;
        }

        match (last_noun_end, software_end) {
            (Some(noun_end), Some(software_end)) if noun_end == software_end => Some(software_end),
            _ => None,
        }
    }
}

pub struct NounCountability {
    expr: SequenceExpr,
}

impl Default for NounCountability {
    fn default() -> Self {
        let determiner_or_quantifier = || {
            let quantifier = WordSet::new(&[
                "another", "both", "each", "every", "few", "fewer", "many", "multiple", "one",
                "several",
            ]);

            SequenceExpr::any_of(vec![
                Box::new(IndefiniteArticle::default()),
                Box::new(quantifier) as Box<dyn Expr>,
            ])
            .then_whitespace()
        };

        let direct = determiner_or_quantifier().then_mass_noun_only();
        let qualified_software = determiner_or_quantifier().then(QualifiedSoftware);

        Self {
            expr: SequenceExpr::longest_of([Box::new(direct), Box::new(qualified_software)]),
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
        let [dq_token, whitespace, rest @ ..] = toks else {
            return None;
        };
        let noun_token = rest.last()?;

        if !whitespace.kind.is_whitespace()
            || !noun_token.kind.is_mass_noun_only()
            || followed_by_hyphen(ctx)
            || followed_by_word(ctx, |t| {
                t.kind.is_noun() || t.kind.is_oov() || t.get_ch(src).eq_str("and")
            })
            || (is_ing_word(noun_token, src) && followed_by_nominal_head(ctx, src))
        {
            return None;
        }

        // the determiner or quantifier
        let dq = dq_token.get_str(src).to_lowercase();

        // the mass noun
        let noun = noun_token.get_str(src).to_lowercase();

        let phrase_span = toks.span()?;
        let toks_chars = phrase_span.get_content(src);

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

        if toks.len() == 3 {
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
        } else {
            let noun_chars = noun_token.get_ch(src);
            let dq_chars = dq_token.get_ch(src);
            let before_noun = Span::new(phrase_span.start, noun_token.span.start).get_content(src);
            let after_dq = Span::new(dq_token.span.end, noun_token.span.end).get_content(src);
            let without_dq =
                Span::new(rest.first()?.span.start, noun_token.span.end).get_content(src);

            for correction in synonym_corrections {
                let replacement = match correction {
                    ReplaceNounWith(word) => replace_noun(before_noun, word, noun_chars),
                    _ => return None,
                };
                suggestions.push(replacement);
            }

            suggestions.extend(basic_corrections.iter().map(|correction| match correction {
                DropDQ => Suggestion::ReplaceWith(without_dq.to_vec()),
                ReplaceDQWith(word) => replace_dq(word, dq_chars, after_dq),
                InsertBetween(word) => insert_after_dq(word, dq_chars, after_dq),
                ReplaceNounWith(word) => replace_noun(before_noun, word, noun_chars),
            }));
        }

        Some(Lint {
            span: phrase_span,
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

fn match_case(value: &str, template: &[char]) -> Vec<char> {
    let Suggestion::ReplaceWith(chars) = Suggestion::replace_with_match_case_str(value, template)
    else {
        unreachable!();
    };
    chars
}

fn replace_noun(prefix: &[char], replacement: &str, noun: &[char]) -> Suggestion {
    let mut result = prefix.to_vec();
    result.extend(match_case(replacement, noun));
    Suggestion::ReplaceWith(result)
}

fn replace_dq(replacement: &str, dq: &[char], suffix: &[char]) -> Suggestion {
    let mut result = match_case(replacement, dq);
    result.extend_from_slice(suffix);
    Suggestion::ReplaceWith(result)
}

fn insert_after_dq(insertion: &str, dq: &[char], suffix: &[char]) -> Suggestion {
    let mut result = dq.to_vec();
    result.push(' ');
    result.extend(insertion.chars());
    result.extend_from_slice(suffix);
    Suggestion::ReplaceWith(result)
}

fn is_ing_word(tok: &Token, src: &[char]) -> bool {
    tok.get_ch(src).ends_with_ignore_ascii_case_str("ing")
}

fn followed_by_nominal_head(ctx: Option<(&[Token], &[Token])>, src: &[char]) -> bool {
    let Some((_, after)) = ctx else {
        return false;
    };

    follows_modifiers_then_noun(after, src)
        || follows_slash_separated_ing_modifiers_then_noun(after, src)
}

fn follows_modifiers_then_noun(tokens: &[Token], src: &[char]) -> bool {
    let mut cursor = 0;

    loop {
        let Some([ws, word, rest @ ..]) = tokens.get(cursor..) else {
            return false;
        };

        if !ws.kind.is_whitespace() {
            return false;
        }

        if is_nominal_chain_boundary(word, src) {
            return false;
        }

        if is_likely_nominal_head(word, src) {
            return true;
        }

        if word.kind.is_adjective()
            || word.kind.is_adverb()
            || word.kind.is_verb_progressive_form()
            || word.kind.is_verb_past_participle_form()
        {
            cursor = tokens.len() - rest.len();
            continue;
        }

        return false;
    }
}

fn is_nominal_chain_boundary(tok: &Token, src: &[char]) -> bool {
    tok.kind.is_preposition()
        || tok.kind.is_determiner()
        || tok.kind.is_conjunction()
        || tok.get_ch(src).eq_any_ignore_ascii_case_str(&[
            "about", "after", "as", "at", "before", "by", "for", "from", "in", "into", "of", "on",
            "through", "to", "under", "with",
        ])
}

fn is_software_phrase_word(tok: &Token, src: &[char]) -> bool {
    !is_nominal_chain_boundary(tok, src)
        && (tok.kind.is_adjective()
            || tok.kind.is_adverb()
            || tok.kind.is_noun()
            || tok.kind.is_oov()
            || tok.kind.is_verb_progressive_form()
            || tok.kind.is_verb_past_participle_form())
}

fn is_likely_nominal_head(tok: &Token, src: &[char]) -> bool {
    (tok.kind.is_noun() || tok.kind.is_oov()) && !is_nominal_chain_boundary(tok, src)
}

fn follows_slash_separated_ing_modifiers_then_noun(tokens: &[Token], src: &[char]) -> bool {
    let mut cursor = 0;
    let mut saw_slash_modifier = false;

    while let Some([slash, word, rest @ ..]) = tokens.get(cursor..) {
        if !slash.kind.is_slash() || !is_ing_word(word, src) {
            break;
        }

        saw_slash_modifier = true;
        cursor = tokens.len() - rest.len();
    }

    saw_slash_modifier && follows_modifiers_then_noun(&tokens[cursor..], src)
}

#[cfg(test)]
mod tests {
    use super::NounCountability;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn corrects_a() {
        assert_suggestion_result(
            "If the unit turns out to be noisy, can I expect a firmware with phase ...",
            NounCountability::default(),
            "If the unit turns out to be noisy, can I expect some firmware with phase ...",
        );
    }

    #[test]
    #[ignore = "replace_with_match_case matches by index, not by lower vs title vs upper"]
    fn corrects_a_title_case() {
        assert_suggestion_result(
            "Simple POC of a Ransomware.",
            NounCountability::default(),
            "Simple POC of a piece of Ransomware.",
        );
    }

    #[test]
    fn corrects_an() {
        assert_suggestion_result(
            "The PlaySEM platform provides an infrastructure for playing and rendering sensory effects in multimedia applications.",
            NounCountability::default(),
            "The PlaySEM platform provides infrastructure for playing and rendering sensory effects in multimedia applications.",
        );
    }

    #[test]
    #[ignore = "replace_with_match_case matches by index, not by lower vs title vs upper"]
    fn corrects_an_title_case() {
        assert_suggestion_result(
            "An Infrastructure for Integrated EDA.",
            NounCountability::default(),
            "Infrastructure for Integrated EDA.",
        );
    }

    #[test]
    fn corrects_another() {
        assert_suggestion_result(
            "Another ransomware made by me for fun.",
            NounCountability::default(),
            "Another piece of ransomware made by me for fun.",
        );
    }

    #[test]
    fn corrects_both() {
        assert_suggestion_result(
            "Make a terminal show both information of your CPU and GPU!",
            NounCountability::default(),
            "Make a terminal show both pieces of information of your CPU and GPU!",
        );
    }

    #[test]
    // "piece of traffic" sounds very weird
    fn can_correct_each_with_traffic() {
        assert_suggestion_result(
            "Beside each traffic there is also a pedestrian traffic light.",
            NounCountability::default(),
            "Beside each traffic there is also a pedestrian traffic light.",
        );
    }

    #[test]
    fn corrects_every() {
        assert_suggestion_result(
            "Capacitor plugin to get access to every info about the device software and hardware.",
            NounCountability::default(),
            "Capacitor plugin to get access to every piece of info about the device software and hardware.",
        );
    }

    #[test]
    fn corrects_few() {
        assert_suggestion_result(
            "Displays a few information to help you rotating through your spells.",
            NounCountability::default(),
            "Displays a few pieces of information to help you rotating through your spells.",
        );
    }

    #[test]
    fn corrects_many() {
        assert_suggestion_result(
            "It shows clearly how many information about objects you can get with old search ...",
            NounCountability::default(),
            "It shows clearly how much information about objects you can get with old search ...",
        );
    }

    #[test]
    fn corrects_one() {
        assert_suggestion_result(
            "For example, it only makes sense to compare global protein q-value filtering in one software with that in another.",
            NounCountability::default(),
            "For example, it only makes sense to compare global protein q-value filtering in one application with that in another.",
        );
    }

    #[test]
    #[ignore = "'in' = noun because conflated with 'IN' (Indiana)"]
    fn corrects_several() {
        assert_suggestion_result(
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
    fn dont_flag_ing_modifier_before_adjective_noun() {
        assert_no_lints(
            "They succeeded by receiving a working direct link in return.",
            NounCountability::default(),
        );
    }

    #[test]
    fn dont_flag_slash_separated_ing_modifiers_before_noun() {
        assert_no_lints(
            "MockAPI is meant to be a prototyping/testing/learning tool.",
            NounCountability::default(),
        );
    }

    #[test]
    fn dont_flag_coordinate_predicate_ing_adjective() {
        assert_no_lints(
            "They're both confusing and tedious to use.",
            NounCountability::default(),
        );
    }

    #[test]
    #[ignore]
    fn dont_correct_first_do_correct_second() {
        assert_suggestion_result(
            "A advice description is required for each advice.",
            NounCountability::default(),
            "A advice description is required for each piece of advice.",
        );
    }

    #[test]
    fn corrects_an_advice() {
        assert_suggestion_result(
            "Origin will not always provide the right method when an advice is applied to a bridged method.",
            NounCountability::default(),
            "Origin will not always provide the right method when an tip is applied to a bridged method.",
        );
    }

    #[test]
    fn corrects_one_advice() {
        assert_suggestion_result(
            "Is it possible to use more than one advice on the same method?",
            NounCountability::default(),
            "Is it possible to use more than one tip on the same method?",
        );
    }

    #[test]
    fn corrects_every_advice() {
        assert_suggestion_result(
            "Ideally every advice would have a unique identifier.",
            NounCountability::default(),
            "Ideally every tip would have a unique identifier.",
        );
    }

    #[test]
    fn corrects_a_advice() {
        assert_suggestion_result(
            "Hello! I need a advice.",
            NounCountability::default(),
            "Hello! I need a tip.",
        );
    }

    #[test]
    fn corrects_a_software() {
        assert_suggestion_result(
            "HGroup-DIA, a software for analyzing multiple DIA data files.",
            NounCountability::default(),
            "HGroup-DIA, a software package for analyzing multiple DIA data files.",
        );
    }

    #[test]
    fn corrects_a_software_with_modifiers() {
        assert_suggestion_result(
            "bibisco is a novel writing software that helps you plan, organize and write your story without getting lost.",
            NounCountability::default(),
            "bibisco is a novel writing program that helps you plan, organize and write your story without getting lost.",
        );
    }

    #[test]
    fn preserves_modifiers_in_software_suggestions() {
        assert_suggestion_result(
            "They installed a useful software.",
            NounCountability::default(),
            "They installed useful software.",
        );
        assert_suggestion_result(
            "They installed a useful software.",
            NounCountability::default(),
            "They installed some useful software.",
        );
        assert_suggestion_result(
            "They tested one useful software.",
            NounCountability::default(),
            "They tested one piece of useful software.",
        );
    }

    #[test]
    fn corrects_hyphenated_modifiers() {
        assert_suggestion_result(
            "It is a novel-writing software.",
            NounCountability::default(),
            "It is a novel-writing program.",
        );
    }

    #[test]
    fn corrects_software_before_relative_clause() {
        assert_suggestion_result(
            "This is a software you can use.",
            NounCountability::default(),
            "This is a program you can use.",
        );
    }

    #[test]
    fn dont_flag_modified_compound_nouns() {
        assert_no_lints(
            "It is a novel writing software tool.",
            NounCountability::default(),
        );
        assert_no_lints(
            "She is a software engineering manager.",
            NounCountability::default(),
        );
        assert_no_lints(
            "They operate a custom software development platform.",
            NounCountability::default(),
        );
    }

    #[test]
    fn corrects_a_luggage() {
        assert_suggestion_result(
            "A luggage with a little engine, sensors (gps, ultrasounds, etc...) and bluetooth connection that will follow you everywhere.",
            NounCountability::default(),
            "A suitcase with a little engine, sensors (gps, ultrasounds, etc...) and bluetooth connection that will follow you everywhere.",
        );
    }

    #[test]
    fn corrects_multiple_advice() {
        assert_suggestion_result(
            "Update Advice API doc for event and data params, multiple advice.",
            NounCountability::default(),
            "Update Advice API doc for event and data params, multiple suggestions.",
        );
    }

    #[test]
    fn corrects_every_software() {
        assert_suggestion_result(
            "Rewrite every software known to man in Rust.",
            NounCountability::default(),
            "Rewrite every application known to man in Rust.",
        );
    }

    #[test]
    fn corrects_each_furniture() {
        assert_suggestion_result(
            "the position (x, y) and size (height, width, length) of each furniture",
            NounCountability::default(),
            "the position (x, y) and size (height, width, length) of each piece of furniture",
        );
    }

    #[test]
    fn corrects_one_clothing() {
        assert_suggestion_result(
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
        assert_suggestion_result(
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
        assert_suggestion_result(
            "Not in this form because it currently works with one punctuation with one letter either side.",
            NounCountability::default(),
            "Not in this form because it currently works with one punctuation mark with one letter either side.",
        );
    }
}
