use super::{Lint, LintKind, PatternLinter, Suggestion};
use crate::{
    Lrc, Token, TokenKind, TokenStringExt,
    patterns::{All, EitherPattern, Pattern, SequencePattern, Word},
};

pub struct Everyday {
    pattern: Box<dyn Pattern>,
}

// TODO .is_present_tense_verb() is currently broken
// TODO it returns true for -s 3rd pers. sing. pres.
// TODO and for -ing continuous/progressive forms, which are not present-only
// TODO English doesn't have a morphological way to tell
// TODO the difference between present tense, infinitive, future tense, etc.
// TODO Switch to use the .is_progressive_form() method when it's merged
fn is_progressive_form(tok: &Token, src: &[char]) -> bool {
    tok.kind.is_verb()
        && tok.kind.is_present_tense_verb()
        && tok
            .span
            .get_content_string(src)
            .to_lowercase()
            .ends_with("ing")
}

fn is_article(tok: &Token, src: &[char]) -> bool {
    matches!(
        tok.span.get_content_string(src).to_lowercase().as_str(),
        "a" | "an" | "the"
    )
}

fn is_unknown_word(tok: &Token) -> bool {
    matches!(&tok.kind, TokenKind::Word(None))
}

impl Default for Everyday {
    fn default() -> Self {
        let everyday = Word::new("everyday");
        let every_day = Lrc::new(
            SequencePattern::default()
                .t_aco("every")
                .t_ws()
                .t_aco("day"),
        );

        let everyday_bad_after = All::new(vec![
            Box::new(
                SequencePattern::default()
                    .then(everyday.clone())
                    .t_ws()
                    .then_any_word(),
            ),
            Box::new(SequencePattern::default().t_any().t_any().then(
                |tok: &Token, src: &[char]| {
                    !tok.kind.is_noun() && !is_unknown_word(tok) && !is_progressive_form(tok, src)
                },
            )),
        ]);

        let bad_before_every_day = All::new(vec![
            Box::new(
                SequencePattern::default()
                    .then_any_word()
                    .t_ws()
                    .then(every_day.clone()),
            ),
            Box::new(|tok: &Token, src: &[char]| is_article(tok, src)),
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
            pattern: Box::new(EitherPattern::new(vec![
                Box::new(everyday_bad_after),
                Box::new(bad_before_every_day),
            ])),
        }
    }
}

impl PatternLinter for Everyday {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // Helper functions make the match tables more compact and readable.
        let norm = |i: usize| toks[i].span.get_content_string(src).to_lowercase();
        let isws = |i: usize| toks[i].kind.is_whitespace();
        let span3 = |i: usize| toks[i].span;
        let span5 = |i: usize| toks[i..i + 3].span().unwrap();

        let (span, replacement, pos) = match toks.len() {
            3 => match (norm(0).as_str(), norm(2).as_str()) {
                ("everyday", _) if isws(1) => Some((span3(0), "every day", "adverb")),
                (_, "everyday") if isws(1) => Some((span3(2), "every day", "adverb")),
                _ => None,
            },
            5 => match (norm(0).as_str(), norm(2).as_str(), norm(4).as_str()) {
                ("every", "day", _) if isws(1) && isws(3) => {
                    Some((span5(0), "everyday", "adjective"))
                }
                (_, "every", "day") if isws(1) && isws(3) => {
                    Some((span5(2), "everyday", "adjective"))
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
            message: format!("You probably mean the {} `{}` here.", pos, replacement),
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
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn dont_flag_lone_adjective() {
        assert_lint_count("everyday", Everyday::default(), 0);
    }

    #[test]
    fn dont_flag_lone_adverb() {
        assert_lint_count("every day", Everyday::default(), 0);
    }

    #[test]
    #[ignore = "Can't yet match end-of-chunk after it. Verb before is legit for both adjective and adverb."]
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
    #[ignore = "Can't yet match end-of-chunk after it. Past verb before is legit for both adjective and adverb."]
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
}
