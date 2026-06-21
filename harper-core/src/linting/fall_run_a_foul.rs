use crate::{
    CharStringExt, Lint,
    linting::{LintKind, Linter, Suggestion},
    rig::{Alternation, Atom, CaptureGroup, Concat, Quantifier, RegexNode, RigLinter, RigMatch},
};

pub struct FallRunAFoul {
    rig: Box<dyn RegexNode>,
}

impl Linter for FallRunAFoul {
    fn lint(&mut self, document: &crate::Document) -> Vec<crate::Lint> {
        crate::rig::rig_linter::run_rig_linter(self, document)
    }

    fn description(&self) -> &str {
        crate::rig::RigLinter::description(self)
    }
}

impl FallRunAFoul {
    pub fn new() -> Self {
        Self {
            rig: Box::new(Concat::new(vec![
                Box::new(Quantifier::optional(Box::new(Concat::new(vec![
                    Box::new(Alternation::new(vec![
                        Box::new(Atom::word("fall")),
                        Box::new(Atom::word("fell")),
                        Box::new(Atom::word("falls")),
                        Box::new(Atom::word("falling")),
                        Box::new(Atom::word("run")),
                        Box::new(Atom::word("ran")),
                        Box::new(Atom::word("runs")),
                        Box::new(Atom::word("running")),
                    ])),
                    Box::new(Atom::whitespace()),
                ])))),
                Box::new(Concat::new(vec![
                    Box::new(CaptureGroup::new(
                        0,
                        Box::new(Alternation::new(vec![
                            Box::new(Atom::word("fowl")),
                            Box::new(Atom::word("afowl")),
                            Box::new(Concat::new(vec![
                                Box::new(Atom::word("a")),
                                Box::new(Atom::whitespace()),
                                Box::new(Alternation::new(vec![
                                    Box::new(Atom::word("fowl")),
                                    Box::new(Atom::word("foul")),
                                ])),
                            ])),
                        ])),
                    )),
                    Box::new(Atom::whitespace()),
                    Box::new(Atom::word("of")),
                ])),
            ])),
        }
    }
}

impl RigLinter for FallRunAFoul {
    fn match_to_lint(&self, rig_match: &RigMatch) -> Option<Lint> {
        let span = rig_match.captures.get(&0)?.to_char_span(rig_match.tokens);
        const AFOUL: &str = "afoul";
        let tmp = span.get_content(rig_match.source);
        let i: usize = (!tmp.starts_with_ignore_ascii_case_str("a")).into();

        Some(Lint {
            span,
            lint_kind: LintKind::Eggcorn,
            suggestions: vec![Suggestion::replace_with_match_case_str(&AFOUL[i..], tmp)],
            message: "Did you mean `foul` rather than `fowl`?".to_owned(),
            ..Default::default()
        })
    }

    fn pattern(&self) -> &dyn RegexNode {
        self.rig.as_ref()
    }

    fn description(&self) -> &str {
        "Checks for the eggcorn `fowl` in expressions like `run afoul of` and `fall foul of`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::FallRunAFoul;

    #[test]
    fn fall_a_foul_of() {
        assert_suggestion_result(
            "so shouldn't fall a foul of kernel limitations",
            FallRunAFoul::new(),
            "so shouldn't fall afoul of kernel limitations",
        );
    }

    #[test]
    fn fall_fowl_of() {
        assert_suggestion_result(
            "I share the concern about the shell execute feature. If used for state changes or IO then there are a lot of things developers can fall fowl of.",
            FallRunAFoul::new(),
            "I share the concern about the shell execute feature. If used for state changes or IO then there are a lot of things developers can fall foul of.",
        );
    }

    #[test]
    fn falling_fowl_of() {
        assert_suggestion_result(
            "I suspect you are falling fowl of the way PPPoE works",
            FallRunAFoul::new(),
            "I suspect you are falling foul of the way PPPoE works",
        );
    }

    #[test]
    fn falls_afowl_of() {
        assert_suggestion_result(
            "I tried actually resolving the reader and checking its class, but that again falls afowl of the aforementioned errors",
            FallRunAFoul::new(),
            "I tried actually resolving the reader and checking its class, but that again falls afoul of the aforementioned errors",
        );
    }

    #[test]
    fn falls_fowl_of() {
        assert_suggestion_result(
            "these are actually exposed and used by applications - as it simply falls fowl of the \"technologies with support\" bullet",
            FallRunAFoul::new(),
            "these are actually exposed and used by applications - as it simply falls foul of the \"technologies with support\" bullet",
        );
    }

    #[test]
    fn fell_afowl_of() {
        assert_suggestion_result(
            "when I tried to delete the trailing pipe, I fell afowl of MD055",
            FallRunAFoul::new(),
            "when I tried to delete the trailing pipe, I fell afoul of MD055",
        );
    }

    #[test]
    fn fell_fowl_of() {
        assert_suggestion_result(
            "I fell fowl of that unfortunately, but all good now.",
            FallRunAFoul::new(),
            "I fell foul of that unfortunately, but all good now.",
        );
    }

    #[test]
    fn run_a_fowl_of() {
        assert_suggestion_result(
            "Let's not run a fowl of existing data projects by squatting on their namespaces.",
            FallRunAFoul::new(),
            "Let's not run afoul of existing data projects by squatting on their namespaces.",
        );
    }

    #[test]
    fn run_afowl_of() {
        assert_suggestion_result(
            "served from an http origin to not run afowl of mixed content checks",
            FallRunAFoul::new(),
            "served from an http origin to not run afoul of mixed content checks",
        );
    }

    #[test]
    fn running_a_foul_of() {
        assert_suggestion_result(
            "and therefore not running a foul of the issue",
            FallRunAFoul::new(),
            "and therefore not running afoul of the issue",
        );
    }

    #[test]
    fn running_a_fowl_of() {
        assert_suggestion_result(
            "We're running a fowl of variance restrictions (and correctness) here.",
            FallRunAFoul::new(),
            "We're running afoul of variance restrictions (and correctness) here.",
        );
    }

    #[test]
    fn running_afowl_of() {
        assert_suggestion_result(
            "(usually) avoiding running afowl of any timeout on builds imposed by compiler explorer",
            FallRunAFoul::new(),
            "(usually) avoiding running afoul of any timeout on builds imposed by compiler explorer",
        );
    }

    #[test]
    fn runs_a_foul_of() {
        assert_suggestion_result(
            "argument that anti-discrimination law runs a foul of",
            FallRunAFoul::new(),
            "argument that anti-discrimination law runs afoul of",
        );
    }

    #[test]
    fn runs_afowl_of() {
        assert_suggestion_result(
            "which unfortunately runs afowl of pyojbus",
            FallRunAFoul::new(),
            "which unfortunately runs afoul of pyojbus",
        );
    }
}
