use crate::expr::{Expr, LongestMatchOf, SequenceExpr};
use crate::{Lrc, Token, patterns::WordSet};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;
use hashbrown::HashMap;

pub struct OpenCompounds {
    expr: Box<dyn Expr>,
    compound_to_phrase: HashMap<String, String>,
}

impl Default for OpenCompounds {
    fn default() -> Self {
        let phrases = [
            "a few",
            "a lot",
            "as well",
            "at all",
            "at least",
            "each other",
            "in case",
            "in fact",
            "in front",
            "up to",
        ];
        let mut compound_to_phrase = HashMap::new();
        for phrase in phrases {
            compound_to_phrase.insert(
                phrase
                    .split_whitespace()
                    .map(|s| s.to_lowercase())
                    .collect::<String>(),
                phrase.to_string(),
            );
        }

        let mut compound_wordset = WordSet::default();
        for compound in compound_to_phrase.keys().cloned().collect::<Vec<_>>() {
            compound_wordset.add(&compound);
        }
        let compound = Lrc::new(SequenceExpr::with(compound_wordset));

        let with_prev = SequenceExpr::anything().then(compound.clone());

        let with_next = SequenceExpr::with(compound.clone()).then_anything();

        let with_prev_and_next = SequenceExpr::anything()
            .then(compound.clone())
            .then_anything();

        Self {
            expr: Box::new(LongestMatchOf::new(vec![
                Box::new(with_prev_and_next),
                Box::new(with_prev),
                Box::new(with_next),
                Box::new(compound),
            ])),
            compound_to_phrase,
        }
    }
}

impl ExprLinter for OpenCompounds {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_toks: &[Token], source_chars: &[char]) -> Option<Lint> {
        // Because we don't have anything like regex captures we need to find which token matched which compound
        let index = self
            .compound_to_phrase
            .keys()
            .find_map(|compound| get_compound_idx(matched_toks, source_chars, compound))?;

        let span = matched_toks[index].span;
        let compound_string = span.get_content(source_chars).iter().collect::<String>();

        // Ignore if there's a hyphen immediately on either side
        if (0..matched_toks.len())
            .filter(|&i| i != index)
            .any(|i| matched_toks[i].kind.is_hyphen())
        {
            return None;
        }

        // Ignore trademarks etc. like InFront, inFront
        let phrase = self
            .compound_to_phrase
            .get(&compound_string.to_lowercase())?;
        if compound_string
            .chars()
            .nth(phrase.find(' ')?)?
            .is_uppercase()
        {
            return None;
        }

        Some(Lint {
            span,
            lint_kind: LintKind::BoundaryError,
            suggestions: vec![Suggestion::replace_with_match_case(
                phrase.chars().collect(),
                span.get_content(source_chars),
            )],
            message: format!("`{phrase}` should be written as two words."),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Corrects compound words that should be written as two words."
    }
}

fn get_compound_idx(toks: &[Token], src: &[char], compound: &str) -> Option<usize> {
    let len = compound.len();
    let tok_count = toks.len();

    match tok_count {
        1 => Some(0),
        3 => Some(1),
        2 => {
            let [tok0, tok1] = toks else { return None };
            let [len0, len1] = [tok0.span.len(), tok1.span.len()];

            if len0 == len && len1 != len {
                Some(0)
            } else if len1 == len && len0 != len {
                Some(1)
            } else if tok0.kind.is_word() && !tok1.kind.is_word() {
                Some(0)
            } else if !tok0.kind.is_word() && tok1.kind.is_word() {
                Some(1)
            } else {
                Some(
                    !tok0
                        .span
                        .get_content(src)
                        .iter()
                        .collect::<String>()
                        .eq_ignore_ascii_case(compound) as usize,
                )
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::OpenCompounds;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    // In front

    #[test]
    fn corrects_lone_infront() {
        assert_suggestion_result(
            "Button always overlaps (infront) of other views.",
            OpenCompounds::default(),
            "Button always overlaps (in front) of other views.",
        );
    }

    #[test]
    fn corrects_infront() {
        assert_suggestion_result(
            "So if i have no variable or a running process id/name which indicates that liveley is infront/fullscreen i can't do anything further via batch and must wait ...",
            OpenCompounds::default(),
            "So if i have no variable or a running process id/name which indicates that liveley is in front/fullscreen i can't do anything further via batch and must wait ...",
        );
    }

    #[test]
    fn ignores_pascalcase() {
        assert_lint_count(
            "InFront Labs, LLC has 16 repositories available. Follow their code on GitHub.",
            OpenCompounds::default(),
            0,
        );
    }

    #[test]
    fn ignores_camelcase() {
        assert_lint_count(
            "Click the \"toggle\" button to see how wrapping changes when an inFront is added to a letter in a word.",
            OpenCompounds::default(),
            0,
        );
    }

    #[test]
    fn correct_with_period_after() {
        assert_suggestion_result(
            "Car with a reversed ramp infront.",
            OpenCompounds::default(),
            "Car with a reversed ramp in front.",
        );
    }

    #[test]
    fn ignore_hyphen_before() {
        assert_lint_count("-infront", OpenCompounds::default(), 0);
    }

    #[test]
    fn ignore_hyphen_after() {
        assert_lint_count("infront-", OpenCompounds::default(), 0);
    }

    #[test]
    fn ignores_with_hyphen_before() {
        assert_lint_count(
            "Instantly share code, notes, and snippets. @yossi-infront",
            OpenCompounds::default(),
            0,
        );
    }

    #[test]
    fn ignores_with_hyphen_after() {
        assert_lint_count(
            "infront-cycle.ipe · infront-cycle.ipe · infront-cycle.svg · infront-cycle.svg · infront-s1s2.ipe · infront-s1s2.ipe · infront-s1s2.svg · infront-s1s2.svg.",
            OpenCompounds::default(),
            0,
        );
    }

    #[test]
    fn even_repeated_infront_works() {
        assert_suggestion_result(
            "infront infront",
            OpenCompounds::default(),
            "in front in front",
        );
    }

    // A few

    #[test]
    fn correct_afew_atomic() {
        assert_suggestion_result(
            "ITK code to generate anisotropic metrics, mostly Riemannian metrics and afew particular cases of Finslerian metrics.",
            OpenCompounds::default(),
            "ITK code to generate anisotropic metrics, mostly Riemannian metrics and a few particular cases of Finslerian metrics.",
        );
    }

    // A lot

    #[test]
    fn correct_alot_atomic() {
        assert_suggestion_result("Alot", OpenCompounds::default(), "A lot");
    }

    // As well

    #[test]
    fn correct_aswell_atomic() {
        assert_suggestion_result("Aswell", OpenCompounds::default(), "As well");
    }

    #[test]
    fn corrects_as_keyboards_aswell() {
        assert_suggestion_result(
            "Tool to read physical joystick devices, keyboards aswell, and create virtual joystick devices and output keyboard presses on a Linux system.",
            OpenCompounds::default(),
            "Tool to read physical joystick devices, keyboards as well, and create virtual joystick devices and output keyboard presses on a Linux system.",
        );
    }

    #[test]
    fn corrects_aswell_as() {
        assert_suggestion_result(
            "When UseAcrylic is true in Focused aswell as Unfocused Apearance , changing enableUnfocusedAcrylic at runtime doesn't work",
            OpenCompounds::default(),
            "When UseAcrylic is true in Focused as well as Unfocused Apearance , changing enableUnfocusedAcrylic at runtime doesn't work",
        );
    }

    #[test]
    fn corrects_toml_aswell() {
        assert_suggestion_result(
            "format Cargo.toml aswell #5893 - rust-lang/rustfmt",
            OpenCompounds::default(),
            "format Cargo.toml as well #5893 - rust-lang/rustfmt",
        );
    }

    #[test]
    fn correct_aswell() {
        assert_suggestion_result(
            "'wejoy' is a tool to read physical joystick devices, aswell as keyboards, create virtual joystick devices and output keyboard presses on a Linux system.",
            OpenCompounds::default(),
            "'wejoy' is a tool to read physical joystick devices, as well as keyboards, create virtual joystick devices and output keyboard presses on a Linux system.",
        );
    }

    // At all

    #[test]
    fn correct_atall() {
        assert_suggestion_result(
            "claude code with vs code extension not working atall",
            OpenCompounds::default(),
            "claude code with vs code extension not working at all",
        );
    }

    // At least

    #[test]
    fn correct_atleast_atomic() {
        assert_suggestion_result("Atleast", OpenCompounds::default(), "At least");
    }

    #[test]
    fn ignore_atleast_pascalcase() {
        assert_lint_count(
            "I want to understand if we are using AtLeast correctly.",
            OpenCompounds::default(),
            0,
        );
    }

    #[test]
    fn ignore_atleast_camelcase() {
        assert_lint_count(
            "verfiy with atLeast = 0 should pass even if the mocked function is never called.",
            OpenCompounds::default(),
            0,
        );
    }

    #[test]
    fn correct_atleast() {
        assert_suggestion_result(
            "Mar 22, 2562 BE — constructor - expected atleast one input #250.",
            OpenCompounds::default(),
            "Mar 22, 2562 BE — constructor - expected at least one input #250.",
        );
    }

    // Each other

    #[test]
    fn correct_eachother_atomic() {
        assert_suggestion_result("Eachother", OpenCompounds::default(), "Each other");
    }

    #[test]
    fn correct_eachother() {
        assert_suggestion_result(
            "Script parsing fails when two scenes reference eachother",
            OpenCompounds::default(),
            "Script parsing fails when two scenes reference each other",
        );
    }

    // In case

    #[test]
    fn correct_incase_atomic() {
        assert_suggestion_result("Incase", OpenCompounds::default(), "In case");
    }

    #[test]
    fn correct_in_case() {
        assert_suggestion_result(
            "Support for enum variable incase of reusable enum class",
            OpenCompounds::default(),
            "Support for enum variable in case of reusable enum class",
        );
    }

    #[test]
    fn ignore_incase_pascalcase() {
        assert_lint_count(
            "InCase save your secrets for a friend, so they can use in case it in case you went \"missing\".",
            OpenCompounds::default(),
            0,
        );
    }

    // In fact

    #[test]
    fn correct_infact_atomic() {
        assert_suggestion_result(
            "Yes I do infact exist :O",
            OpenCompounds::default(),
            "Yes I do in fact exist :O",
        );
    }

    // up to

    #[test]
    fn correct_upto() {
        assert_suggestion_result(
            "Free for upto 10k subscribers, unlimited push notifications, in-browser messaging",
            OpenCompounds::default(),
            "Free for up to 10k subscribers, unlimited push notifications, in-browser messaging",
        );
    }
}
