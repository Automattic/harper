use itertools::Itertools;

use crate::{Document, Punctuation};

use super::{Lint, LintKind, Linter, Suggestion};

pub struct MissingSpace;

impl Linter for MissingSpace {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for (a, b, c) in document.tokens().tuple_windows() {
            if let Some(punct) = b.kind.as_punctuation()
                && [
                    Punctuation::Period,
                    Punctuation::Bang,
                    Punctuation::Question,
                    Punctuation::Semicolon,
                ]
                .contains(punct)
                && a.kind.is_word()
                && c.kind.is_word()
            {
                lints.push(Lint {
                    span: b.span,
                    lint_kind: LintKind::Formatting,
                    suggestions: vec![Suggestion::InsertAfter(vec![' '])],
                    message: "It looks like you're missing a space here.".to_owned(),
                    priority: 31,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Looks for missing spaces after a comma or period."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::MissingSpace;

    #[test]
    fn issue_2191() {
        assert_suggestion_result(
            "people that can help us.So I feel like there",
            MissingSpace,
            "people that can help us. So I feel like there",
        );
    }

    #[test]
    fn coffee_table() {
        assert_suggestion_result(
            "The coffee cooled on the table.The room stayed quiet.",
            MissingSpace,
            "The coffee cooled on the table. The room stayed quiet.",
        );
    }

    #[test]
    fn open_window() {
        assert_suggestion_result(
            "A small breeze moved through the open window.The curtains lifted and fell in slow waves.",
            MissingSpace,
            "A small breeze moved through the open window. The curtains lifted and fell in slow waves.",
        );
    }

    #[test]
    fn hallway_cat() {
        assert_suggestion_result(
            "The cat watched the hallway.Its tail twitched with steady focus.",
            MissingSpace,
            "The cat watched the hallway. Its tail twitched with steady focus.",
        );
    }

    #[test]
    fn rain_glass() {
        assert_suggestion_result(
            "Rain tapped against the glass.The sound made the afternoon feel longer.",
            MissingSpace,
            "Rain tapped against the glass. The sound made the afternoon feel longer.",
        );
    }

    #[test]
    fn cyclist_house() {
        assert_suggestion_result(
            "A cyclist passed by the house.The wheels hummed softly on the road.",
            MissingSpace,
            "A cyclist passed by the house. The wheels hummed softly on the road.",
        );
    }

    #[test]
    fn kettle_stove() {
        assert_suggestion_result(
            "The kettle hissed on the stove.A thin ribbon of steam curled toward the ceiling.",
            MissingSpace,
            "The kettle hissed on the stove. A thin ribbon of steam curled toward the ceiling.",
        );
    }

    #[test]
    fn sparrow_fence() {
        assert_suggestion_result(
            "A sparrow landed on the fence.Its wings fluttered once before it settled.",
            MissingSpace,
            "A sparrow landed on the fence. Its wings fluttered once before it settled.",
        );
    }

    #[test]
    fn streetlamp_dusk() {
        assert_suggestion_result(
            "The streetlamp flickered at dusk.A pale glow spread across the sidewalk.",
            MissingSpace,
            "The streetlamp flickered at dusk. A pale glow spread across the sidewalk.",
        );
    }

    #[test]
    fn distant_laughter() {
        assert_suggestion_result(
            "Someone laughed in the distance.The echo drifted between the buildings.",
            MissingSpace,
            "Someone laughed in the distance. The echo drifted between the buildings.",
        );
    }

    #[test]
    fn notebook_desk() {
        assert_suggestion_result(
            "A notebook lay open on the desk.Its blank pages waited for a pen.",
            MissingSpace,
            "A notebook lay open on the desk. Its blank pages waited for a pen.",
        );
    }

    #[test]
    fn question_mark_mid_sentence() {
        assert_suggestion_result(
            "Where are you?I looked around the room.",
            MissingSpace,
            "Where are you? I looked around the room.",
        );
    }

    #[test]
    fn question_mark_before_name() {
        assert_suggestion_result(
            "Are you coming?Elijah is already waiting.",
            MissingSpace,
            "Are you coming? Elijah is already waiting.",
        );
    }

    #[test]
    fn exclamation_mid_sentence() {
        assert_suggestion_result(
            "The door slammed shut!Everyone in the hall jumped.",
            MissingSpace,
            "The door slammed shut! Everyone in the hall jumped.",
        );
    }

    #[test]
    fn exclamation_before_clause() {
        assert_suggestion_result(
            "You actually solved it!That changes everything.",
            MissingSpace,
            "You actually solved it! That changes everything.",
        );
    }

    #[test]
    fn semicolon_before_adverb() {
        assert_suggestion_result(
            "He wanted to leave;however, he stayed until the end.",
            MissingSpace,
            "He wanted to leave; however, he stayed until the end.",
        );
    }

    #[test]
    fn semicolon_connecting_clauses() {
        assert_suggestion_result(
            "The night was cold;stars glittered above the dark field.",
            MissingSpace,
            "The night was cold; stars glittered above the dark field.",
        );
    }
}
