use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::expr::All;
use crate::expr::Expr;
use crate::expr::LongestMatchOf;
use crate::expr::SequenceExpr;
use crate::linting::expr_linter::Chunk;
use crate::{Lrc, Punctuation, Token, TokenKind, TokenStringExt, patterns::Word};

pub struct Everyday {
    expr: Box<dyn Expr>,
}

impl Default for Everyday {
    fn default() -> Self {
        let everyday = Word::new("everyday");
        let every_day = Lrc::new(SequenceExpr::aco("every").t_ws().t_aco("day"));

        let everyday_bad_after = All::new(vec![
            Box::new(SequenceExpr::with(everyday.clone()).t_ws().then_any_word()),
            Box::new(SequenceExpr::anything().t_any().then_kind_where(|kind| {
                !kind.is_noun() && !kind.is_oov() && !kind.is_verb_progressive_form()
            })),
        ]);

        let bad_before_every_day = All::new(vec![
            Box::new(SequenceExpr::any_word().t_ws().then(every_day.clone())),
            Box::new(|tok: &Token, _src: &[char]| {
                // "this" and "that" are both determiners and pronouns
                tok.kind.is_determiner() && !tok.kind.is_pronoun()
            }),
        ]);

        // (why does) everyday feel the (same ?)
        let everyday_ambiverb_after_then_noun = All::new(vec![
            Box::new(
                SequenceExpr::with(everyday.clone())
                    .t_ws()
                    .then_any_word()
                    .t_ws()
                    .then_any_word(),
            ),
            Box::new(
                SequenceExpr::anything()
                    .t_any()
                    .then_kind_both(TokenKind::is_noun, TokenKind::is_verb)
                    .t_any()
                    .then_determiner(),
            ),
        ]);

        // (Do you actually improve if you draw) everyday?
        let everyday_punctuation_after = All::new(vec![
            Box::new(SequenceExpr::with(everyday.clone()).then_punctuation()),
            Box::new(SequenceExpr::anything().then_kind_where(|kind| {
                matches!(
                    kind,
                    TokenKind::Punctuation(
                        Punctuation::Question | Punctuation::Comma | Punctuation::Period
                    )
                )
            })),
        ]);

        // (However, the message goes far beyond) every day things.
        let every_day_noun_after_then_punctuation = All::new(vec![
            Box::new(
                SequenceExpr::with(every_day.clone())
                    .t_ws()
                    .then_plural_noun()
                    .then_punctuation(),
            ),
            Box::new(
                SequenceExpr::anything()
                    .t_any()
                    .t_any()
                    .t_any()
                    .t_any()
                    .then_kind_where(|kind| {
                        matches!(
                            kind,
                            TokenKind::Punctuation(
                                Punctuation::Question | Punctuation::Comma | Punctuation::Period
                            )
                        )
                    }),
            ),
        ]);

        // Can we detect all mistakes with just one token before or after?

        // ❌ after adjective ✅ after adverb
        // $ (end of chunk)

        // ✅ after adjective ❌ after adverb
        // singular count noun: "An everyday task"

        // ✅ after adjective ✅ after adverb - can't disambiguate!
        // plural noun: "Everyday tasks are boring." vs "Every day tasks get completed."
        // mass noun: "Everyday information" vs "Every day information gets processed."

        // ❌ before adjective ✅ before adverb
        // none found yet

        // ✅ before adjective ❌ before adverb
        // none found yet

        // ✅ before adjective ✅ before adverb - can't disambiguate!
        // "some": "some everyday tasks" / "Do some every day"
        // verb, past form: "I coded every day" / "I learned everyday phrases"

        Self {
            expr: Box::new(LongestMatchOf::new(vec![
                Box::new(everyday_bad_after),
                Box::new(bad_before_every_day),
                Box::new(everyday_ambiverb_after_then_noun),
                Box::new(everyday_punctuation_after),
                Box::new(every_day_noun_after_then_punctuation),
            ])),
        }
    }
}

impl ExprLinter for Everyday {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // Helper functions make the match tables more compact and readable.
        let norm = |i: usize| toks[i].span.get_content_string(src).to_lowercase();
        let isws = |i: usize| toks[i].kind.is_whitespace();
        let tokspan = |i: usize| toks[i].span;
        let slicespan = |i: usize| toks[i..i + 3].span().unwrap();

        let (span, replacement, pos) = match toks.len() {
            2 => match (norm(0).as_str(), norm(1).as_str()) {
                ("everyday", _) => Some((tokspan(0), "every day", "adverb")),
                _ => None,
            },
            3 => match (norm(0).as_str(), norm(2).as_str()) {
                ("everyday", _) if isws(1) => Some((tokspan(0), "every day", "adverb")),
                (_, "everyday") if isws(1) => Some((tokspan(2), "every day", "adverb")),
                _ => None,
            },
            5 => match (norm(0).as_str(), norm(2).as_str(), norm(4).as_str()) {
                ("every", "day", _) if isws(1) && isws(3) => {
                    Some((slicespan(0), "everyday", "adjective"))
                }
                (_, "every", "day") if isws(1) && isws(3) => {
                    Some((slicespan(2), "everyday", "adjective"))
                }
                ("everyday", _, _) if isws(1) && isws(3) => {
                    Some((tokspan(0), "every day", "adverb"))
                }
                _ => None,
            },
            6 => match (
                norm(0).as_str(),
                norm(2).as_str(),
                norm(4).as_str(),
                norm(5).as_str(),
            ) {
                ("every", "day", _, _) if isws(1) && isws(3) => {
                    Some((slicespan(0), "everyday", "adjective"))
                }
                _ => None,
            },
            _ => None,
        }?;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                replacement,
                span.get_content(src),
            )],
            message: format!("You probably mean the {pos} `{replacement}` here."),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "This rule tries to sort out confusing the adjective `everyday` and the adverb `every day`."
    }
}

#[cfg(test)]
mod tests {
    use super::Everyday;
    use crate::linting::tests::{
        assert_lint_count, assert_no_lints, assert_suggestion_result, assert_top3_suggestion_result,
    };

    #[test]
    fn dont_flag_lone_adjective() {
        assert_lint_count("everyday", Everyday::default(), 0);
    }

    #[test]
    fn dont_flag_lone_adverb() {
        assert_lint_count("every day", Everyday::default(), 0);
    }

    #[test]
    fn correct_adjective_at_end_of_chunk() {
        assert_suggestion_result(
            "This is something I do everyday.",
            Everyday::default(),
            "This is something I do every day.",
        );
    }

    #[test]
    fn correct_adverb_after_article_before_noun() {
        assert_suggestion_result(
            "It's nothing special, just an every day thing.",
            Everyday::default(),
            "It's nothing special, just an everyday thing.",
        );
    }

    #[test]
    #[ignore = "Can't yet match end-of-chunk after it. Adjective before is legit for both adjective and adverb."]
    fn correct_adjective_without_following_noun() {
        assert_suggestion_result(
            "Some git commands used everyday",
            Everyday::default(),
            "Some git commands used every day",
        );
    }

    #[test]
    fn dont_flag_everyday_adjective_before_dev() {
        assert_lint_count(
            "At everyday dev, engineering isn't just a job - it's our passion.",
            Everyday::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_everyday_adjective_before_present_participle() {
        assert_lint_count("Everyday coding projects.", Everyday::default(), 0);
    }

    #[test]
    fn dont_flag_everyday_adjective_before_plural_noun() {
        assert_lint_count(
            "Exploring Everyday Things with R and Ruby",
            Everyday::default(),
            0,
        );
    }

    #[test]
    fn correct_everyday_at_end_of_sentence_after_past_verb() {
        assert_suggestion_result(
            "Trying to write about what I learned everyday.",
            Everyday::default(),
            "Trying to write about what I learned every day.",
        );
    }

    #[test]
    fn dont_flag_every_day_at_start_of_sentence_before_comma() {
        assert_lint_count(
            "Every day, a new concept or improvement will be shared",
            Everyday::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_every_day_at_start_of_sentence_before_copula() {
        assert_lint_count("Every day is worth remembering...", Everyday::default(), 0);
    }

    #[test]
    fn dont_flag_every_day_at_end_of_sentence_after_noun() {
        assert_lint_count("You learn new stuff every day.", Everyday::default(), 0);
    }

    #[test]
    fn dont_flag_every_day_after_noun_before_conjunction() {
        assert_lint_count(
            "Pick a different test item every day and confirm it is present.",
            Everyday::default(),
            0,
        );
    }

    #[test]
    #[ignore = "replace_with_match_case_str converts to EveryDay instead of Everyday"]
    fn correct_every_day_after_article() {
        assert_suggestion_result(
            "The Every Day Calendar with Dark Mode",
            Everyday::default(),
            "The Everyday Calendar with Dark Mode",
        );
    }

    #[test]
    fn dont_flag_everyday_before_unknown_word() {
        assert_lint_count(
            "It's just a normal everyday splorg.",
            Everyday::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_every_day_at_end_of_chunk_after_adverb() {
        assert_lint_count(
            "I use the same amount of energy basically every day",
            Everyday::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_every_day_after_verb_before_if() {
        assert_lint_count(
            "This would happen every day if left alone.",
            Everyday::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_every_day_after_noun_before_preposition() {
        assert_lint_count(
            "An animal can do training and inference every day of its existence until the day of its death.",
            Everyday::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_every_day_after_time() {
        assert_lint_count(
            "Can I take a picture at 12:00 every day?",
            Everyday::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_every_day_at_start_of_chunk_before_np() {
        assert_lint_count(
            "Every day the application crashes several times on macOS Sequoia version 15.3",
            Everyday::default(),
            0,
        );
    }

    #[test]
    fn fix_everyday_and_every_day_used_wrongly() {
        assert_top3_suggestion_result(
            "Each and everyday you ought to strive to learn something that is not an every day thing.",
            Everyday::default(),
            "Each and every day you ought to strive to learn something that is not an everyday thing.",
        );
    }

    #[test]
    fn fix_reddit_why_does_everyday() {
        assert_top3_suggestion_result(
            "Why does everyday feel the same?",
            Everyday::default(),
            "Why does every day feel the same?",
        );
    }

    #[test]
    fn fix_reddit_everyday_is_going_to() {
        assert_top3_suggestion_result(
            "... everyday is going to be a good day that's just the way it is!",
            Everyday::default(),
            "... every day is going to be a good day that's just the way it is!",
        );
    }

    #[test]
    fn fix_reddit_draw_everyday() {
        assert_top3_suggestion_result(
            "Do you actually improve if you draw everyday?",
            Everyday::default(),
            "Do you actually improve if you draw every day?",
        );
    }

    #[test]
    fn fix_reddit_two_bad_out_of_three() {
        assert_top3_suggestion_result(
            "Yes you can jog everyday, not a personal best every day, but a steady pace run everyday.",
            Everyday::default(),
            "Yes you can jog every day, not a personal best every day, but a steady pace run every day.",
        );
    }

    #[test]
    fn fix_reddit_every_day_routine() {
        assert_top3_suggestion_result(
            "Habit stacking - stacking the small skill with something that's already worked into my every day routine.",
            Everyday::default(),
            "Habit stacking - stacking the small skill with something that's already worked into my everyday routine.",
        );
    }

    #[test]
    fn fix_stackoverflow_every_day_things() {
        assert_top3_suggestion_result(
            "However, the message goes far beyond every day things.",
            Everyday::default(),
            "However, the message goes far beyond everyday things.",
        );
    }

    #[test]
    fn fix_reddit_everyday_is_same() {
        assert_top3_suggestion_result(
            "Everyday is exactly the same",
            Everyday::default(),
            "Every day is exactly the same",
        );
    }

    #[test]
    #[ignore = "doesn't work yet because title case demands 'Every Day' but we get 'Every day'"]
    fn fix_medium_little_bit_everyday() {
        assert_top3_suggestion_result(
            "Does Learning A Little Bit Everyday Actually Work?",
            Everyday::default(),
            "Does Learning A Little Bit Every Day Actually Work?",
        );
    }

    #[test]
    fn fix_stackexchange_use_everyday() {
        assert_top3_suggestion_result(
            "We use this everyday without noticing, but we hate it when ...",
            Everyday::default(),
            "We use this every day without noticing, but we hate it when ...",
        );
    }

    #[test]
    fn fix_github_what_i_learned_everyday() {
        assert_top3_suggestion_result(
            "Trying to write about what I learned everyday.",
            Everyday::default(),
            "Trying to write about what I learned every day.",
        );
    }

    #[test]
    fn fix_medium_one_bad_out_of_three() {
        assert_top3_suggestion_result(
            "Even inside a routine, everyday we adapt to changes and challenges ... We are not the same person every day, but every day we are ourselves…",
            Everyday::default(),
            "Even inside a routine, every day we adapt to changes and challenges ... We are not the same person every day, but every day we are ourselves…",
        );
    }

    #[test]
    fn fix_medium_doing_something_everyday() {
        assert_top3_suggestion_result(
            "There was nothing wrong with my braincells processing the concepts of doing something everyday and ...",
            Everyday::default(),
            "There was nothing wrong with my braincells processing the concepts of doing something every day and ...",
        );
    }

    #[test]
    fn fix_medium_all_caps() {
        assert_top3_suggestion_result(
            "MEET SOMEONE NEW EVERYDAY.",
            Everyday::default(),
            "MEET SOMEONE NEW EVERY DAY.",
        );
    }

    #[test]
    fn dont_flag_every_day_singular_noun_2020() {
        assert_no_lints("50 requests per day, every day free.", Everyday::default());
    }
}
