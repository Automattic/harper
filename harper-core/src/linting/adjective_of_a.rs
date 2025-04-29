use super::{Lint, LintKind, Linter, Suggestion};
use crate::{Document, Span, TokenStringExt};

/// Detect sequences of words of the form "adjective of a".
#[derive(Debug, Clone, Copy, Default)]
pub struct AdjectiveOfA;

const FALSE_POSITIVES: &[&str] = &[
    // Different valid constructions.
    "all",
    "emblematic",
    "equivalent",
    "full",
    "fun",
    "illustrative",
    "inside",
    "more",
    "much",
    "off",
    "out",
    "shy",
    "up",
    // The word is used more as a noun in this context.
    // (using .kind.is_likely_homograph() here is too strict)
    "back",
    "bit",
    "borderline",
    "bottom",
    "chance",
    "clockwork",
    "derivative",
    "dream",
    "eighth",
    "front",
    "half",
    "head",
    "kind",
    "left",
    "light",
    "meaning",
    "middle",
    "multiple",
    "one",
    "part",
    "potential",
    "precision",
    "red",
    // for "rid" I removed the `5` flag in `dictionary.dict``
    "shadow",
    "side",
    "short",
    "slack",
    "something",
    "sound",
    "top",
];

fn is_false_positive(chars: &[char]) -> bool {
    let word = chars.iter().collect::<String>();
    FALSE_POSITIVES
        .iter()
        .any(|&false_pos| word.eq_ignore_ascii_case(false_pos))
}

impl Linter for AdjectiveOfA {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for i in document.iter_adjective_indices() {
            let adjective = document.get_token(i).unwrap();
            let space_1 = document.get_token(i + 1);
            let word_of = document.get_token(i + 2);
            let space_2 = document.get_token(i + 3);
            let a_or_an = document.get_token(i + 4);
            let adj_chars: &[char] = document.get_span_content(&adjective.span);

            // Rule out false positives
            if is_false_positive(adj_chars) {
                continue;
            }

            // Rule out comparatives and superlatives.

            // Pros:
            // "for the better of a day"
            // "might not be the best of a given run"
            // "Which brings me to my best of a bad situation."
            //
            // Cons:
            // "see if you can give us a little better of an answer"
            // "hopefully it won't be too much worse of a problem"
            // "seems far worse of a result to me"
            if adj_chars.ends_with(&['e', 'r']) || adj_chars.ends_with(&['s', 't']) {
                continue;
            }
            // Rule out present participles (e.g. "beginning of a")
            // The -ing form of a verb acts as an adjective called a present participle
            // and also acts as a noun called a gerund.
            if adj_chars.ends_with(&['i', 'n', 'g'])
                && (adjective.kind.is_noun() && adjective.kind.is_verb())
            {
                continue;
            }

            if space_1.is_none() || word_of.is_none() || space_2.is_none() || a_or_an.is_none() {
                continue;
            }
            let space_1 = space_1.unwrap();
            if !space_1.kind.is_whitespace() {
                continue;
            }
            let word_of = word_of.unwrap();
            if !word_of.kind.is_word() {
                continue;
            }
            let word_of = document.get_span_content(&word_of.span);
            if word_of != ['o', 'f'] {
                continue;
            }
            let space_2 = space_2.unwrap();
            if !space_2.kind.is_whitespace() {
                continue;
            }
            let a_or_an = a_or_an.unwrap();
            if !a_or_an.kind.is_word() {
                continue;
            }
            let a_or_an_chars = document.get_span_content(&a_or_an.span);
            if a_or_an_chars != ['a'] && a_or_an_chars != ['a', 'n'] {
                continue;
            }

            // Whitespace may differ, add the other replacement if so
            let mut sugg_1 = Vec::new();
            sugg_1.extend_from_slice(document.get_span_content(&adjective.span));
            sugg_1.extend_from_slice(document.get_span_content(&space_1.span));
            sugg_1.extend_from_slice(document.get_span_content(&a_or_an.span));

            let mut sugg_2 = Vec::new();
            sugg_2.extend_from_slice(document.get_span_content(&adjective.span));
            sugg_2.extend_from_slice(document.get_span_content(&space_2.span));
            sugg_2.extend_from_slice(document.get_span_content(&a_or_an.span));

            let mut suggestions = vec![Suggestion::ReplaceWith(sugg_1.clone())];
            if sugg_1 != sugg_2 {
                suggestions.push(Suggestion::ReplaceWith(sugg_2));
            }

            lints.push(Lint {
                span: Span::new(adjective.span.start, a_or_an.span.end),
                lint_kind: LintKind::Style,
                suggestions,
                message: "The word `of` is not needed here.".to_string(),
                priority: 63,
            });
        }

        lints
    }

    fn description(&self) -> &str {
        "This rule looks for sequences of words of the form `adjective of a`."
    }
}

#[cfg(test)]
mod tests {
    use super::AdjectiveOfA;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn correct_large_of_a() {
        assert_suggestion_result(
            "Yeah I'm using as large of a batch size as I can on this machine",
            AdjectiveOfA,
            "Yeah I'm using as large a batch size as I can on this machine",
        )
    }

    #[test]
    fn correct_bad_of_an() {
        assert_suggestion_result(
            "- If forking is really that bad of an option, let's first decide where to put this.",
            AdjectiveOfA,
            "- If forking is really that bad an option, let's first decide where to put this.",
        );
    }

    #[test]
    fn dont_flag_comparative() {
        assert_lint_count(
            "I only worked with custom composer installers for the better of a day, so please excuse me if I missed a thing.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_superlative() {
        assert_lint_count(
            "I am trying to use composites to visualize the worst of a set of metrics.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_kind() {
        // Adjective as in "a kind person" vs noun as in "A kind of person"
        assert_lint_count(
            "Log.txt file automatic creation in PWD is kind of an anti-feature",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_part() {
        // Can be an adjective in e.g. "He is just part owner"
        assert_lint_count(
            "cannot delete a food that is no longer part of a recipe",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_much() {
        // "much of" is correct idiomatic usage
        assert_lint_count(
            "How much of a performance impact when switching from rails to rails-api ?",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_part_uppercase() {
        // Can be an adjective in e.g. "Part man, part machine"
        assert_lint_count(
            "Quarkus Extension as Part of a Project inside a Monorepo?",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_all_of() {
        // "all of" is correct idiomatic usage
        assert_lint_count(
            "This repository is deprecated. All of its content and history has been moved.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_inside() {
        // "inside of" is idiomatic usage
        assert_lint_count(
            "Michael and Brock sat inside of a diner in Brandon",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_out() {
        // "out of" is correct idiomatic usage
        assert_lint_count(
            "not only would he potentially be out of a job and back to sort of poverty",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_full() {
        // "full of" is correct idiomatic usage
        assert_lint_count(
            "fortunately I happen to have this Tupperware full of an unceremoniously disassembled LED Mac Mini",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_something() {
        // Can be a noun in e.g. "a certain something"
        assert_lint_count(
            "Well its popularity seems to be taking something of a dip right now.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_short() {
        // Can be a noun in e.g. "use a multimeter to find the short"
        assert_lint_count(
            "I found one Youtube short of an indonesian girl.",
            AdjectiveOfA,
            0,
        )
    }

    #[test]
    fn dont_flag_bottom() {
        // Can be an adjective in e.g. "bottom bunk"
        assert_lint_count(
            "When leaves are just like coming out individually from the bottom of a fruit.",
            AdjectiveOfA,
            0,
        )
    }

    #[test]
    fn dont_flag_left() {
        // Can be an adjective in e.g. "left hand"
        assert_lint_count("and what is left of a 12vt coil", AdjectiveOfA, 0)
    }

    #[test]
    fn dont_flag_full_uppercase() {
        assert_lint_count("Full of a bunch varnish like we get.", AdjectiveOfA, 0);
    }

    #[test]
    fn dont_flag_head() {
        // Can be an adjective in e.g. "the head cook"
        assert_lint_count(
            "You need to get out if you're the head of an education department and you're not using AI",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_middle() {
        // Can be an adjective in e.g. "middle child"
        assert_lint_count(
            "just to get to that part in the middle of a blizzard",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_chance() {
        // Can be an adjective in e.g. "a chance encounter"
        assert_lint_count(
            "products that you overpay for because there are subtle details in the terms and conditions that reduce the size or chance of a payout.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_potential() {
        // Can be an adjective in e.g. "a potential candidate"
        assert_lint_count(
            "People that are happy to accept it for the potential of a reward.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_sound() {
        // Can be an adjective in e.g. "sound advice"
        assert_lint_count("the sound of an approaching Krampus", AdjectiveOfA, 0);
    }

    #[test]
    fn dont_flag_rid() {
        // I removed the `5` flag from `rid` in `dictionary.dict`
        // because dictionaries say the sense is archaic.
        assert_lint_count("I need to get rid of a problem", AdjectiveOfA, 0);
    }

    #[test]
    fn dont_flag_precision() {
        // Can be an adjective in e.g. "a precision instrument"
        assert_lint_count(
            "a man whose crew cut has the precision of a targeted drone strike",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_back() {
        // Can be an adjective in e.g. "back door"
        assert_lint_count(
            "a man whose crew cut has the back of a targeted drone strike",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_emblematic() {
        // "emblematic of" is correct idiomatic usage
        assert_lint_count(
            "... situation was emblematic of a publication that ...",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_half() {
        // Can be an adjective in e.g. "half man, half machine"
        assert_lint_count("And now I only have half of a CyberTruck", AdjectiveOfA, 0);
    }

    #[test]
    fn dont_flag_bit() {
        // Technically also an adj as in "that guy's bit - he'll turn into a zombie"
        assert_lint_count("we ran into a bit of an issue", AdjectiveOfA, 0);
    }

    #[test]
    fn dont_flag_dream() {
        // Can be an adjective in e.g. "we built our dream house"
        assert_lint_count("When the dream of a united Europe began", AdjectiveOfA, 0);
    }

    #[test]
    fn dont_flag_beginning() {
        // Present participles have properties of adjectives, nouns, and verbs
        assert_lint_count("That's the beginning of a conversation.", AdjectiveOfA, 0);
    }

    #[test]
    fn dont_flag_side() {
        // Can be an adjective in e.g. "via a side door"
        assert_lint_count(
            "it hit the barrier on the side of a highway",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_derivative() {
        // Adj: "a derivative story", Noun: "stocks and derivatives"
        assert_lint_count(
            "Techniques for evaluating the *partial derivative of a function",
            AdjectiveOfA,
            0,
        )
    }

    #[test]
    fn dont_flag_equivalent() {
        assert_lint_count(
            "Rust's equivalent of a switch statement is a match expression",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_clockwork() {
        assert_lint_count(
            "so something's wrong in this clockwork of a thing and I'm not going to bother taking this apart",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_up() {
        assert_lint_count(
            "Yeah gas is made up of a bunch of teenytiny particles all moving around.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_eighth() {
        assert_lint_count(
            "It's about an eighth of an inch or whatever",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_shy() {
        assert_lint_count(
            "... or just shy of a third of the country's total trade deficit.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_fun() {
        assert_lint_count(
            "Remember that $4,000 Hermes horse bag I was making fun of a little while ago.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_off() {
        // Can be an adjective in e.g. "The TV is off".
        // This should be in a different lint that handles based on/off/off of.
        assert_lint_count(
            "can't identify a person based off of an IP from 10 years ago",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_borderline_of() {
        assert_lint_count(
            "it's very very on the borderline of a rock pop ballad",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_light() {
        assert_lint_count("The light of a star.", AdjectiveOfA, 0);
    }

    #[test]
    fn dont_flag_multiple() {
        assert_lint_count(
            "The image needs to be a multiple of a certain size.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_red() {
        assert_lint_count("The red of a drop of blood.", AdjectiveOfA, 0);
    }

    #[test]
    fn dont_flag_top() {
        assert_lint_count("The top of a hill.", AdjectiveOfA, 0);
    }

    #[test]
    fn dont_flag_slack() {
        assert_lint_count(
            "They've been picking up the slack of a federal government mostly dominated by whatever this is.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_illustrative() {
        assert_lint_count(
            "Yet, the fact that they clearly give a one-sided account of most of their case studies is illustrative of a bias.",
            AdjectiveOfA,
            0,
        );
    }
}
