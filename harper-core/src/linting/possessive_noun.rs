use harper_brill::UPOS;

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::expr::{All, Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::patterns::{UPOSSet, WordSet};
use crate::spell::Dictionary;
use crate::{Token, TokenKind};

pub struct PossessiveNoun<D> {
    expr: Box<dyn Expr>,
    dict: D,
}

impl<D> PossessiveNoun<D>
where
    D: Dictionary,
{
    pub fn new(dict: D) -> Self {
        let expr = SequenceExpr::with(UPOSSet::new(&[UPOS::DET, UPOS::PROPN]))
            .t_ws()
            .then_kind_is_but_is_not(TokenKind::is_plural_nominal, TokenKind::is_singular_nominal)
            .t_ws()
            .then(UPOSSet::new(&[UPOS::NOUN, UPOS::PROPN]))
            .then_optional(SequenceExpr::anything().t_any());

        let additional_req = SequenceExpr::anything().t_any().t_any().t_any().then_noun();

        let exceptions =
            SequenceExpr::unless(|tok: &Token, _: &[char]| tok.kind.is_demonstrative_determiner())
                .t_any()
                .then_unless(WordSet::new(&["flags", "checks", "catches", "you"]))
                .t_any()
                .then_unless(WordSet::new(&["form", "go"]));

        Self {
            expr: Box::new(All::new(vec![
                Box::new(expr),
                Box::new(additional_req),
                Box::new(exceptions),
            ])),
            dict,
        }
    }
}

impl<D> ExprLinter for PossessiveNoun<D>
where
    D: Dictionary,
{
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        let last_kind = &matched_tokens.last()?.kind;

        if last_kind.is_upos(UPOS::ADV) {
            return None;
        }

        let first = matched_tokens.get(2)?;

        let span = first.span;
        let plural = span.get_content_string(_source);
        let singular = if plural.ends_with('s') {
            &plural[..plural.len() - 1]
        } else {
            &plural
        };

        let replacement = format!("{singular}'s");

        if !self.dict.contains_word_str(&replacement) {
            return None;
        }

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith(replacement.chars().collect())],
            message: self.description().to_string(),
            priority: 10,
        })
    }

    fn description(&self) -> &'static str {
        "Use an apostrophe and `s` to form a noun’s possessive."
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::PossessiveNoun;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};
    use crate::spell::FstDictionary;

    fn test_linter() -> PossessiveNoun<Arc<FstDictionary>> {
        PossessiveNoun::new(FstDictionary::curated())
    }

    /// Sourced from a Hacker News comment
    #[test]
    fn fixes_hn() {
        assert_suggestion_result(
            "Me and Jennifer went to have seen the ducks cousin.",
            test_linter(),
            "Me and Jennifer went to have seen the duck's cousin.",
        );
    }

    #[test]
    fn fixes_cats_tail() {
        assert_suggestion_result(
            "The cats tail is long.",
            test_linter(),
            "The cat's tail is long.",
        );
    }

    #[test]
    fn fixes_children_toys() {
        assert_suggestion_result(
            "The children toys were scattered.",
            test_linter(),
            "The children's toys were scattered.",
        );
    }

    #[test]
    fn fixes_teachers_lounge() {
        assert_suggestion_result(
            "Many schools have a teachers lounge.",
            test_linter(),
            "Many schools have a teacher's lounge.",
        );
    }

    #[test]
    fn fixes_ducks_park() {
        assert_suggestion_result(
            "Kids played in the ducks park.",
            test_linter(),
            "Kids played in the duck's park.",
        );
    }

    #[test]
    fn no_lint_for_already_possessive() {
        assert_lint_count("The duck's cousin visited.", test_linter(), 0);
    }

    #[test]
    fn no_lint_for_alrreaady_possessive() {
        assert_lint_count("The duck's cousin visited.", test_linter(), 0);
    }

    #[test]
    fn fixes_dogs_bone() {
        assert_suggestion_result(
            "The dogs bone is delicious.",
            test_linter(),
            "The dog's bone is delicious.",
        );
    }

    #[test]
    fn fixes_students_books() {
        assert_suggestion_result(
            "The students books are on the desk.",
            test_linter(),
            "The student's books are on the desk.",
        );
    }

    #[test]
    fn fixes_farmers_field() {
        assert_suggestion_result(
            "The farmers field looked beautiful.",
            test_linter(),
            "The farmer's field looked beautiful.",
        );
    }

    #[test]
    fn fixes_women_dress() {
        assert_suggestion_result(
            "The women dress was elegant.",
            test_linter(),
            "The women's dress was elegant.",
        );
    }

    #[test]
    fn fixes_birds_song() {
        assert_suggestion_result(
            "We heard the birds song.",
            test_linter(),
            "We heard the bird's song.",
        );
    }

    #[test]
    fn fixes_scientists_research() {
        assert_suggestion_result(
            "The scientists research was groundbreaking.",
            test_linter(),
            "The scientist's research was groundbreaking.",
        );
    }

    #[test]
    fn fixes_artists_gallery() {
        assert_suggestion_result(
            "The artists gallery is open.",
            test_linter(),
            "The artist's gallery is open.",
        );
    }

    #[test]
    fn no_lint_for_plural_noun() {
        assert_lint_count("The ducks are swimming.", test_linter(), 0);
    }

    #[test]
    fn no_lint_for_proper_noun() {
        assert_lint_count("John's car is red.", test_linter(), 0);
    }

    #[test]
    fn fixes_the_students_assignment() {
        assert_suggestion_result(
            "The students assignment was due yesterday.",
            test_linter(),
            "The student's assignment was due yesterday.",
        );
    }

    #[test]
    fn fixes_the_birds_flight() {
        assert_suggestion_result(
            "The birds flight was graceful.",
            test_linter(),
            "The bird's flight was graceful.",
        );
    }

    #[test]
    fn allows_the_city_lights() {
        assert_lint_count(
            "The city lights twinkled in the distance.",
            test_linter(),
            0,
        );
    }

    #[test]
    fn fixes_the_farmers_crops() {
        assert_suggestion_result(
            "The farmers crops were bountiful this year.",
            test_linter(),
            "The farmer's crops were bountiful this year.",
        );
    }

    #[test]
    fn fixes_the_artists_inspiration() {
        assert_suggestion_result(
            "The artists inspiration came from nature.",
            test_linter(),
            "The artist's inspiration came from nature.",
        );
    }

    #[test]
    fn fixes_the_scientists_discovery() {
        assert_suggestion_result(
            "The scientists discovery revolutionized the field.",
            test_linter(),
            "The scientist's discovery revolutionized the field.",
        );
    }

    #[test]
    fn fixes_the_writers_novel() {
        assert_suggestion_result(
            "The writers novel was a bestseller.",
            test_linter(),
            "The writer's novel was a bestseller.",
        );
    }

    #[test]
    fn fixes_the_students_presentation() {
        assert_suggestion_result(
            "The students presentation was well-received.",
            test_linter(),
            "The student's presentation was well-received.",
        );
    }

    #[test]
    fn fixes_the_teams_victory() {
        assert_suggestion_result(
            "The teams victory was celebrated by the fans.",
            test_linter(),
            "The team's victory was celebrated by the fans.",
        );
    }

    #[test]
    fn fixes_the_museums_collection() {
        assert_suggestion_result(
            "The museums collection included many artifacts.",
            test_linter(),
            "The museum's collection included many artifacts.",
        );
    }

    #[test]
    fn no_lint_for_already_possessive_2() {
        assert_lint_count("John's car is red.", test_linter(), 0);
    }

    #[test]
    fn no_lint_for_proper_noun_2() {
        assert_lint_count("Mary went to the store.", test_linter(), 0);
    }

    #[test]
    fn fixes_the_doctors_office() {
        assert_suggestion_result(
            "The doctors office is on Main Street.",
            test_linter(),
            "The doctor's office is on Main Street.",
        );
    }

    #[test]
    fn fixes_the_neighbors_garden() {
        assert_suggestion_result(
            "The neighbors garden is beautiful.",
            test_linter(),
            "The neighbor's garden is beautiful.",
        );
    }

    #[test]
    fn fixes_the_architects_design() {
        assert_suggestion_result(
            "The architects design was innovative.",
            test_linter(),
            "The architect's design was innovative.",
        );
    }

    #[test]
    fn fixes_the_bakers_shop() {
        assert_suggestion_result(
            "The bakers shop is famous for its bread.",
            test_linter(),
            "The baker's shop is famous for its bread.",
        );
    }

    #[test]
    fn fixes_the_musics_performance() {
        assert_suggestion_result(
            "The musics performance was captivating.",
            test_linter(),
            "The music's performance was captivating.",
        );
    }

    #[test]
    fn fixes_the_flowers_scent() {
        assert_suggestion_result(
            "The flowers scent filled the room.",
            test_linter(),
            "The flower's scent filled the room.",
        );
    }

    #[test]
    fn allows_birds_hurried() {
        assert_lint_count("The birds hurried off.", test_linter(), 0);
    }

    #[test]
    #[ignore = "false positive issue 1582"]
    fn allows_1582_harms_readability() {
        assert_lint_count(
            "This harms readability and maintainability.",
            test_linter(),
            0,
        );
    }

    #[test]
    #[ignore = "false positive issue 1582"]
    fn allows_1582_imports_couples() {
        assert_lint_count(
            "Since using Webpack syntax in the imports couples the code to a module bundler",
            test_linter(),
            0,
        );
    }

    #[test]
    #[ignore = "false positive issue 1582"]
    fn allows_1582_graphics_programmer() {
        assert_lint_count(
            "Are you a graphics programmer or Rust developer?",
            test_linter(),
            0,
        );
    }

    #[test]
    #[ignore = "false positive issue 1582"]
    fn allows_1582_data_sources() {
        assert_lint_count(
            "these data sources can be queried using a full SQL dialect",
            test_linter(),
            0,
        );
    }
}
