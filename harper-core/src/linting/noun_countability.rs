use crate::{
    Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::{IndefiniteArticle, WordSet},
};

#[derive(Debug, Clone, Copy)]
pub enum Correction {
    JustStuff,
    ReplaceWithSome,
    ReplaceWithAPieceOf,
    InsertBetweenPieceOf,
    InsertBetweenPiecesOf,
    ReplaceWithAll,
    ReplaceWithLittle,
    ReplaceWithMuch,
    ReplaceWithALotOf,
}

use Correction::*;

pub struct NounCountability {
    expr: Box<dyn Expr>,
}

impl Default for NounCountability {
    fn default() -> Self {
        let quantifier = WordSet::new(&[
            "another", "both", "each", "every", "few", "many", "one", "several",
        ]);

        // A determiner or quantifier followed by a mass noun
        let det_quant_stuff = SequenceExpr::default()
            .then(FirstMatchOf::new(vec![
                Box::new(IndefiniteArticle::default()),
                Box::new(quantifier),
            ]))
            .then_whitespace()
            .then_mass_noun_only();

        Self {
            expr: Box::new(det_quant_stuff),
        }
    }
}

impl ExprLinter for NounCountability {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let toks_chars = toks.span()?.get_content(src);

        // the determiner or quantifier
        let dq = toks[0].span.get_content_string(src).to_lowercase();

        // the mass noun
        let noun = toks[2].span.get_content_string(src).to_lowercase();

        let corrections: &'static [Correction] = match dq.as_str() {
            "a" | "an" => &[JustStuff, ReplaceWithSome, ReplaceWithAPieceOf],
            "another" => &[InsertBetweenPieceOf],
            "both" => &[InsertBetweenPiecesOf],
            "each" => &[InsertBetweenPieceOf],
            "every" => &[InsertBetweenPieceOf],
            "few" => &[ReplaceWithLittle, InsertBetweenPiecesOf],
            "many" => &[ReplaceWithMuch, ReplaceWithALotOf, InsertBetweenPiecesOf],
            "one" => &[InsertBetweenPieceOf],
            "several" => &[InsertBetweenPiecesOf],
            _ => return None,
        };

        let suggestions = corrections
            .iter()
            .map(|correction| match correction {
                JustStuff => format!("{noun}"),
                ReplaceWithSome => format!("some {noun}"),
                ReplaceWithAPieceOf => format!("a piece of {noun}"),
                InsertBetweenPieceOf => format!("{dq} piece of {noun}"),
                InsertBetweenPiecesOf => format!("{dq} pieces of {noun}"),
                ReplaceWithAll => format!("all {noun}"),
                ReplaceWithLittle => format!("little {noun}"),
                ReplaceWithMuch => format!("much {noun}"),
                ReplaceWithALotOf => format!("a lot of {noun}"),
            })
            .map(|s| Suggestion::replace_with_match_case(s.chars().collect(), toks_chars))
            .collect();

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Agreement,
            suggestions,
            message: format!("`{}` is a mass noun.", noun),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Correct mass nouns that are preceded by the wrong determiners or quantifiers."
    }
}

#[cfg(test)]
mod tests {
    use super::NounCountability;
    use crate::linting::tests::{assert_lint_count, assert_top3_suggestion_result};

    #[test]
    fn corrects_a() {
        assert_top3_suggestion_result(
            "If the unit turns out to be noisy, can I expect a firmware with phase ...",
            NounCountability::default(),
            "If the unit turns out to be noisy, can I expect some firmware with phase ...",
        );
    }

    #[test]
    #[ignore = "replace_with_match_case matches by index, not by lower vs title vs upper"]
    fn corrects_a_title_case() {
        assert_top3_suggestion_result(
            "Simple POC of a Ransomware.",
            NounCountability::default(),
            "Simple POC of a piece of Ransomware.",
        );
    }

    #[test]
    fn corrects_an() {
        assert_top3_suggestion_result(
            "The PlaySEM platform provides an infrastructure for playing and rendering sensory effects in multimedia applications.",
            NounCountability::default(),
            "The PlaySEM platform provides infrastructure for playing and rendering sensory effects in multimedia applications.",
        );
    }

    #[test]
    #[ignore = "replace_with_match_case matches by index, not by lower vs title vs upper"]
    fn corrects_an_title_case() {
        assert_top3_suggestion_result(
            "An Infrastructure for Integrated EDA.",
            NounCountability::default(),
            "Infrastructure for Integrated EDA.",
        );
    }

    #[test]
    fn corrects_another() {
        assert_top3_suggestion_result(
            "Another ransomware made by me for fun.",
            NounCountability::default(),
            "Another piece of ransomware made by me for fun.",
        );
    }

    #[test]
    fn corrects_both() {
        assert_top3_suggestion_result(
            "Make a terminal show both information of your CPU and GPU!",
            NounCountability::default(),
            "Make a terminal show both pieces of information of your CPU and GPU!",
        );
    }

    #[test]
    fn corrects_each() {
        assert_top3_suggestion_result(
            "Beside each traffic there is also a pedestrian traffic light.",
            NounCountability::default(),
            // TODO "piece of traffic" is pretty bad - what should we do?
            "Beside each piece of traffic there is also a pedestrian traffic light.",
        );
    }

    #[test]
    fn corrects_every() {
        assert_top3_suggestion_result(
            "Capacitor plugin to get access to every info about the device software and hardware.",
            NounCountability::default(),
            "Capacitor plugin to get access to every piece of info about the device software and hardware.",
        );
    }

    #[test]
    fn corrects_few() {
        assert_top3_suggestion_result(
            "Displays a few information to help you rotating through your spells.",
            NounCountability::default(),
            "Displays a few pieces of information to help you rotating through your spells.",
        );
    }

    #[test]
    fn corrects_many() {
        assert_top3_suggestion_result(
            "It shows clearly how many information about objects you can get with old search ...",
            NounCountability::default(),
            "It shows clearly how much information about objects you can get with old search ...",
        );
    }

    #[test]
    fn corrects_one() {
        assert_top3_suggestion_result(
            "For example, it only makes sense to compare global protein q-value filtering in one software with that in another.",
            NounCountability::default(),
            "For example, it only makes sense to compare global protein q-value filtering in one piece of software with that in another.",
        );
    }

    #[test]
    fn corrects_several() {
        assert_top3_suggestion_result(
            "The program takes in input a single XML file and outputs several info in different files.",
            NounCountability::default(),
            "The program takes in input a single XML file and outputs several pieces of info in different files.",
        );
    }

    #[test]
    #[ignore]
    fn dont_correct_many_compound() {
        assert_lint_count(
            "Additionally, many software development platforms also provide access to a community of developers.",
            NounCountability::default(),
            0,
        );
    }

    #[test]
    #[ignore]
    fn dont_correct_first_do_correct_second() {
        assert_top3_suggestion_result(
            "A advice description is required for each advice.",
            NounCountability::default(),
            "A advice description is required for each piece of advice.",
        );
    }
}
