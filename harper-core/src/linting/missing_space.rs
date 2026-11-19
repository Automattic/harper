use crate::{Document, Punctuation};

use super::{Lint, LintKind, Linter, Suggestion};

#[derive(Debug, Default)]
pub struct MissingSpace;

impl Linter for MissingSpace {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for (index, token) in document.tokens().enumerate() {
            let Some(punct) = token.kind.as_punctuation() else {
                continue;
            };

            let Some(next) = document.get_token_offset(index, 1) else {
                continue;
            };

            if ![
                Punctuation::Period,
                Punctuation::Bang,
                Punctuation::Question,
                Punctuation::Semicolon,
            ]
            .contains(punct)
                || !next.kind.is_word()
            {
                continue;
            }

            if punct == &Punctuation::Period {
                let next_word = document.get_span_content(&next.span);
                // All-caps suffixes can be filenames, domains, or dotfiles (PDF,
                // COM, DS_Store). Prefer missing an ambiguous sentence boundary
                // to inserting a space into a name. Single-letter words like I
                // and A can still begin sentences.
                if !next_word.first().is_some_and(|c| c.is_uppercase())
                    || (next_word.len() > 1 && !next_word.iter().any(|c| c.is_lowercase()))
                {
                    continue;
                }
            }

            let previous = document.get_token_offset(index, -1);
            let has_word_before = previous.is_some_and(|previous| previous.kind.is_word())
                || (previous.is_some_and(|previous| previous.kind.is_space())
                    && document
                        .get_token_offset(index, -2)
                        .is_some_and(|previous| previous.kind.is_word()));

            if !has_word_before {
                continue;
            }

            lints.push(Lint {
                span: token.span,
                lint_kind: LintKind::Formatting,
                suggestions: vec![Suggestion::InsertAfter(vec![' '])],
                message: "It looks like you're missing a space here.".to_owned(),
                priority: 31,
            });
        }

        lints
    }

    fn description(&self) -> &str {
        "Looks for missing spaces after periods, exclamation points, question marks, and semicolons."
    }
}

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::linting::Linter;
    use crate::linting::tests::{
        assert_markdown_suggestion_result, assert_no_lints, assert_suggestion_result,
    };

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
    fn issue_3800() {
        assert_suggestion_result(
            "The government .Once the policy changed, the program ended.",
            MissingSpace,
            "The government . Once the policy changed, the program ended.",
        );
    }

    #[test]
    fn allows_domain_names() {
        assert_no_lints("WordPress.com is a managed hosting provider.", MissingSpace);
    }

    #[test]
    fn allows_file_names() {
        assert_no_lints("Open composer.json to edit the dependencies.", MissingSpace);
    }

    #[test]
    fn allows_dotfiles() {
        assert_no_lints("Use the .harper file for configuration.", MissingSpace);
    }

    #[test]
    fn allows_uppercase_names() {
        for text in [
            "Open report.PDF to read the results.",
            "Open report.DOCX to edit the results.",
            "The photograph is saved as holiday.JPEG.",
            "Visit WordPress.COM for details.",
            "Visit EXAMPLE.ORG for details.",
            "Remove the .DS_Store file before committing.",
        ] {
            for document in [
                Document::new_plain_english_curated(text),
                Document::new_markdown_default_curated(text),
            ] {
                assert!(MissingSpace.lint(&document).is_empty(), "{text}");
            }
        }
    }

    #[test]
    fn retains_sentence_spacing_corrections() {
        for (text, expected) in [
            (
                "The door closed.I stayed outside.",
                "The door closed. I stayed outside.",
            ),
            (
                "The door closed.A key was missing.",
                "The door closed. A key was missing.",
            ),
            (
                "The door closed.I'm still outside.",
                "The door closed. I'm still outside.",
            ),
            (
                "The door closed .Once again, I was outside.",
                "The door closed . Once again, I was outside.",
            ),
            ("Who called?NASA called.", "Who called? NASA called."),
            (
                "They called!NASA needs help.",
                "They called! NASA needs help.",
            ),
            (
                "They called;NASA needs help.",
                "They called; NASA needs help.",
            ),
        ] {
            assert_suggestion_result(text, MissingSpace, expected);
            assert_markdown_suggestion_result(text, MissingSpace, expected);
        }
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
