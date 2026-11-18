use crate::{
    Dialect, Lint, Token, TokenStringExt,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    indefinite_article::{InitialSound, starts_with_vowel},
    linting::{
        ExprLinter, LintGroup, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_word},
    },
};

struct GonnaGottaWanna {
    expr: SequenceExpr,
    dialect: Dialect,
    informal: &'static str,
    formal: &'static str,
}

impl GonnaGottaWanna {
    fn new(dialect: Dialect, informal: &'static str, formal: &'static str) -> Self {
        Self {
            expr: SequenceExpr::aco(informal)
                .then_optional(SequenceExpr::whitespace().t_set(["to", "a"])),
            dialect,
            informal,
            formal,
        }
    }
}

enum Verb {
    Gonna,
    Gotta,
    Wanna,
}
#[derive(PartialEq)]
enum Rest {
    None,
    To,
    A,
}

impl ExprLinter for GonnaGottaWanna {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let verb_ch = &toks.first()?.get_ch(src).get(..5)?;
        let full_span = toks.span()?;
        let full_ch = full_span.get_content(src);

        let verb_enum = match () {
            _ if verb_ch.eq_str("gonna") => Verb::Gonna,
            _ if verb_ch.eq_str("gotta") => Verb::Gotta,
            _ if verb_ch.eq_str("wanna") => Verb::Wanna,
            _ => return None,
        };

        let rest = match toks.get(2).map(|t| t.get_ch(src)) {
            Some(ch) if ch.eq_str("to") => Rest::To,
            Some(ch) if ch.eq_str("a") => Rest::A,
            _ => Rest::None,
        };

        let mut suggestions = Vec::new();

        if rest != Rest::A {
            suggestions.push(Suggestion::replace_with_match_case(
                self.formal.chars().chain(" to".chars()).collect(),
                full_ch,
            ));
        }

        let fixed_grammar = rest != Rest::None;

        match (verb_enum, rest) {
            (Verb::Gonna, Rest::None) => {}
            (Verb::Gonna, Rest::To) => {
                suggestions.push(Suggestion::ReplaceWith(verb_ch.to_vec()));
            }
            (Verb::Gonna, Rest::A) => {}
            (Verb::Gotta, Rest::None) => {
                suggestions.push(Suggestion::replace_with_match_case(
                    "have to".chars().collect(),
                    full_ch,
                ));
            }
            (Verb::Gotta, Rest::To) => {
                suggestions.push(Suggestion::ReplaceWith(verb_ch.to_vec()));
                suggestions.push(Suggestion::replace_with_match_case(
                    "have to".chars().collect(),
                    full_ch,
                ));
            }
            (Verb::Gotta, Rest::A) => {
                let followed_by_vowel = followed_by_word(ctx, |t| {
                    matches!(
                        starts_with_vowel(t.get_ch(src), self.dialect),
                        Some(InitialSound::Vowel)
                    )
                });

                let text = if followed_by_vowel { "got an" } else { "got a" };
                suggestions.push(Suggestion::replace_with_match_case(
                    text.chars().collect(),
                    full_ch,
                ));
            }
            (Verb::Wanna, Rest::None) => {}
            (Verb::Wanna, Rest::To) => {
                suggestions.push(Suggestion::ReplaceWith(verb_ch.to_vec()));
            }
            (Verb::Wanna, Rest::A) => {
                suggestions.push(Suggestion::replace_with_match_case(
                    "want a".chars().collect(),
                    full_ch,
                ));
            }
        };

        let message = if fixed_grammar {
            format!(
                "`{}` is very informal and the final `a` means `to`.",
                self.informal
            )
        } else {
            format!("`{}` is very informal.", self.informal)
        };

        Some(Lint {
            span: full_span,
            lint_kind: LintKind::Miscellaneous,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects the informal contractions `gonna`, `gotta`, and `wanna` to their full forms."
    }
}

pub fn lint_group(dialect: Dialect) -> LintGroup {
    let mut group = LintGroup::empty();

    let rules = [
        ("Gonna", "gonna", "going"),
        ("Gotta", "gotta", "got"),
        ("Wanna", "wanna", "want"),
    ];

    for &(name, informal, formal) in &rules {
        group.add(
            name,
            Box::new(GonnaGottaWanna::new(dialect, informal, formal)),
        );
    }

    group.set_all_rules_to(Some(true));
    group
}

#[cfg(test)]
mod tests {
    use crate::{
        Dialect,
        linting::tests::{
            assert_good_and_bad_suggestions, assert_lint_message, assert_suggestion_result,
        },
    };

    use super::lint_group;

    #[test]
    fn fix_gonna() {
        // We think you're gonna like it here."
        assert_suggestion_result(
            "We think you're gonna like it here.",
            lint_group(Dialect::American),
            // fix the grammar and the informality
            "We think you're going to like it here.",
        )
    }

    #[test]
    fn fix_gonna_to() {
        // "gonna" already means "going to", so adding "to" is redundant
        assert_good_and_bad_suggestions(
            "I am gonna to create the region by clicking and dragging the mouse on waveform but I couldn't find the way to do that.",
            lint_group(Dialect::American),
            &[
                // fix the grammar but remain informal
                "I am gonna create the region by clicking and dragging the mouse on waveform but I couldn't find the way to do that.",
                // fix the grammar and the informality
                "I am going to create the region by clicking and dragging the mouse on waveform but I couldn't find the way to do that.",
            ],
            &[],
        )
    }

    #[test]
    fn fix_gotta() {
        assert_suggestion_result(
            "Gotta Hear Them All: Towards Sound Source Aware Audio Generation.",
            lint_group(Dialect::American),
            "Got to Hear Them All: Towards Sound Source Aware Audio Generation.",
            // What about "have to"?
        )
    }

    #[test]
    fn fix_gotta_a_error() {
        // "gotta" only means "(have) got to", using it as "got a" is incorrect
        // but adding "a" is also redundant
        assert_suggestion_result(
            "when I try to create a c/c++ database,I gotta a error",
            lint_group(Dialect::American),
            // fix the grammar and the informality
            "when I try to create a c/c++ database,I got an error",
        )
    }

    #[test]
    fn fix_gotta_a_coupom() {
        // "gotta" only means "(have) got to", using it as "got a" is incorrect
        // but adding "a" is also redundant
        assert_suggestion_result(
            "You gotta a 20% OFF coupom.",
            lint_group(Dialect::American),
            "You got a 20% OFF coupom.",
        )
    }

    #[test]
    // it works well enough for me and gotta to get back working on other stuff
    fn fix_gotta_to() {
        // "gotta" already means "got to", so adding "to" is redundant
        assert_good_and_bad_suggestions(
            "it works well enough for me and gotta to get back working on other stuff",
            lint_group(Dialect::American),
            &[
                // fix the grammar but remain informal
                "it works well enough for me and gotta get back working on other stuff",
                // fix the grammar and the informality
                "it works well enough for me and have to get back working on other stuff",
                // What about "has to", "had to"?
            ],
            &[],
        )
    }

    #[test]
    fn fix_wanna_control() {
        assert_suggestion_result(
            "I wanna control G1 arms while walking using gamepad. But ...",
            lint_group(Dialect::American),
            "I want to control G1 arms while walking using gamepad. But ...",
        )
    }

    #[test]
    fn fix_wanna_a() {
        // "wanna" only means "want to", using it as "want a" is incorrect
        // Only way to fix fixes both the informality and the grammar goether
        assert_suggestion_result(
            "And then I remember from the beginning I just wanna a very simple array with some simple opreations.",
            lint_group(Dialect::American),
            "And then I remember from the beginning I just want a very simple array with some simple opreations.",
        )
    }

    #[test]
    fn fix_wanna_to() {
        // "wanna" already means "want to", so adding "to" is redundant
        assert_good_and_bad_suggestions(
            "it is stoped but i wanna to continue",
            lint_group(Dialect::American),
            &[
                // fix the grammar but remain informal
                "it is stoped but i wanna continue",
                // fix the grammar and the informality
                "it is stoped but i want to continue",
            ],
            &[],
        )
    }

    // Check lint messages

    #[test]
    fn gonna_none_msg() {
        assert_lint_message(
            "gonna",
            lint_group(Dialect::American),
            "`gonna` is very informal.",
        )
    }

    #[test]
    fn gonna_a_msg() {
        assert_lint_message(
            "gonna a",
            lint_group(Dialect::American),
            "`gonna` is very informal and the final `a` means `to`.",
        )
    }

    #[test]
    fn gonna_to_msg() {
        assert_lint_message(
            "gonna to",
            lint_group(Dialect::American),
            "`gonna` is very informal and the final `a` means `to`.",
        )
    }

    #[test]
    fn gotta_noun_msg() {
        assert_lint_message(
            "gotta",
            lint_group(Dialect::American),
            "`gotta` is very informal.",
        )
    }

    #[test]
    fn gotta_a_msg() {
        assert_lint_message(
            "gotta a",
            lint_group(Dialect::American),
            "`gotta` is very informal and the final `a` means `to`.",
        )
    }

    #[test]
    fn gotta_to_msg() {
        assert_lint_message(
            "gotta to",
            lint_group(Dialect::American),
            "`gotta` is very informal and the final `a` means `to`.",
        )
    }

    #[test]
    fn wanna_none_msg() {
        assert_lint_message(
            "wanna",
            lint_group(Dialect::American),
            "`wanna` is very informal.",
        )
    }

    #[test]
    fn wanna_a_msg() {
        assert_lint_message(
            "wanna a",
            lint_group(Dialect::American),
            "`wanna` is very informal and the final `a` means `to`.",
        )
    }

    #[test]
    fn wanna_to_msg() {
        assert_lint_message(
            "wanna to",
            lint_group(Dialect::American),
            "`wanna` is very informal and the final `a` means `to`.",
        )
    }
}
