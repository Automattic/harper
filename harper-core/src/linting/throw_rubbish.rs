use super::{Lint, LintKind, Linter};
use crate::{Document, Span, TokenStringExt, linting::Suggestion};
use hashbrown::HashSet;
use lazy_static::lazy_static;

lazy_static! {
    static ref THROW: HashSet<&'static str> =
        HashSet::from(["throw", "throws", "threw", "thrown", "throwing"]);
}

lazy_static! {
    static ref JUNK: HashSet<&'static str> = HashSet::from(["rubbish", "trash", "garbage", "junk"]);
}

lazy_static! {
    static ref ADV_PREP: HashSet<&'static str> = HashSet::from([
        // adverbs
        "away",
        "out",
        "back",
        "everywhere",
        // prepositions
        "in",
        "into",
        "at",
        "on"
    ]);
}

#[derive(Debug, Default)]
pub struct ThrowRubbish;

impl Linter for ThrowRubbish {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            for verb_i in chunk.iter_verb_indices() {
                let verb_token = &chunk[verb_i];
                let verb_str = document
                    .get_span_content_str(&verb_token.span)
                    .to_lowercase();

                if !THROW.contains(verb_str.as_str()) {
                    continue;
                }

                let chunk_rest = &chunk[verb_i + 1..];
                let mut adv_prep_seen = false;
                let mut junk_seen = false;
                let mut last_i = None;

                for (rest_i, token) in chunk_rest.iter().enumerate() {
                    let chunk_i = verb_i + rest_i + 1;
                    let token_str = document.get_span_content_str(&token.span).to_lowercase();

                    if ADV_PREP.contains(token_str.as_str()) {
                        adv_prep_seen = true;
                        last_i = Some(chunk_i);
                        if junk_seen {
                            break;
                        }
                    }
                    if JUNK.contains(token_str.as_str()) {
                        // Check if this is being used as a qualifier for another noun
                        // by looking at the next token after any whitespace
                        if let Some(next_token) = document.get_next_word_from_offset(chunk_i, 1)
                            && next_token.kind.is_noun()
                            && !is_progressive_verb_form(document, next_token)
                        {
                            continue; // Skip if it's being used as an adjective
                        }
                        junk_seen = true;
                        last_i = Some(chunk_i);
                        if adv_prep_seen {
                            break;
                        }
                    }
                }

                if junk_seen && !adv_prep_seen {
                    let span = Span::new(chunk[verb_i].span.start, chunk[last_i.unwrap()].span.end);

                    let verb = document.get_span_content_str(&chunk[verb_i].span);

                    let rest = document.get_span_content_str(&Span::new(
                        chunk[verb_i].span.end,
                        chunk[last_i.unwrap()].span.end,
                    ));

                    // Generate all possible suggestions
                    let suggestions = ["away", "out"]
                        .iter()
                        .flat_map(|adv| {
                            [format!("{verb} {adv}{rest}"), format!("{verb}{rest} {adv}")]
                        })
                        .map(|sugg| Suggestion::ReplaceWith(sugg.chars().collect()))
                        .collect();

                    lints.push(Lint {
                        span,
                        lint_kind: LintKind::Miscellaneous,
                        suggestions,
                        message: "To dispose of rubbish we don't just throw it, we throw it away"
                            .to_string(),
                        priority: 63,
                    });
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks for throwing rubbish rather than throwing it away."
    }
}

// TODO replace this when we have proper support for progressive verb form in metadata
fn is_progressive_verb_form(document: &Document, token: &crate::Token) -> bool {
    token.kind.is_verb_progressive_form()
        && document
            .get_span_content_str(&token.span)
            .to_lowercase()
            .ends_with("ing")
}

#[cfg(test)]
mod tests {
    use super::ThrowRubbish;
    use crate::linting::tests::{assert_lint_count, assert_top3_suggestion_result};

    // Test correct patterns (should not trigger lint)
    #[test]
    fn allow_throw_away_rubbish() {
        assert_lint_count("Throw away the rubbish", ThrowRubbish, 0);
    }

    #[test]
    fn allow_throws_garbage_away() {
        assert_lint_count("He throws garbage away", ThrowRubbish, 0);
    }

    #[test]
    fn allow_threw_out_trash() {
        assert_lint_count("I threw out the trash", ThrowRubbish, 0);
    }

    #[test]
    fn allow_throw_junk_into() {
        assert_lint_count("Throw that junk into the bin!", ThrowRubbish, 0);
    }

    // Test incorrect patterns (should trigger lint)
    #[test]
    fn reject_throw_garbage() {
        assert_lint_count("You should throw garbage", ThrowRubbish, 1);
    }

    #[test]
    fn reject_throwing_rubbish() {
        assert_lint_count("Throwing rubbish is not a good idea", ThrowRubbish, 1);
    }

    // Test suggestions
    #[test]
    fn correct_thrown_some_trash() {
        assert_top3_suggestion_result(
            "I've thrown some trash",
            ThrowRubbish,
            "I've thrown some trash away",
        );
    }

    #[test]
    fn correct_throws_garbage() {
        assert_top3_suggestion_result(
            "That guy just throws his garbage",
            ThrowRubbish,
            "That guy just throws out his garbage",
        );
    }

    // Test edge cases
    #[test]
    fn ignore_throw_ball() {
        assert_lint_count("Can you throw the ball?", ThrowRubbish, 0);
    }

    // Sentences from GitHub
    #[test]
    fn correct_come_close_to_throw_trash() {
        assert_top3_suggestion_result(
            "Smart Dustbin is a trash bin that automatically opens when you come close to throw trash.",
            ThrowRubbish,
            "Smart Dustbin is a trash bin that automatically opens when you come close to throw away trash.",
        );
    }

    #[test]
    fn correct_thrown_rubbish() {
        assert_top3_suggestion_result(
            "Add a script that draws the bin behind thrown rubbish.",
            ThrowRubbish,
            "Add a script that draws the bin behind thrown away rubbish.",
        );
    }

    #[test]
    #[ignore = "`on` doesn't go with `throw` but with `daily basis`"]
    fn correct_encourage_people_to_throw_trash() {
        assert_top3_suggestion_result(
            "The app main goal is to encourage people to throw trash they can found on a daily basis.",
            ThrowRubbish,
            "The app main goal is to encourage people to throw away trash they can found on a daily basis.",
        );
    }

    #[test]
    fn correct_a_person_throwing_trash() {
        assert_top3_suggestion_result(
            "I think personally the icons look okay, aside from the clear prompt one, as it's currently accented on a person throwing trash.",
            ThrowRubbish,
            "I think personally the icons look okay, aside from the clear prompt one, as it's currently accented on a person throwing away trash.",
        );
    }

    #[test]
    fn allow_at_and_back() {
        assert_lint_count(
            "Throw garbage at a program, it will throw garbage back.",
            ThrowRubbish,
            0,
        );
    }

    #[test]
    fn correct_responsibly_throw_trash() {
        assert_top3_suggestion_result(
            "Reward system for people responsibly throwing trash saving the environment.",
            ThrowRubbish,
            "Reward system for people responsibly throwing away trash saving the environment.",
        );
    }

    // False positive when "rubbish" is a qualifier for another word
    #[test]
    fn dont_flag_throws_junk_errors() {
        assert_lint_count(
            "Experimental init throws junk errors, Ignore.",
            ThrowRubbish,
            0,
        );
    }

    #[test]
    fn dont_flag_throwing_garbage_data() {
        assert_lint_count(
            "I can resolve this in various ways, such as by not throwing garbage data at Typesense",
            ThrowRubbish,
            0,
        );
    }

    #[test]
    fn dont_flag_throwing_garbage_value() {
        assert_lint_count("Fix Spill tree Throwing garbage value", ThrowRubbish, 0);
    }

    #[test]
    fn correct_threw_trash_properly() {
        assert_top3_suggestion_result(
            "we want to know which student threw trash properly so that we can reward that student",
            ThrowRubbish,
            "we want to know which student threw away trash properly so that we can reward that student",
        );
    }

    #[test]
    fn dont_flag_throw_junk_bytes() {
        assert_lint_count(
            "the most efficient way to enforce the buffer size and throw junk bytes is to have a local (to the reception function) buffer",
            ThrowRubbish,
            0,
        );
    }

    #[test]
    fn dont_flag_throw_trash_everywhere() {
        assert_lint_count(
            "People throw trash everywhere and this tendency is very harmful.",
            ThrowRubbish,
            0,
        );
    }

    #[test]
    fn dont_flag_throws_garbage_comments() {
        assert_lint_count(
            "We dont need guys that throws garbage comments based on theyr lack of knowledge.",
            ThrowRubbish,
            0,
        );
    }

    #[test]
    fn dont_flag_trash_can_be_thrown_into_the_trash() {
        assert_lint_count(
            "Trash balls generated during cutting can be thrown into the trash",
            ThrowRubbish,
            0,
        );
    }

    #[test]
    fn correct_throwing_rubbish() {
        assert_top3_suggestion_result(
            "Admiring paintings, throwing rubbish, greeting.",
            ThrowRubbish,
            "Admiring paintings, throwing away rubbish, greeting.",
        );
    }
}
