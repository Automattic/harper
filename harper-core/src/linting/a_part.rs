use crate::expr::Expr;
use crate::expr::FirstMatchOf;
use crate::expr::FixedPhrase;
use crate::{
    Token, TokenStringExt,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct APart {
    expr: Box<dyn Expr>,
}

impl Default for APart {
    fn default() -> Self {
        let pattern = FirstMatchOf::new(vec![
            Box::new(FixedPhrase::from_phrase("a part from")),
            Box::new(FixedPhrase::from_phrase("apart of")),
            Box::new(FixedPhrase::from_phrase("fall a part")),
            Box::new(FixedPhrase::from_phrase("far a part")),
        ]);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for APart {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let text: String = span.get_content(source).iter().collect();
        let text_lower = text.to_lowercase();

        let (suggestions, message) = match text_lower.as_str() {
            // Not always a mistake:
            // I ordered a part from an online store
            "a part from" => (
                vec![
                    Suggestion::ReplaceWith("apart from".chars().collect()),
                    Suggestion::ReplaceWith("a part of".chars().collect()),
                ],
                "If you mean 'except for', use 'apart from'. If you mean 'a piece belonging to', use 'a part of'. Keep it this way if referring to the origin of a piece.",
            ),
            "apart of" => (
                vec![
                    Suggestion::ReplaceWith("a part of".chars().collect()),
                    Suggestion::ReplaceWith("apart from".chars().collect()),
                ],
                "Did you mean 'a part of' (a piece belonging to) or 'apart from' (except for)?",
            ),
            // Not necessarily always a mistake:
            // How would you detect how far a part is from another part?
            // Any one else amazed with how far a part will travel if you accidentally drop a little model piece?
            // ... how far a part of the value of the relevant step is attributable to the overseas part of the tax ...
            //
            // If the previous word before "far" is "how" or "so", it's still ambiguous
            // But could the next word after "part" help us understand if it's a mistake?
            "far a part" => (
                vec![Suggestion::ReplaceWith("far apart".chars().collect())],
                "If you mean 'separated by a distance', use 'far apart'. If referring to the distance of a piece, keep it this way.",
            ),
            "fall a part" => (
                vec![Suggestion::ReplaceWith("fall apart".chars().collect())],
                "'Fall apart' meaning 'collapse into pieces' or 'stop functioning' is written as two words.",
            ),
            _ => return None,
        };

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message: message.to_owned(),
            priority: 50,
        })
    }

    fn description(&self) -> &'static str {
        "Finds and corrects common mistakes between 'a part' and 'apart'"
    }
}

#[cfg(test)]
mod tests {
    use super::APart;
    use crate::linting::tests::{assert_lint_count, assert_top3_suggestion_result};

    #[test]
    fn allow_normal_use_of_a_part() {
        assert_lint_count(
            "That's not the whole truth, it's just a part.",
            APart::default(),
            0,
        );
    }

    #[test]
    fn allow_normal_use_of_apart() {
        assert_lint_count("You shouldn't have taken it apart.", APart::default(), 0);
    }

    #[test]
    fn allow_normal_use_of_a_part_of() {
        assert_lint_count("The elbow is a part of the arm.", APart::default(), 0);
    }

    #[test]
    fn allow_normal_use_of_apart_from() {
        assert_lint_count("Apart from one error, the code works.", APart::default(), 0);
    }

    #[test]
    fn allow_normal_us_of_fall_apart() {
        assert_lint_count("The roof fell apart.", APart::default(), 0);
    }

    #[test]
    fn allow_normal_use_of_far_apart() {
        assert_lint_count("Okinawa and Hokkaido are far apart.", APart::default(), 0);
    }

    #[test]
    fn corrects_a_part_from_to_apart_from_format() {
        assert_top3_suggestion_result(
            "Is it correct that the output file seems to be the same of the input file a part from the format (input: jpg, output: png)?",
            APart::default(),
            "Is it correct that the output file seems to be the same of the input file apart from the format (input: jpg, output: png)?",
        );
    }

    #[test]
    fn corrects_a_part_from_to_apart_from_english() {
        assert_top3_suggestion_result(
            "Do you know there are more languages out there a part from English right?",
            APart::default(),
            "Do you know there are more languages out there apart from English right?",
        )
    }

    #[test]
    fn corrects_a_part_from_to_a_part_of() {
        assert_top3_suggestion_result(
            "An easy tool to generate backdoor with msfvenom (a part from metasploit framework).",
            APart::default(),
            "An easy tool to generate backdoor with msfvenom (a part of metasploit framework).",
        )
    }

    #[test]
    fn corrects_apart_of_to_apart_from_cflinuxfs() {
        assert_top3_suggestion_result(
            "Doesn't work with any stacks apart of cflinuxfs2 and cflinuxfs3",
            APart::default(),
            "Doesn't work with any stacks apart from cflinuxfs2 and cflinuxfs3",
        )
    }

    #[test]
    fn corrects_apart_of_to_apart_from_using() {
        assert_top3_suggestion_result(
            "apart of using filter, i can't find it in the documentation",
            APart::default(),
            "apart from using filter, i can't find it in the documentation",
        )
    }

    #[test]
    fn corrects_apart_of_to_a_part_of_openai() {
        assert_top3_suggestion_result(
            "export 'Usage' class as apart of openai.types",
            APart::default(),
            "export 'Usage' class as a part of openai.types",
        )
    }

    #[test]
    fn corrects_apart_of_to_a_part_of_formly() {
        assert_top3_suggestion_result(
            "FormlyDatepickerTypeComponent is not listed as apart of the Formly Public API",
            APart::default(),
            "FormlyDatepickerTypeComponent is not listed as a part of the Formly Public API",
        )
    }

    #[test]
    fn corrects_far_a_part() {
        assert_top3_suggestion_result(
            "That leaves you only the other hand on the keyboard and you don't want the keys to be that far a part.",
            APart::default(),
            "That leaves you only the other hand on the keyboard and you don't want the keys to be that far apart.",
        )
    }

    #[test]
    fn corrects_so_far_a_part_from_being_taken() {
        assert_top3_suggestion_result(
            "I can't see in the code what is done really with this session_timeout so far a part from being taken from the conf if defined there or setup ...",
            APart::default(),
            "I can't see in the code what is done really with this session_timeout so far apart from being taken from the conf if defined there or setup ...",
        )
    }

    #[test]
    fn corrects_so_far_a_part_from_version_upgrade() {
        assert_top3_suggestion_result(
            "Any workaround so far a part from the version upgrade?",
            APart::default(),
            "Any workaround so far apart from the version upgrade?",
        )
    }

    #[test]
    fn corrects_fall_a_part() {
        assert_top3_suggestion_result(
            "When I set up a script I set up card priority based on my frontline but sometimes a servant dies which sometimes causes things to fall a part.",
            APart::default(),
            "When I set up a script I set up card priority based on my frontline but sometimes a servant dies which sometimes causes things to fall apart.",
        )
    }

    // Sentences from GitHub I can't understand so can't suggest a fix

    // Use Reanimated to create Animated Map Components and provide as **a part from** library.
    // I'm trying to add tokens to the bucket **apart of** the time so this can help to have distributed rate-limiting so that
    // Slice **apart of** slice apart breaks after slightest change to sliced model
    // Docker image running as different user **apart of** multiple groups
    // Diamond Checker is a cookie checker **apart of** the DIamond Software
    // fetchParent() doesn't work properly with foreign key referencing just a part from a composed primary key
}
