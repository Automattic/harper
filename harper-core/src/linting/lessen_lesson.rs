use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    token_kind::TokenKind,
};

const MODALS: &[&str] = &[
    "will",
    "would",
    "can",
    "could",
    "may",
    "might",
    "must",
    "shall",
    "should",
    "cannot",
    "won't",
    "wouldn't",
    "can't",
    "couldn't",
    "shouldn't",
    "mustn't",
    "shan't",
];

const DO_FORMS: &[&str] = &["did", "do", "does"];

const DO_NEGATIONS: &[&str] = &["didn't", "don't", "doesn't"];

/// Pronouns that are always singular subjects, so a following `lessons`
/// can only be a (misused) verb.
const SINGULAR_SUBJECT_PRONOUNS: &[&str] = &[
    "it",
    "everything",
    "nothing",
    "something",
    "anything",
    "everyone",
    "nobody",
    "somebody",
    "anybody",
];

/// Words that can introduce a clause or act as a singular demonstrative
/// subject; a following `lessons` is only a verb when a noun phrase
/// object follows it.
const CLAUSE_SUBJECTS: &[&str] = &["this", "that", "which", "what", "who"];

fn is_modal_or_auxiliary(word: &str) -> bool {
    MODALS.contains(&word) || DO_FORMS.contains(&word) || DO_NEGATIONS.contains(&word)
}

/// A lesson/lessen confusion is only flagged when a noun phrase object
/// follows, which rules out noun readings like "to lesson 5" and
/// "listen to lesson recordings".
fn followed_by_determiner(sequence: SequenceExpr) -> SequenceExpr {
    sequence
        .t_ws()
        .then_kind_where(|kind: &TokenKind| kind.is_determiner())
}

/// For singular `lesson`, an adverb after the verb keeps the verb reading
/// ("will lesson greatly"), which plural `lessons` can't have without a
/// verb between ("IT lessons greatly improve").
fn followed_by_determiner_or_adverb(sequence: SequenceExpr) -> SequenceExpr {
    sequence
        .t_ws()
        .then_kind_where(|kind: &TokenKind| kind.is_determiner() || kind.is_adverb())
}

pub struct LessenLesson {
    expr: FirstMatchOf,
}

impl Default for LessenLesson {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                // "to lesson the impact" -> "to lessen the impact"
                Box::new(followed_by_determiner(
                    SequenceExpr::aco("to")
                        .t_ws()
                        .then(SequenceExpr::aco("lesson")),
                )) as Box<dyn Expr>,
                // "will lesson the impact" / "will not lesson the impact" /
                // "will lesson greatly"
                Box::new(followed_by_determiner_or_adverb(
                    SequenceExpr::word_set(MODALS)
                        .t_ws()
                        .then_optional(SequenceExpr::aco("not").t_ws())
                        .then(SequenceExpr::aco("lesson")),
                )) as Box<dyn Expr>,
                // "will lessons the pain" -> "will lessen the pain"
                Box::new(followed_by_determiner(
                    SequenceExpr::word_set(MODALS)
                        .t_ws()
                        .then_optional(SequenceExpr::aco("not").t_ws())
                        .then(SequenceExpr::aco("lessons")),
                )) as Box<dyn Expr>,
                // "did not lesson the tension" / "does lesson the tension"
                Box::new(followed_by_determiner_or_adverb(
                    SequenceExpr::word_set(DO_FORMS)
                        .t_ws()
                        .then_optional(SequenceExpr::aco("not").t_ws())
                        .then(SequenceExpr::aco("lesson")),
                )) as Box<dyn Expr>,
                // "didn't lesson her spending"
                Box::new(followed_by_determiner_or_adverb(
                    SequenceExpr::word_set(DO_NEGATIONS)
                        .t_ws()
                        .then(SequenceExpr::aco("lesson")),
                )) as Box<dyn Expr>,
                // "it lessons the pain"
                Box::new(followed_by_determiner(
                    SequenceExpr::word_set(SINGULAR_SUBJECT_PRONOUNS)
                        .t_ws()
                        .then(SequenceExpr::aco("lessons")),
                )) as Box<dyn Expr>,
                // "a treatment that lessons the pain"
                Box::new(followed_by_determiner(
                    SequenceExpr::word_set(CLAUSE_SUBJECTS)
                        .t_ws()
                        .then(SequenceExpr::aco("lessons")),
                )) as Box<dyn Expr>,
                // "a lessen from history" / "the lessen" / "her lessen" /
                // "important lessen" / "the instructor's lessen"
                // `that` and `this` are excluded: "drugs that lessen the
                // pain" is a valid relative clause.
                Box::new(
                    SequenceExpr::with(|tok: &Token, _: &[char]| {
                        let kind = &tok.kind;
                        (kind.is_non_quantifier_determiner() && !kind.is_demonstrative_determiner())
                            || kind.is_adjective()
                            || kind.is_possessive_noun()
                    })
                    .t_ws()
                    .then(SequenceExpr::aco("lessen")),
                ) as Box<dyn Expr>,
                // "each lessen" / "every lessen"
                Box::new(
                    SequenceExpr::word_set(["each", "every"])
                        .t_ws()
                        .then(SequenceExpr::aco("lessen")),
                ) as Box<dyn Expr>,
                // "these lessens are expensive"
                Box::new(
                    SequenceExpr::word_set(["these", "those"])
                        .t_ws()
                        .then(SequenceExpr::aco("lessens")),
                ) as Box<dyn Expr>,
                // "the lessens of history"
                Box::new(
                    SequenceExpr::aco("the")
                        .t_ws()
                        .then(SequenceExpr::aco("lessens")),
                ) as Box<dyn Expr>,
                // "their lessens" / "his lessens" / "valuable lessens" /
                // "the instructor's lessens"
                Box::new(
                    SequenceExpr::with(|tok: &Token, _: &[char]| {
                        let kind = &tok.kind;
                        kind.is_possessive_determiner()
                            || kind.is_possessive_noun()
                            || kind.is_adjective()
                    })
                    .t_ws()
                    .then(SequenceExpr::aco("lessens")),
                ) as Box<dyn Expr>,
            ]),
        }
    }
}

impl ExprLinter for LessenLesson {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let conf_idx = toks.iter().position(|tok| {
            let word = tok.span.get_content(src);
            word.eq_str("lesson")
                || word.eq_str("lessons")
                || word.eq_str("lessen")
                || word.eq_str("lessens")
        })?;

        let conf_tok = &toks[conf_idx];
        let conf_span = conf_tok.span;
        let conf_chars = conf_span.get_content(src);

        // The word before the confused one, skipping an intervening "not"
        // ("will not lesson" -> predecessor is "will").
        let mut pred_word = toks[conf_idx - 2]
            .span
            .get_content_string(src)
            .to_lowercase();
        if pred_word == "not" && conf_idx >= 4 {
            pred_word = toks[conf_idx - 4]
                .span
                .get_content_string(src)
                .to_lowercase();
        }

        let (suggestion, message) = if conf_chars.eq_str("lesson") {
            (
                "lessen",
                "`Lesson` is a unit of learning; the verb meaning \"to make less\" is `lessen`.",
            )
        } else if conf_chars.eq_str("lessons") {
            if is_modal_or_auxiliary(&pred_word) {
                (
                    "lessen",
                    "After a modal or auxiliary verb, use the base form `lessen`.",
                )
            } else {
                (
                    "lessens",
                    "`Lessons` are units of learning; the verb form meaning \"makes less\" is `lessens`.",
                )
            }
        } else if conf_chars.eq_str("lessen") {
            (
                "lesson",
                "`Lessen` means \"to make less\"; a unit of learning is a `lesson`.",
            )
        } else {
            (
                "lessons",
                "`Lessens` is a verb form; the plural noun is `lessons`.",
            )
        };

        Some(Lint {
            span: conf_span,
            lint_kind: LintKind::Malapropism,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                suggestion, conf_chars,
            )],
            message: message.to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Looks for `lesson` used where the verb `lessen` is intended, and vice versa, including their `-s` forms."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::LessenLesson as Rule;

    // -- True positives: lesson(s) used for lessen(s) --

    #[test]
    fn correct_modal_lesson_to_lessen() {
        assert_suggestion_result(
            "We can lesson the risk by diversifying.",
            Rule::default(),
            "We can lessen the risk by diversifying.",
        );
    }

    #[test]
    fn correct_will_lesson_to_lessen() {
        assert_suggestion_result(
            "Nothing will lesson the impact of the storm.",
            Rule::default(),
            "Nothing will lessen the impact of the storm.",
        );
    }

    #[test]
    fn correct_to_lesson_to_lessen() {
        assert_suggestion_result(
            "To lesson his anxiety, she spoke softly.",
            Rule::default(),
            "To lessen his anxiety, she spoke softly.",
        );
    }

    #[test]
    fn correct_didnt_lesson_to_lessen() {
        assert_suggestion_result(
            "She didn't lesson her spending.",
            Rule::default(),
            "She didn't lessen her spending.",
        );
    }

    #[test]
    fn correct_did_not_lesson_to_lessen() {
        assert_suggestion_result(
            "That did not lesson her enthusiasm.",
            Rule::default(),
            "That did not lessen her enthusiasm.",
        );
    }

    #[test]
    fn correct_it_lessons_to_lessens() {
        assert_suggestion_result(
            "It lessons the tension.",
            Rule::default(),
            "It lessens the tension.",
        );
    }

    #[test]
    fn correct_that_lessons_to_lessens() {
        assert_suggestion_result(
            "A treatment that lessons the pain is welcome.",
            Rule::default(),
            "A treatment that lessens the pain is welcome.",
        );
    }

    #[test]
    fn correct_which_lessons_to_lessens() {
        assert_suggestion_result(
            "A drug which lessons the symptoms is still in trials.",
            Rule::default(),
            "A drug which lessens the symptoms is still in trials.",
        );
    }

    #[test]
    fn correct_will_lessons_to_lessen() {
        assert_suggestion_result(
            "Nothing will lessons the blow.",
            Rule::default(),
            "Nothing will lessen the blow.",
        );
    }

    // -- True positives: lessen(s) used for lesson(s) --

    #[test]
    fn correct_a_lessen_to_lesson() {
        assert_suggestion_result(
            "He watched a lessen from history.",
            Rule::default(),
            "He watched a lesson from history.",
        );
    }

    #[test]
    fn correct_the_lessen_to_lesson() {
        assert_suggestion_result(
            "The lessen was clear enough.",
            Rule::default(),
            "The lesson was clear enough.",
        );
    }

    #[test]
    fn correct_each_lessen_to_lesson() {
        assert_suggestion_result(
            "Each lessen taught us patience.",
            Rule::default(),
            "Each lesson taught us patience.",
        );
    }

    #[test]
    fn correct_adjective_lessen_to_lesson() {
        assert_suggestion_result(
            "An important lessen from the crisis is that redundancy matters.",
            Rule::default(),
            "An important lesson from the crisis is that redundancy matters.",
        );
    }

    #[test]
    fn correct_possessive_noun_lessen_to_lesson() {
        assert_suggestion_result(
            "The instructor's lessen was engaging.",
            Rule::default(),
            "The instructor's lesson was engaging.",
        );
    }

    #[test]
    fn correct_possessive_determiner_lessens_to_lessons() {
        assert_suggestion_result(
            "Their piano lessens helped her through a hard year.",
            Rule::default(),
            "Their piano lessons helped her through a hard year.",
        );
    }

    #[test]
    fn correct_these_lessens_to_lessons() {
        assert_suggestion_result(
            "These lessens are expensive.",
            Rule::default(),
            "These lessons are expensive.",
        );
    }

    #[test]
    fn correct_adjective_lessens_to_lessons() {
        assert_suggestion_result(
            "Valuable lessens were learned.",
            Rule::default(),
            "Valuable lessons were learned.",
        );
    }

    #[test]
    fn correct_the_lessens_to_lessons() {
        assert_suggestion_result(
            "The lessens of history are often ignored.",
            Rule::default(),
            "The lessons of history are often ignored.",
        );
    }

    #[test]
    fn correct_possessive_noun_lessens_to_lessons() {
        assert_suggestion_result(
            "The instructor's lessens were engaging.",
            Rule::default(),
            "The instructor's lessons were engaging.",
        );
    }

    // -- False positives: correct usages that must not be flagged --

    #[test]
    fn allow_correct_lessen_after_modal() {
        assert_no_lints("You should lessen the impact.", Rule::default());
    }

    #[test]
    fn allow_correct_lessens_verb() {
        assert_no_lints("The pain lessens over time.", Rule::default());
    }

    #[test]
    fn allow_correct_it_lessens() {
        assert_no_lints("It lessens the tension.", Rule::default());
    }

    #[test]
    fn allow_correct_the_lesson() {
        assert_no_lints("The lesson was hard.", Rule::default());
    }

    #[test]
    fn allow_correct_life_lessons() {
        assert_no_lints("Life's lessons are the best teacher.", Rule::default());
    }

    #[test]
    fn allow_correct_piano_lesson() {
        assert_no_lints("The piano lesson begins at noon.", Rule::default());
    }

    #[test]
    fn allow_lesson_number() {
        assert_no_lints("Skip to lesson 5.", Rule::default());
    }

    #[test]
    fn allow_capitalized_lesson_number() {
        assert_no_lints("Will Lesson 5 be on the test?", Rule::default());
    }

    #[test]
    fn allow_will_lessons_subject() {
        assert_no_lints("Will lessons be hard?", Rule::default());
    }

    #[test]
    fn allow_to_lessons_plural_noun() {
        assert_no_lints("I listen to lessons online.", Rule::default());
    }

    #[test]
    fn allow_does_lessons_idiom() {
        assert_no_lints("She does lessons online.", Rule::default());
    }

    #[test]
    fn allow_relative_that_lessen() {
        assert_no_lints("Drugs that lessen the pain help most.", Rule::default());
    }

    #[test]
    fn allow_to_lesson_pronoun_object() {
        assert_no_lints("To lesson him was her gift.", Rule::default());
    }

    #[test]
    fn allow_it_lessons_capitalized_tech() {
        assert_no_lints("IT lessons teach coding.", Rule::default());
    }

    #[test]
    fn allow_noun_lessons_verb_follower() {
        assert_no_lints("The piano lessons cost $20.", Rule::default());
    }

    #[test]
    fn allow_subjunctive_lessen() {
        assert_no_lints("They demanded that the pressure lessen.", Rule::default());
    }

    #[test]
    fn allow_he_lessons_archaic() {
        assert_no_lints(
            "He lessons students in the art of negotiation.",
            Rule::default(),
        );
    }

    #[test]
    fn allow_which_lessons_interrogative() {
        assert_no_lints("Which lessons do you remember?", Rule::default());
    }
}
