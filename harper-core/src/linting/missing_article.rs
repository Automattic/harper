use harper_brill::UPOS;

use crate::Token;
use crate::expr::Expr;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::expr::UnlessStep;
use crate::patterns::AnyPattern;
use crate::patterns::UPOSSet;
use crate::patterns::WordSet;
use crate::starts_with_vowel::starts_with_vowel;

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct MissingArticle {
    expr: Box<dyn Expr>,
}

impl Default for MissingArticle {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .then_unless(
                UPOSSet::new(&[UPOS::DET, UPOS::ADJ, UPOS::ADV, UPOS::PRON, UPOS::AUX])
                    .or(WordSet::new(&["means"])),
            )
            .t_ws()
            .then(|tok: &Token, _: &[char]| {
                tok.kind.is_upos(UPOS::VERB)
                    && !tok.kind.is_verb_progressive_form()
                    && !tok.kind.is_linking_verb()
                    && !tok.kind.is_auxiliary_verb()
            })
            .t_ws()
            .then(UPOSSet::new(&[UPOS::NOUN, UPOS::PROPN]).and_not(WordSet::new(&["forces"])))
            .then_optional(AnyPattern)
            .then_optional(UnlessStep::new(
                UPOSSet::new_nominal(),
                |_: &Token, _: &[char]| true,
            ));

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for MissingArticle {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let first_meta = matched_tokens.first()?.kind.as_word()?.as_ref()?;

        if first_meta.pos_tag.is_none()
            && (first_meta.is_determiner()
                || first_meta.is_adjective()
                || first_meta.is_possessive_noun())
        {
            return None;
        }

        let object = matched_tokens.get(4)?;
        let singular = object.kind.is_singular_nominal();

        let indef = if singular {
            if starts_with_vowel(object.span.get_content(source)) {
                "an"
            } else {
                "a"
            }
        } else {
            return None;
        };

        let replacement_indefinite =
            format!("{} {}", indef, object.span.get_content_string(source));
        let replacement_definite = format!("the {}", object.span.get_content_string(source));

        Some(Lint {
            span: object.span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![
                Suggestion::replace_with_str(replacement_indefinite),
                Suggestion::replace_with_str(replacement_definite),
            ],
            message: "Consider adding an article before this noun.".to_owned(),
            priority: 0,
        })
    }

    fn description(&self) -> &'static str {
        "Flags nouns that follow a verb without an article or quantifier."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_top3_suggestion_result};

    use super::MissingArticle;

    #[test]
    fn fixes_good_day() {
        assert_top3_suggestion_result(
            "I had good day.",
            MissingArticle::default(),
            "I had good day.",
        );
    }

    #[test]
    fn fixes_cat_feather() {
        assert_top3_suggestion_result(
            "The cat chased feather.",
            MissingArticle::default(),
            "The cat chased a feather.",
        );
    }

    #[test]
    fn fixes_baker_sprinkles() {
        assert_top3_suggestion_result(
            "The baker decorated cake with sprinkles.",
            MissingArticle::default(),
            "The baker decorated the cake with sprinkles.",
        );
    }

    #[test]
    fn fixes_student_novel() {
        assert_top3_suggestion_result(
            "The student read novel.",
            MissingArticle::default(),
            "The student read a novel.",
        );
    }

    #[test]
    fn fixes_artist_landscape() {
        assert_top3_suggestion_result(
            "The artist painted landscape.",
            MissingArticle::default(),
            "The artist painted a landscape.",
        );
    }

    #[test]
    fn fixes_child_balloon() {
        assert_top3_suggestion_result(
            "The child held balloon.",
            MissingArticle::default(),
            "The child held a balloon.",
        );
    }

    #[test]
    fn fixes_dog_bone() {
        assert_top3_suggestion_result(
            "The dog buried bone.",
            MissingArticle::default(),
            "The dog buried a bone.",
        );
    }

    #[test]
    fn fixes_teacher_lesson() {
        assert_top3_suggestion_result(
            "The teacher presented lesson.",
            MissingArticle::default(),
            "The teacher presented a lesson.",
        );
    }

    #[test]
    fn fixes_musician_guitar() {
        assert_top3_suggestion_result(
            "The musician played guitar.",
            MissingArticle::default(),
            "The musician played a guitar.",
        );
    }

    #[test]
    fn fixes_hiker_map() {
        assert_top3_suggestion_result(
            "The hiker carried map.",
            MissingArticle::default(),
            "The hiker carried a map.",
        );
    }

    #[test]
    fn fixes_chef_salad() {
        assert_top3_suggestion_result(
            "The chef prepared salad.",
            MissingArticle::default(),
            "The chef prepared a salad.",
        );
    }

    #[test]
    fn allow_translation_strings() {
        assert_no_lints(
            "This option can be set to any locale for which you plan to have translation strings.",
            MissingArticle::default(),
        );
    }
}
