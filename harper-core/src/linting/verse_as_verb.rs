use crate::{
    CharStringExt, Document, Token,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, Lint, LintKind, Linter, MapPhraseSetLinter, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
        merge_linters::merge_linters,
    },
};

const DESCRIPTION: &str =
    "Corrects nonstandard uses of `verse` where `versus` or a verb meaning to compete is intended.";

struct VerseVerbPhrases(MapPhraseSetLinter<'static>);

impl Default for VerseVerbPhrases {
    fn default() -> Self {
        Self(MapPhraseSetLinter::many_to_many(
            &[
                (&["verse against"], &["play against", "compete against"]),
                (&["versed against"], &["played against", "competed against"]),
                (
                    &["versing against"],
                    &["playing against", "competing against"],
                ),
                (&["verses against"], &["plays against", "competes against"]),
                (
                    &["verse me"],
                    &["play me", "play against me", "compete against me"],
                ),
                (
                    &["verse him"],
                    &["play him", "play against him", "compete against him"],
                ),
                (
                    &["verse her"],
                    &["play her", "play against her", "compete against her"],
                ),
                (
                    &["verse them"],
                    &["play them", "play against them", "compete against them"],
                ),
                (
                    &["verse you"],
                    &["play you", "play against you", "compete against you"],
                ),
            ],
            "`Verse` is not a verb meaning to compete. Use `play against` or `compete against` instead.",
            DESCRIPTION,
            Some(LintKind::Nonstandard),
        ))
    }
}

impl Linter for VerseVerbPhrases {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        self.0.lint(document)
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }
}

struct VerseComparison {
    expr: SequenceExpr,
}

impl Default for VerseComparison {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_word()
                .t_ws()
                .t_aco("verse")
                .t_ws()
                .then_kind_where(|kind| kind.is_adjective() || kind.is_noun())
                .t_ws()
                .then_kind_where(|kind| kind.is_adjective() || kind.is_noun())
                .t_ws()
                .then_kind_where(|kind| kind.is_proper_noun()),
        }
    }
}

impl ExprLinter for VerseComparison {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if !toks.first()?.get_ch(src).first()?.is_uppercase() {
            return None;
        }
        let verse =
            find_the_only_token_matching(toks, src, |tok, src| tok.get_ch(src).eq_str("verse"))?;
        Some(Lint {
            span: verse.span,
            lint_kind: LintKind::Nonstandard,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "versus",
                verse.get_ch(src),
            )],
            message: "Use `versus` to compare the two competitors.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }
}

merge_linters!(VerseAsVerb => VerseVerbPhrases, VerseComparison => super::DESCRIPTION);

#[cfg(test)]
mod tests {
    use super::VerseAsVerb;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};
    use crate::{Dialect, linting::LintGroup, spell::FstDictionary};
    fn test_linter() -> VerseAsVerb {
        VerseAsVerb::default()
    }
    fn single_lint(name: &str) -> LintGroup {
        let mut group = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);
        group.set_all_rules_to(Some(false));
        group.config.set_rule_enabled(name, true);
        group
    }
    // VerseAsVerb

    #[test]
    fn corrects_verse_against() {
        assert_suggestion_result(
            "A game of Morra, with 3 different AI you can verse against.",
            test_linter(),
            "A game of Morra, with 3 different AI you can play against.",
        );
    }

    #[test]
    fn corrects_versing_against() {
        assert_suggestion_result(
            "This will help when you are versing against a particular boss.",
            test_linter(),
            "This will help when you are playing against a particular boss.",
        );
    }

    #[test]
    fn corrects_verse_me() {
        assert_suggestion_result(
            "Come verse me in this game.",
            test_linter(),
            "Come play me in this game.",
        );
    }

    #[test]
    fn allows_versus() {
        assert_no_lints("It was red versus blue in the finals.", test_linter());
    }

    #[test]
    fn corrects_verse_between_competitors() {
        assert_suggestion_result(
            "Today it's all-wheel drive Civic verse twin turbo Lamborghini.",
            test_linter(),
            "Today it's all-wheel drive Civic versus twin turbo Lamborghini.",
        );
    }

    #[test]
    fn corrects_verse_between_other_competitors() {
        assert_suggestion_result(
            "Mustang verse twin turbo Ferrari.",
            test_linter(),
            "Mustang versus twin turbo Ferrari.",
        );
    }

    #[test]
    fn allows_lowercase_noun_verse_context() {
        assert_no_lints("A verse twin poets remember.", test_linter());
    }

    #[test]
    fn allows_verse_with_non_proper_noun_tail() {
        assert_no_lints("Civic verse twin turbo engines.", test_linter());
    }

    #[test]
    fn allows_noun_verse() {
        assert_no_lints("She read a verse in the poem.", test_linter());
    }

    #[test]
    fn allows_well_versed() {
        assert_no_lints("He is well versed in English literature.", test_linter());
    }

    #[test]
    fn preserves_capitalized_verb() {
        assert_suggestion_result("Verse me!", test_linter(), "Play me!");
    }

    #[test]
    fn offers_inflected_alternatives() {
        use crate::linting::tests::assert_good_and_bad_suggestions;
        assert_good_and_bad_suggestions(
            "She verses against him.",
            test_linter(),
            &["She plays against him.", "She competes against him."],
            &[],
        );
    }

    #[test]
    fn preserves_poetry_contexts() {
        for text in [
            "Blank verse was what Shakespeare preferred.",
            "English verse is what Shakespeare wrote.",
            "A verse was by Shakespeare.",
        ] {
            assert_no_lints(text, single_lint("VerseAsVerb"));
        }
    }

    #[test]
    fn offers_competition_alternatives() {
        use crate::linting::tests::assert_good_and_bad_suggestions;
        assert_good_and_bad_suggestions(
            "Come verse me.",
            single_lint("VerseAsVerb"),
            &[
                "Come play me.",
                "Come play against me.",
                "Come compete against me.",
            ],
            &[],
        );
        assert_good_and_bad_suggestions(
            "We versed against them.",
            single_lint("VerseAsVerb"),
            &["We played against them.", "We competed against them."],
            &[],
        );
    }

    #[test]
    fn single_toggle_controls_both_forms() {
        let text = "Come verse me. Civic verse twin turbo Lamborghini.";
        let mut group = single_lint("VerseAsVerb");
        assert_eq!(
            group
                .iter_keys()
                .filter(|name| *name == "VerseAsVerb")
                .count(),
            1
        );
        use crate::{Document, linting::Linter};
        let document = Document::new_plain_english_curated(text);
        assert_eq!(group.lint(&document).len(), 2);
        group.config.set_rule_enabled("VerseAsVerb", false);
        assert_no_lints(text, group);
    }
}
