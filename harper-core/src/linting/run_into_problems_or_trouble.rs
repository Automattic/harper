use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

pub struct RunIntoProblemsOrTrouble {
    expr: SequenceExpr,
}

impl Default for RunIntoProblemsOrTrouble {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["run", "ran", "running", "runs"])
                .t_ws()
                .t_aco("into")
                .t_ws()
                .then_any_of([
                    Box::new(WordSet::new(&["problem", "troubles"])) as Box<dyn Expr>,
                    Box::new(SequenceExpr::word_seq(&["a", "trouble"])),
                ]),
        }
    }
}

impl ExprLinter for RunIntoProblemsOrTrouble {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let a = match toks.len() {
            5 => false,
            7 => true,
            _ => return None,
        };

        let nptoks = &toks[4..];
        let npspan = nptoks.span()?;
        let noun = nptoks.last()?;
        let first = noun.get_ch(src).iter().next()?;
        let last = noun.get_ch(src).iter().last()?;

        // into   problem  -> problems / a problem
        // into a trouble  -> trouble
        // into   troubles -> trouble
        let (replacements, msg): (&[&str], &str) = match (a, first, last) {
            (false, &'p' | &'P', &'m' | &'M') => (
                &["problems", "a problem"],
                "The noun `problem` is countable. Use the plural `problems` or add the article `a`.",
            ),
            (true, &'t' | &'T', &'e' | &'E') => (
                &["trouble"],
                "The noun `trouble` is uncountable. Drop the article `a`.",
            ),
            (false, &'t' | &'T', &'s' | &'S') => (
                &["trouble"],
                "The idiom is `run into trouble`. Use the singular form.",
            ),
            _ => return None,
        };

        let suggestions = replacements
            .iter()
            .map(|&replacement| {
                let correction_vec_of_char = replacement.chars().collect::<Vec<char>>();
                Suggestion::replace_with_match_case(correction_vec_of_char, npspan.get_content(src))
            })
            .collect();

        Some(Lint {
            span: npspan,
            lint_kind: LintKind::Usage,
            suggestions,
            message: msg.to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `running into` `problems` or `trouble` with wrong article, singular, or plural forms."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_suggestion_result};

    use super::RunIntoProblemsOrTrouble;

    #[test]
    fn ran_into_a_trouble() {
        assert_suggestion_result(
            "but recently I ran into a trouble that the release button is not shown",
            RunIntoProblemsOrTrouble::default(),
            "but recently I ran into trouble that the release button is not shown",
        );
    }

    #[test]
    fn ran_into_problem() {
        assert_good_and_bad_suggestions(
            "Ran into problem when trying to install the upgrade assistant",
            RunIntoProblemsOrTrouble::default(),
            &[
                "Ran into problems when trying to install the upgrade assistant",
                "Ran into a problem when trying to install the upgrade assistant",
            ],
            &[],
        );
    }

    #[test]
    fn ran_into_troubles() {
        assert_suggestion_result(
            "I ran into troubles when executing contrib/utilities/update_all_files.sh",
            RunIntoProblemsOrTrouble::default(),
            "I ran into trouble when executing contrib/utilities/update_all_files.sh",
        );
    }

    #[test]
    fn run_into_a_trouble() {
        assert_suggestion_result(
            "I'm have run into a trouble when writing a lexer with below tokens",
            RunIntoProblemsOrTrouble::default(),
            "I'm have run into trouble when writing a lexer with below tokens",
        );
    }

    #[test]
    fn run_into_problem() {
        assert_good_and_bad_suggestions(
            "Running into problem when I add it into chatbox.",
            RunIntoProblemsOrTrouble::default(),
            &[
                "Running into problems when I add it into chatbox.",
                "Running into a problem when I add it into chatbox.",
            ],
            &[],
        );
    }

    #[test]
    fn run_into_troubles() {
        assert_suggestion_result(
            "[post here if you run into troubles]",
            RunIntoProblemsOrTrouble::default(),
            "[post here if you run into trouble]",
        );
    }

    #[test]
    fn running_into_a_trouble() {
        assert_suggestion_result(
            "We are spawning built app without this argument, thus running into a trouble in v3.5.0",
            RunIntoProblemsOrTrouble::default(),
            "We are spawning built app without this argument, thus running into trouble in v3.5.0",
        );
    }

    #[test]
    fn running_into_problem() {
        assert_good_and_bad_suggestions(
            "Running into problem with Prisma when using Git submodules.",
            RunIntoProblemsOrTrouble::default(),
            &[
                "Running into problems with Prisma when using Git submodules.",
                "Running into a problem with Prisma when using Git submodules.",
            ],
            &[],
        );
    }

    #[test]
    fn running_into_troubles() {
        assert_suggestion_result(
            "Running into troubles with 0.6.2",
            RunIntoProblemsOrTrouble::default(),
            "Running into trouble with 0.6.2",
        );
    }

    #[test]
    fn runs_into_problem() {
        assert_good_and_bad_suggestions(
            "Serving FaceNet with Tensorflow Serving runs into problem with PyFunc",
            RunIntoProblemsOrTrouble::default(),
            &[
                "Serving FaceNet with Tensorflow Serving runs into problems with PyFunc",
                "Serving FaceNet with Tensorflow Serving runs into a problem with PyFunc",
            ],
            &[],
        );
    }
}
