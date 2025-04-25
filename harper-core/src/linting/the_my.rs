use super::{Lint, LintKind, PatternLinter, Suggestion};
use crate::{
    CharStringExt, Token, TokenStringExt,
    patterns::{AnyCapitalization, EitherPattern, Pattern, SequencePattern, WordSet},
};

pub struct TheMy {
    pattern: Box<dyn Pattern>,
}

impl Default for TheMy {
    fn default() -> Self {
        let the = AnyCapitalization::of("the");
        let any_possessive = WordSet::new(&["my", "your", "his", "her", "its", "our", "their"]);

        let the_poss = SequencePattern::default()
            .then(the.clone())
            .then_whitespace()
            .then(any_possessive.clone());

        let poss_the = SequencePattern::default()
            .then(any_possessive)
            .then_whitespace()
            .then(the);

        Self {
            pattern: Box::new(EitherPattern::new(vec![
                Box::new(the_poss),
                Box::new(poss_the),
            ])),
        }
    }
}

impl PatternLinter for TheMy {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span().unwrap();
        let span_content = span.get_content(source);

        let first_word = matched_tokens[0].span.get_content(source);
        let second_word = matched_tokens[2].span.get_content(source);

        let first_word_string: String = first_word.to_string();

        let possessive = if first_word_string.eq_ignore_ascii_case("the") {
            second_word
        } else {
            first_word
        };

        // Don't flag "the My" or "my The" since they could be titles of things
        if second_word[0].is_uppercase() {
            return None;
        }

        // Don't flag "her the" since her is also the object case pronoun: "give her the book"
        if first_word_string.eq_ignore_ascii_case("her") {
            return None;
        }

        let suggestions = vec![
            Suggestion::replace_with_match_case(possessive.to_vec(), span_content),
            Suggestion::replace_with_match_case("the".chars().collect(), span_content),
        ];

        Some(Lint {
            span,
            lint_kind: LintKind::Repetition,
            suggestions,
            message: "Use either the definite article 'the' or the possessive. Using both together is ungrammatical in English.".to_owned(),
            priority: 127,
        })
    }

    fn description(&self) -> &'static str {
        "Flags the definite article used together with a possessive."
    }
}

#[cfg(test)]
mod tests {
    use super::TheMy;
    use crate::linting::tests::{
        assert_lint_count, assert_nth_suggestion_result, assert_suggestion_result,
    };

    #[test]
    fn correct_the_my_atomic_lowercase() {
        assert_suggestion_result("the my", TheMy::default(), "my");
    }

    #[test]
    fn correct_the_my_atomic_2nd_suggestion() {
        assert_nth_suggestion_result("the my", TheMy::default(), "the", 1);
    }

    #[test]
    fn correct_the_my_atomic_uppercase() {
        assert_suggestion_result("The my", TheMy::default(), "My");
    }

    #[test]
    fn correct_my_the_atomic_lowercase() {
        assert_suggestion_result("my the", TheMy::default(), "my");
    }

    #[test]
    fn correct_my_the_atomic_2nd_suggestion() {
        assert_nth_suggestion_result("my the", TheMy::default(), "the", 1);
    }

    #[test]
    fn correct_my_the_atomic_uppercase() {
        assert_suggestion_result("My the", TheMy::default(), "My");
    }

    #[test]
    fn dont_correct_capitalized_possessive() {
        assert_lint_count("For some time the My Projects personal page was \"sluggish\" or took some time to generate the miniature depicting the project, now it seems completely stuck ...
", TheMy::default(), 0);
    }

    #[test]
    fn correct_the_my_github() {
        assert_suggestion_result(
            "When I try to configure the my react-native app to support koltin file, this library gives these errors",
            TheMy::default(),
            "When I try to configure my react-native app to support koltin file, this library gives these errors",
        );
    }

    #[test]
    fn correct_the_our_github() {
        assert_suggestion_result(
            "Source codes of the our paper titled \"Multi-level Textual-Visual Alignment and Fusion Network for Multimodal Aspect-based Sentiment Analysis\"",
            TheMy::default(),
            "Source codes of our paper titled \"Multi-level Textual-Visual Alignment and Fusion Network for Multimodal Aspect-based Sentiment Analysis\"",
        );
    }

    #[test]
    fn correct_the_their_github() {
        assert_suggestion_result(
            "the slider cannot render when i use again the their component on NextJS app",
            TheMy::default(),
            "the slider cannot render when i use again their component on NextJS app",
        );
    }

    #[test]
    fn correct_your_the_github() {
        assert_suggestion_result(
            "This plugin allows you to view your the information about order and customer from your spree store on zendesk",
            TheMy::default(),
            "This plugin allows you to view your information about order and customer from your spree store on zendesk",
        );
    }

    #[test]
    fn correct_my_the_github() {
        assert_suggestion_result(
            "Scripts used my the project to collect, process and store social media data from a number of sources",
            TheMy::default(),
            "Scripts used my project to collect, process and store social media data from a number of sources",
        );
    }

    #[test]
    fn dont_correct_the_your_github() {
        assert_lint_count(
            "What exactly is the sort order of list names on the Your Stars page?",
            TheMy::default(),
            0,
        );
    }

    #[test]
    fn dont_correct_my_the_github() {
        assert_lint_count(
            "My The Frame TV is not pulling information properly",
            TheMy::default(),
            0,
        )
    }

    #[test]
    fn correct_our_the_github() {
        assert_suggestion_result(
            "Companion Repository to our the whitepaper \"Towards Reliable and Scalable Linux Kernel CVE Attribution in Automated Static Firmware Analyses\"",
            TheMy::default(),
            "Companion Repository to our whitepaper \"Towards Reliable and Scalable Linux Kernel CVE Attribution in Automated Static Firmware Analyses\"",
        )
    }

    #[test]
    fn correct_their_the_github() {
        assert_suggestion_result(
            "Types exported by @_exported remember only their the original module",
            TheMy::default(),
            "Types exported by @_exported remember only their original module",
        )
    }

    #[test]
    fn dont_correct_her_the_github() {
        assert_lint_count(
            "Create an admin role for boba-tan and give her the GoreMaster role only in !gore",
            TheMy::default(),
            0,
        )
    }

    #[test]
    fn correct_the_his_github() {
        assert_suggestion_result(
            "Allows the user to specify the his last name.",
            TheMy::default(),
            "Allows the user to specify his last name.",
        )
    }

    #[test]
    fn correct_his_the_github() {
        assert_suggestion_result(
            "One interesting creation was his the Schelling segregation model",
            TheMy::default(),
            "One interesting creation was his Schelling segregation model",
        )
    }

    #[test]
    fn correct_the_her_github() {
        assert_suggestion_result(
            "In memory of the occasion when our Queen Victoria graciously came to see our Island, and the her Royal Consort Albert landed at Ramsey",
            TheMy::default(),
            "In memory of the occasion when our Queen Victoria graciously came to see our Island, and her Royal Consort Albert landed at Ramsey",
        )
    }
}
