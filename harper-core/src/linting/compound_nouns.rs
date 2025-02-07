use std::sync::Arc;

use itertools::Itertools;

use crate::{CharString, CharStringExt, Dictionary, Document, FstDictionary, Span, TokenStringExt};

use super::{Lint, LintKind, Linter, Suggestion};

pub struct CompoundNouns {
    dict: Arc<FstDictionary>,
}

impl CompoundNouns {
    pub fn new() -> Self {
        Self {
            dict: FstDictionary::curated(),
        }
    }

    /// Create a lint for two attempted lint matches.
    /// Since this function does not have access to tokens, you must set and modify the span
    /// yourself.
    fn attempt_lint_match(
        &self,
        word_a: &[char],
        word_b: &[char],
        buffer: &mut CharString,
    ) -> Option<Lint> {
        buffer.clear();
        buffer.extend_from_slice(word_a);
        buffer.extend_from_slice(word_b);

        if self.dict.contains_word(word_a)
            && self.dict.contains_word(word_b)
            && self.dict.get_word_metadata(buffer).is_noun()
        {
            Some(Lint {
                span: Span::default(),
                lint_kind: LintKind::Spelling,
                suggestions: vec![Suggestion::ReplaceWith(buffer.to_vec())],
                message: format!("Did you mean the closed compound “{}”?", buffer.to_string()),
                priority: 63,
            })
        } else {
            None
        }
    }
}

impl Default for CompoundNouns {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for CompoundNouns {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        // A persistent buffer for misc operations.
        let mut buffer = CharString::new();

        for (a, w, b) in document.tokens().tuple_windows() {
            if !a.kind.is_word() || !w.kind.is_whitespace() || !b.kind.is_word() {
                continue;
            }

            let a_meta = a.kind.expect_word();
            let b_meta = b.kind.expect_word();
            let mut a_chars: CharString = document.get_span_content(a.span).into();
            let mut b_chars: CharString = document.get_span_content(b.span).into();

            if a_chars.len() <= 1
                || b_chars.len() <= 1
                || matches!(b_chars.as_slice(), &['i', 's'])
                || a_meta.article
                || b_meta.article
            {
                continue;
            }

            let span = [a, w, b].span().unwrap();

            macro_rules! attempt {
                () => {
                    if let Some(mut lint) = self.attempt_lint_match(&a_chars, &b_chars, &mut buffer)
                    {
                        lint.span = span;
                        lints.push(lint);
                        continue;
                    }
                };
            }

            attempt!();
            a_chars[0] = a_chars[0].to_ascii_uppercase();
            attempt!();
            b_chars[0] = b_chars[0].to_ascii_uppercase();
            attempt!();
            a_chars[0] = a_chars[0].to_ascii_lowercase();
            attempt!();
            b_chars[0] = b_chars[0].to_ascii_lowercase();
            attempt!();
        }

        lints
    }

    fn description(&self) -> &str {
        "Accidentally inserting a space inside a word is common. This rule looks for valid compound nouns that are split by whitespace and whose components are also valid words."
    }
}

#[cfg(test)]
mod tests {
    use super::CompoundNouns;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn lap_top() {
        let test_sentence = "I bought a lap top yesterday.";
        let expected = "I bought a laptop yesterday.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn web_cam() {
        let test_sentence = "The web cam captured a stunning image.";
        let expected = "The web cam captured a stunning image."; // see note below
                                                                 // In many contexts, "web cam" is acceptable if meant as a phrase.
                                                                 // However, if your dictionary marks "webcam" as the noun, then:
        let expected = "The webcam captured a stunning image.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn note_book() {
        let test_sentence = "She always carries a note book to jot down ideas.";
        let expected = "She always carries a notebook to jot down ideas.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn mother_board() {
        let test_sentence = "After the upgrade, the mother board was replaced.";
        let expected = "After the upgrade, the motherboard was replaced.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn smart_phone() {
        let test_sentence = "He bought a new smart phone last week.";
        let expected = "He bought a new smartphone last week.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn desk_top() {
        let test_sentence = "The company provided each employee with a desk top computer.";
        let expected = "The company provided each employee with a desktop computer.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn firm_ware() {
        let test_sentence = "The device's firm ware was updated overnight.";
        let expected = "The device's firmware was updated overnight.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn back_plane() {
        let test_sentence = "A reliable back plane is essential for high-speed data transfer.";
        let expected = "A reliable backplane is essential for high-speed data transfer.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn spread_sheet() {
        let test_sentence = "The accountant reviewed the spread sheet carefully.";
        let expected = "The accountant reviewed the spreadsheet carefully.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn side_bar() {
        let test_sentence = "The website's side bar offers quick navigation links.";
        let expected = "The website's sidebar offers quick navigation links.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn back_pack() {
        let test_sentence = "I packed my books in my back pack before leaving.";
        let expected = "I packed my books in my backpack before leaving.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn cup_board() {
        let test_sentence = "She stored the dishes in the old cup board.";
        let expected = "She stored the dishes in the old cupboard.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn key_board() {
        let test_sentence = "My key board stopped working during the meeting.";
        let expected = "My keyboard stopped working during the meeting.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn touch_screen() {
        let test_sentence = "The device features a responsive touch screen.";
        let expected = "The device features a responsive touchscreen.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn head_set() {
        let test_sentence = "He bought a new head set for his workouts.";
        let expected = "He bought a new headset for his workouts.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn frame_work() {
        let test_sentence = "The frame work of the app was built with care.";
        let expected = "The framework of the app was built with care.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn touch_pad() {
        let test_sentence = "The touch pad on my laptop is very sensitive.";
        let expected = "The touchpad on my laptop is very sensitive.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn micro_processor() {
        let test_sentence = "This micro processor is among the fastest available.";
        let expected = "This microprocessor is among the fastest available.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn head_phone() {
        let test_sentence = "I lost my head phone at the gym.";
        let expected = "I lost my headphone at the gym.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn micro_services() {
        let test_sentence = "Our architecture now relies on micro services.";
        let expected = "Our architecture now relies on microservices.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn dash_board() {
        let test_sentence = "The dash board shows real-time analytics.";
        let expected = "The dashboard shows real-time analytics.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn site_map() {
        let test_sentence = "A site map is provided at the footer of the website.";
        let expected = "A sitemap is provided at the footer of the website.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn fire_wall() {
        let test_sentence = "A robust fire wall is essential for network security.";
        let expected = "A robust firewall is essential for network security.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn bit_stream() {
        let test_sentence = "The bit stream was interrupted during transmission.";
        let expected = "The bitstream was interrupted during transmission.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn block_chain() {
        let test_sentence = "The block chain is revolutionizing the financial sector.";
        let expected = "The blockchain is revolutionizing the financial sector.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn thumb_nail() {
        let test_sentence = "I saved the image as a thumb nail.";
        let expected = "I saved the image as a thumbnail.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn bath_room() {
        let test_sentence = "They remodeled the bath room entirely.";
        let expected = "They remodeled the bathroom entirely.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn for_ever() {
        let test_sentence = "Promise me you'll love me for ever.";
        let expected = "Promise me you'll love me forever.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn over_night() {
        let test_sentence = "They set off on their journey over night.";
        let expected = "They set off on their journey overnight.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn every_one() {
        let test_sentence = "Every one should have access to quality education.";
        let expected = "Everyone should have access to quality education.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn hand_held() {
        let test_sentence = "The camera has a comfortable hand held design.";
        let expected = "The camera has a comfortable handheld design.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn play_ground() {
        let test_sentence = "The kids spent the afternoon at the play ground.";
        let expected = "The kids spent the afternoon at the playground.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn run_way() {
        let test_sentence = "The airplane taxied along the run way.";
        let expected = "The airplane taxied along the runway.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn cyber_space() {
        let test_sentence = "Hackers roam the cyber space freely.";
        let expected = "Hackers roam the cyberspace freely.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn cyber_attack() {
        let test_sentence = "The network was hit by a cyber attack.";
        let expected = "The network was hit by a cyberattack.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn web_socket() {
        let test_sentence = "Real-time updates are sent via a web socket.";
        let expected = "Real-time updates are sent via a WebSocket.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn finger_print() {
        let test_sentence = "The detective collected a finger print as evidence.";
        let expected = "The detective collected a fingerprint as evidence.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }
}
