use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct PluralWrongWordOfPhrase {
    expr: Box<dyn Expr>,
}

// If a noun needs other than an -s suffix to be pluralized, include it as the 2nd array element.
const PATTERNS: &[(&[&str], &str, &[&str])] = &[
    (&["body", "bodies"], "in", &["white"]),
    (&["flash", "flashes"], "in the", &["pan"]),
    (&["flash", "flashes"], "in-the", &["pan"]),
    (&["line"], "of", &["code"]),
    (&["part"], "of", &["speech", "speeches"]),
    (&["point"], "of", &["view"]),
    (&["rule"], "of", &["thumb"]),
];

impl Default for PluralWrongWordOfPhrase {
    fn default() -> Self {
        let word_str = |w| {
            SequenceExpr::with(move |t: &Token, s: &[char]| {
                t.kind.is_word() && t.span.get_content(s).eq_ignore_ascii_case_str(w)
            })
        };

        let word_string = |w: String| {
            SequenceExpr::with(move |t: &Token, s: &[char]| {
                t.kind.is_word() && t.span.get_content(s).eq_ignore_ascii_case_str(&w)
            })
        };

        let mut mistakes = vec![];

        for &(main_noun, mid, last_noun) in PATTERNS {
            let main_pl = if main_noun.len() == 2 {
                main_noun[1].to_string()
            } else {
                format!("{}s", main_noun[0])
            };
            let last_pl = if last_noun.len() == 2 {
                last_noun[1].to_string()
            } else {
                format!("{}s", last_noun[0])
            };

            mistakes.push(Box::new(
                SequenceExpr::any_of(vec![
                    Box::new(word_str(main_noun[0])),
                    Box::new(word_string(main_pl)),
                ])
                .t_ws_h()
                .then_fixed_phrase(mid)
                .t_ws_h()
                .then(word_string(last_pl)),
            ) as Box<dyn Expr>);
        }

        Self {
            expr: Box::new(SequenceExpr::any_of(mistakes)),
        }
    }
}

impl ExprLinter for PluralWrongWordOfPhrase {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Corrects noun phrases that pluralize the last noun instead of the main noun."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (main_noun_tok, last_noun_tok) = (toks.first()?, toks.last()?);
        let (main_noun_span, last_noun_span) = (main_noun_tok.span, last_noun_tok.span);
        let (main_noun_chars, last_noun_chars) = (
            main_noun_span.get_content(src),
            last_noun_span.get_content(src),
        );

        let (main_noun, mid, last_noun) = PATTERNS.iter().find(|(main, _, last)| {
            main_noun_chars.starts_with_ignore_ascii_case_str(main[0])
                && last_noun_chars.starts_with_ignore_ascii_case_str(last[0])
        })?;

        let main_noun_pl = if main_noun.len() == 2 {
            main_noun[1].to_string()
        } else {
            format!("{}s", main_noun[0])
        };

        Some(Lint {
            lint_kind: LintKind::Usage,
            span: toks.span()?,
            suggestions: vec![Suggestion::replace_with_match_case(
                format!("{} {} {}", main_noun_pl, mid, last_noun[0])
                    .chars()
                    .collect::<Vec<char>>(),
                toks.span()?.get_content(src),
            )],
            message: "This phrase is pluralized on the main noun, not on the last noun."
                .to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::PluralWrongWordOfPhrase;
    use crate::linting::tests::assert_suggestion_result;

    // LinesOfCode

    #[test]
    fn corrects_line_of_codes() {
        assert_suggestion_result(
            "desktop application used to estimate the line of codes to certain software application",
            PluralWrongWordOfPhrase::default(),
            "desktop application used to estimate the lines of code to certain software application",
        );
    }

    #[test]
    #[ignore = "Wrong letters capitalized due to how `Suggstion::replace_with_match_case` works by index."]
    fn corrects_line_of_codes_title_case() {
        assert_suggestion_result(
            "A simple tool for Line Of Codes (LOC) calculation.",
            PluralWrongWordOfPhrase::default(),
            "A simple tool for Lines Of Code (LOC) calculation.",
        );
    }

    #[test]
    fn corrects_lines_of_codes() {
        assert_suggestion_result(
            "I myself don't have something against giving users the ability to show the lines of codes they wrote.",
            PluralWrongWordOfPhrase::default(),
            "I myself don't have something against giving users the ability to show the lines of code they wrote.",
        );
    }

    // PartsOfSpeech

    #[test]
    fn corrects_part_of_speeches() {
        assert_suggestion_result(
            "The part of speeches (POS) or as follows:",
            PluralWrongWordOfPhrase::default(),
            "The parts of speech (POS) or as follows:",
        )
    }

    #[test]
    fn corrects_parts_of_speeches() {
        assert_suggestion_result(
            "It can connect different parts of speeches e.g noun to adjective, adjective to adverb, noun to verb etc.",
            PluralWrongWordOfPhrase::default(),
            "It can connect different parts of speech e.g noun to adjective, adjective to adverb, noun to verb etc.",
        )
    }

    // PointsOfView

    #[test]
    fn corrects_point_of_views() {
        assert_suggestion_result(
            "This will produce a huge amount of raw data, representing the region in multiple point of views.",
            PluralWrongWordOfPhrase::default(),
            "This will produce a huge amount of raw data, representing the region in multiple points of view.",
        )
    }

    #[test]
    fn corrects_points_of_views() {
        assert_suggestion_result(
            "log events, places, moods and self-reflect from various points of views",
            PluralWrongWordOfPhrase::default(),
            "log events, places, moods and self-reflect from various points of view",
        )
    }

    // RulesOfThumb

    #[test]
    fn correct_rule_of_thumbs() {
        assert_suggestion_result(
            "Thanks. 0.2 is just from my rule of thumbs.",
            PluralWrongWordOfPhrase::default(),
            "Thanks. 0.2 is just from my rules of thumb.",
        );
    }

    #[test]
    fn correct_rules_of_thumbs() {
        assert_suggestion_result(
            "But as rules of thumbs, what is said in config file should be respected whatever parameter (field or directory) is passed to php-cs-fixer.phar.",
            PluralWrongWordOfPhrase::default(),
            "But as rules of thumb, what is said in config file should be respected whatever parameter (field or directory) is passed to php-cs-fixer.phar.",
        );
    }

    #[test]
    fn correct_rules_of_thumbs_hyphenated() {
        assert_suggestion_result(
            "Add rule-of-thumbs for basic metrics, like \"Spill more than 1GB is a red flag\".",
            PluralWrongWordOfPhrase::default(),
            "Add rules of thumb for basic metrics, like \"Spill more than 1GB is a red flag\".",
        );
    }

    // BodiesInWhite

    #[test]
    fn correct_body_in_whites_1() {
        assert_suggestion_result(
            "Normally, when they manufacture these body in whites, they would spot weld a lot of the components on.",
            PluralWrongWordOfPhrase::default(),
            "Normally, when they manufacture these bodies in white, they would spot weld a lot of the components on.",
        );
    }

    #[test]
    fn correct_body_in_whites_2() {
        assert_suggestion_result(
            "I'm not sure, but just having seen a lot of body in whites, I know normally they try to spot weld it.",
            PluralWrongWordOfPhrase::default(),
            "I'm not sure, but just having seen a lot of bodies in white, I know normally they try to spot weld it.",
        );
    }

    // FlashesInThePan

    #[test]
    fn correct_flash_in_the_pans() {
        assert_suggestion_result(
            "I wish they do more flash in the pans, like the suggestions they do on ERB2 could be such a good way to see if these suggestions are worthy.",
            PluralWrongWordOfPhrase::default(),
            "I wish they do more flashes in the pan, like the suggestions they do on ERB2 could be such a good way to see if these suggestions are worthy.",
        );
    }

    #[test]
    fn correct_flash_in_the_pans_hyphenated() {
        assert_suggestion_result(
            "what makes 'Super Hexagon' rise above other nostalgic flash-in-the-pans is that there is a game to be learned here",
            PluralWrongWordOfPhrase::default(),
            "what makes 'Super Hexagon' rise above other nostalgic flashes in the pan is that there is a game to be learned here",
        );
    }

    #[test]
    fn correct_flashes_in_the_pans() {
        assert_suggestion_result(
            "Who are some of the biggest flashes in the pans in wrestling history?",
            PluralWrongWordOfPhrase::default(),
            "Who are some of the biggest flashes in the pan in wrestling history?",
        );
    }
}
