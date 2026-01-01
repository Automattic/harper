use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct PluralWrongWordOfPhrase {
    expr: Box<dyn Expr>,
}

const PAIRS: &[(&str, &[&str])] = &[
    ("line", &["code"]),
    ("part", &["speech", "speeches"]),
    ("point", &["view"]),
    ("rule", &["thumb"]),
];

impl Default for PluralWrongWordOfPhrase {
    fn default() -> Self {
        let mut mistakes = vec![];

        for &(main_sg, last_noun) in PAIRS {
            let main_pl = format!("{}s", main_sg);
            let last_pl = if last_noun.len() == 2 {
                last_noun[1].to_string()
            } else {
                format!("{}s", last_noun[0])
            };

            let word_str = |w| {
                SequenceExpr::default().then(move |t: &Token, s: &[char]| {
                    t.kind.is_word() && t.span.get_content(s).eq_ignore_ascii_case_str(w)
                })
            };

            let word_string = |w: String| {
                SequenceExpr::default().then(move |t: &Token, s: &[char]| {
                    t.kind.is_word() && t.span.get_content(s).eq_ignore_ascii_case_str(&w)
                })
            };

            mistakes.push(Box::new(
                SequenceExpr::any_of(vec![
                    Box::new(word_str(main_sg)),
                    Box::new(word_string(main_pl)),
                ])
                .t_ws_h()
                .t_aco("of")
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

        if let Some((main_noun, last_noun)) = PAIRS.iter().find(|(main, last)| {
            main_noun_chars.starts_with_ignore_ascii_case_str(main)
                && last_noun_chars.starts_with_ignore_ascii_case_str(last[0])
        }) {
            return Some(Lint {
                lint_kind: LintKind::Usage,
                span: toks.span()?,
                suggestions: vec![Suggestion::replace_with_match_case(
                    format!("{}s of {}", *main_noun, last_noun[0])
                        .chars()
                        .collect::<Vec<char>>(),
                    toks.span()?.get_content(src),
                )],
                message: "This phrase is pluralized on the main noun, not on the last noun."
                    .to_string(),
                ..Default::default()
            });
        }
        None
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

    // PartOfSpeech
    #[test]
    fn corrects_part_of_speeches() {
        assert_suggestion_result(
            "The part of speeches (POS) or as follows:",
            PluralWrongWordOfPhrase::default(),
            "The parts of speech (POS) or as follows:",
        )
    }

    // It can connect different parts of speeches e.g noun to adjective, adjective to adverb, noun to verb etc.
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

    // log events, places, moods and self-reflect from various points of views
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
}
