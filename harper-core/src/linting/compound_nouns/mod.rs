mod compound_noun_after_det_adj;
mod compound_noun_after_possessive;
mod compound_noun_before_aux_verb;

use super::{Lint, LintKind, Suggestion, merge_linters::merge_linters};
use crate::{CharStringExt, DictWordMetadata, Token};

// Helper function to check if a token is a content word (not a function word)
pub(crate) fn is_content_word(tok: &Token, src: &[char]) -> bool {
    let Some(Some(meta)) = tok.kind.as_word() else {
        return false;
    };

    tok.span.len() > 1
        && (meta.is_noun() || meta.is_adjective() || meta.is_verb() || meta.is_adverb())
        && !(meta.is_determiner() || meta.is_conjunction())
        && (!meta.preposition || tok.span.get_content(src).eq_ignore_ascii_case_str("bar"))
}

pub(crate) fn predicate(
    closed: Option<&DictWordMetadata>,
    open: Option<&DictWordMetadata>,
) -> bool {
    open.is_none() && closed.is_some_and(|m| m.is_noun() && !m.is_proper_noun())
}

use compound_noun_after_det_adj::CompoundNounAfterDetAdj;
use compound_noun_after_possessive::CompoundNounAfterPossessive;
use compound_noun_before_aux_verb::CompoundNounBeforeAuxVerb;

merge_linters!(CompoundNouns => CompoundNounAfterDetAdj, CompoundNounBeforeAuxVerb, CompoundNounAfterPossessive => "Detects compound nouns split by a space and suggests merging them when both parts form a valid noun." );

#[cfg(test)]
mod tests {
    use super::CompoundNouns;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn web_cam() {
        let test_sentence = "The web cam captured a stunning image.";
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
    #[ignore = "\"everyone\" is not a valid compound noun, it's a pronoun"]
    fn every_one() {
        let test_sentence = "Every one should have access to quality education.";
        let expected = "Everyone should have access to quality education.";
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
        let expected = "Real-time updates are sent via a websocket.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn finger_print() {
        let test_sentence = "The detective collected a finger print as evidence.";
        let expected = "The detective collected a fingerprint as evidence.";
        assert_suggestion_result(test_sentence, CompoundNouns::default(), expected);
    }

    #[test]
    fn got_is_not_possessive() {
        assert_lint_count("I got here by car...", CompoundNouns::default(), 0);
    }

    #[test]
    fn allow_issue_662() {
        assert_lint_count(
            "They are as old as *modern* computers ",
            CompoundNouns::default(),
            0,
        );
    }

    #[test]
    fn allow_issue_661() {
        assert_lint_count("I may be wrong.", CompoundNouns::default(), 0);
    }

    #[test]
    fn allow_issue_704() {
        assert_lint_count(
            "Here are some ways to do that:",
            CompoundNouns::default(),
            0,
        );
    }

    #[test]
    fn allows_issue_721() {
        assert_lint_count(
            "So if you adjust any one of these adjusters that can have a negative or a positive effect.",
            CompoundNouns::default(),
            0,
        );
    }

    #[test]
    fn allows_678() {
        assert_lint_count(
            "they can't catch all the bugs.",
            CompoundNouns::default(),
            0,
        );
    }

    #[test]
    fn ina_not_suggested() {
        assert_lint_count(
            "past mistakes or a character in a looping reality facing personal challenges.",
            CompoundNouns::default(),
            0,
        );
    }

    #[test]
    fn allow_suppress_or() {
        assert_lint_count(
            "He must decide whether to suppress or coexist with his doppelgänger.",
            CompoundNouns::default(),
            0,
        );
    }

    #[test]
    fn allow_an_arm_and_a_leg() {
        assert_lint_count(
            "I have to pay an arm and a leg get a worker to come and be my assistant baker.",
            CompoundNouns::default(),
            0,
        );
    }

    #[test]
    fn allow_well_and_723() {
        assert_lint_count(
            "I understood very well and decided to go.",
            CompoundNouns::default(),
            0,
        );
    }

    #[test]
    fn allow_can_not() {
        assert_lint_count("Size can not be determined.", CompoundNouns::default(), 0);
    }

    #[test]
    fn dont_flag_lot_to() {
        assert_lint_count(
            "but you'd have to raise taxes a lot to do it.",
            CompoundNouns::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_to_me() {
        assert_lint_count(
            "There's no massive damage to the rockers or anything that to me would indicate that like the whole front of the car was off",
            CompoundNouns::default(),
            0,
        );
    }

    #[test]
    fn allow_issue_1553() {
        assert_no_lints(
            "I'm not sure if there's anyone else that may be interested in more fine-grained control, but as it stands, having the domain level toggle is sufficient for me.",
            CompoundNouns::default(),
        );
    }

    #[test]
    fn allow_issue_1496() {
        assert_no_lints(
            "I am not able to respond to messages.",
            CompoundNouns::default(),
        );
    }

    #[test]
    fn allow_issue_1298() {
        assert_no_lints(
            "A series of tests that cover all possible cases.",
            CompoundNouns::default(),
        );
    }

    #[test]
    fn dont_flag_project_or() {
        assert_no_lints(
            "You can star or watch this project or follow author to get release notifications in time.",
            CompoundNouns::default(),
        );
    }
}
