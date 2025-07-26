use crate::{
    CharStringExt, Token, TokenStringExt,
    expr::{All, Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct PronounAfterPreposition {
    expr: Box<dyn Expr>,
}

impl Default for PronounAfterPreposition {
    fn default() -> Self {
        // TODO: Finish researching which other POSes each of these prepositions also are

        // after: wikt: adv, prep, conj, (adj), (noun) - dict: prep
        // because: conj, adv, intj, prep - conj, prep
        // before: wikt: prep, adv, conj, (noun) - dict: prep, conj
        // bet: wikt: noun, verb, (prep) - dict: noun, verb, (prep)
        // but: wikt: prep, adv, conj, (noun), verb - dict: prep, conj, noun
        // down: wikt: adv, prep, adj, verb, noun - dict: prep, adj, verb, noun
        // for: wikt: conj, prep, (part) - dict: conj, prep
        // gone: wikt: verb, adj, prep - dict: verb, adj, prep
        // since: wikt: adv, prep, conj -
        // than:
        // till:
        // until:
        // up:
        // while:

        // This predicat passes all tests, but is it ideal?
        let blacklist_predicate = WordSet::new(&[
            "after", "because", "before", "bet", "but", "down", "for", "gone", "since", "than",
            "till", "until", "up", "while",
        ]);

        // These other predicates fail the same tests as each other but maybe it's a better approach?
        // Failing tests with the other two predicates:
        //   `fix_she_after_preposition`, `fix_they_after_preposition_all_caps`

        // let other_pos_predicate = SequenceExpr::default()
        //     .then(|tok: &Token, _src: &[char]| {
        //         let k = &tok.kind;
        //         k.is_adverb() || k.is_conjunction() || k.is_adjective() || k.is_noun()
        //     });

        // let homograph_predicate = SequenceExpr::default()
        //     .then(|tok: &Token, _src: &[char]| tok.kind.is_likely_homograph());

        let predicate = blacklist_predicate;

        let expr =
            SequenceExpr::default()
                .then_preposition()
                .t_ws()
                .then(|tok: &Token, _src: &[char]| {
                    tok.kind.is_subject_pronoun() && !tok.kind.is_object_pronoun()
                });

        let expr = All::new(vec![
            Box::new(expr),
            Box::new(SequenceExpr::default().then_unless(predicate)),
        ]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for PronounAfterPreposition {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let subject_span = toks.last()?.span;
        let subject_chars = subject_span.get_content(src);

        eprintln!("🍓 '{}'", toks.span()?.get_content_string(src));

        let object: &[char] = match subject_chars.to_lower().as_ref() {
            ['i'] => &['m', 'e'],
            ['w', 'e'] => &['u', 's'],
            ['h', 'e'] => &['h', 'i', 'm'],
            ['s', 'h', 'e'] => &['h', 'e', 'r'],
            ['t', 'h', 'e', 'y'] => &['t', 'h', 'e', 'm'],
            // _ => return None,
            _ => {
                eprintln!(
                    "🍓🍓 Not a pronoun: '{}'",
                    subject_chars.iter().collect::<String>()
                );
                return None;
            }
        };

        Some(Lint {
            span: subject_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                object.to_vec(),
                subject_chars,
            )],
            message: "Use the correct pronoun form after prepositions.".to_string(),
            priority: 126,
        })
    }

    fn description(&self) -> &'static str {
        "Checks that the correct pronoun forms are used after prepositions."
    }
}

#[cfg(test)]
mod tests {
    use super::PronounAfterPreposition;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fix_she_after_preposition() {
        assert_suggestion_result(
            "I talked to the saleswoman and listened at she carefully.",
            PronounAfterPreposition::default(),
            "I talked to the saleswoman and listened at her carefully.",
        );
    }

    #[test]
    #[ignore = "replace_with_match_case doesn't handle 'I' well"]
    fn fix_i_after_preposition() {
        assert_suggestion_result(
            "After my two weeks of I came back and the images ...",
            PronounAfterPreposition::default(),
            "After my two weeks of me came back and the images ...",
        );
    }

    // Case-matching for normal pronouns

    #[test]
    fn fix_he_after_preposition_all_lower() {
        assert_suggestion_result("from he", PronounAfterPreposition::default(), "from him");
    }

    #[test]
    fn fix_she_after_preposition_all_lower() {
        assert_suggestion_result("To She", PronounAfterPreposition::default(), "To Her");
    }

    #[test]
    fn fix_they_after_preposition_all_caps() {
        assert_suggestion_result("VIA THEY", PronounAfterPreposition::default(), "VIA THEM");
    }

    // Case-matching for I

    #[test]
    #[ignore = "replace_with_match_case doesn't handle 'I' well"]
    fn fix_i_after_preposition_title_case() {
        assert_suggestion_result(
            "All About I",
            PronounAfterPreposition::default(),
            "All About Me",
        );
    }

    #[test]
    #[ignore = "'for' can also be a conjunction and subject pronouns can follow conjunctions"]
    fn fix_i_after_preposition_all_caps() {
        assert_suggestion_result(
            "IF NOT FOR I",
            PronounAfterPreposition::default(),
            "IF NOT FOR ME",
        );
    }

    // False positives

    #[test]
    fn dont_flag_before() {
        assert_lint_count(
            "Alice had not a moment to think about stopping herself before she found herself falling down a very deep well.",
            PronounAfterPreposition::default(),
            0,
        );
    }

    // Interesting cases from the snapshots

    #[test]
    #[ignore = "'in' and 'she' are in different clauses"]
    fn dont_flag_and_in_she_went() {
        assert_lint_count("And in she went.", PronounAfterPreposition::default(), 0);
    }

    #[test]
    #[ignore = "'on' and 'I' are in different clauses"]
    fn dont_flag_and_as_i_walked_on_i_was_lonely_no_longer() {
        assert_lint_count(
            "And as I walked on I was lonely no longer.",
            PronounAfterPreposition::default(),
            0,
        );
    }

    #[test]
    #[ignore = "'in' and 'she' are in different clauses"]
    fn dont_flag_when_we_came_in_she_held_us_silent_for_a_moment_with_a_lifted_hand() {
        assert_lint_count(
            "When we came in she held us silent for a moment with a lifted hand.",
            PronounAfterPreposition::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_i_bet_he_killed_a_man() {
        assert_lint_count(
            "I’ll bet he killed a man.",
            PronounAfterPreposition::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_when_he_was_gone_i_turned_immediately_to_jordan() {
        assert_lint_count(
            "When he was gone I turned immediately to Jordan.",
            PronounAfterPreposition::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_gatsby_and_pulled_his_face() {
        assert_lint_count(
            "As he left the room again she got up and went over to Gatsby and pulled his face.",
            PronounAfterPreposition::default(),
            0,
        );
    }
}
