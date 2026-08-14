use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct FellowCoRedundancy {
    expr: SequenceExpr,
}

impl Default for FellowCoRedundancy {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_capitalization_of("fellow")
                .t_ws()
                .then_any_of(vec![
                    Box::new(SequenceExpr::aco("co").t_ws_h().t_set(&[
                        "admin",
                        "admins",
                        "chair",
                        "chairs",
                        "conspirator",
                        "conspirators",
                        "equal",
                        "equals",
                        "founder",
                        "founders",
                        "host",
                        "hosts",
                        "intern",
                        "interns",
                        "leader",
                        "leaders",
                        "organiser",
                        "organisers",
                        "organizer",
                        "organizers",
                        "pilot",
                        "pilots",
                        "worker",
                        "workers",
                    ])),
                    Box::new(SequenceExpr::word_set(&[
                        "coadmin",
                        "coadmins",
                        "cochair",
                        "cochairs",
                        "coconspirator",
                        "coconspirators",
                        "coequal",
                        "coequals",
                        "cofounder",
                        "cofounders",
                        "cohost",
                        "cohosts",
                        "cointern",
                        "cointerns",
                        "coorganiser",
                        "coorganisers",
                        "coorganizer",
                        "coorganizers",
                        "coleader",
                        "coleaders",
                        "copilot",
                        "copilots",
                        "coworker",
                        "coworkers",
                    ])),
                ]),
        }
    }
}

impl ExprLinter for FellowCoRedundancy {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // "co" and the whitespace or hyphen is 2 tokens, "co" is also 2 chars

        let (fellow_sep, co_worker) = toks.split_at(2);

        let (co, worker) = match co_worker.len() {
            1 => co_worker[0].get_ch(src).split_at(2),
            3 => (co_worker[0..2].get_ch(src)?, co_worker[2..].get_ch(src)?),
            _ => return None,
        };

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Redundancy,
            suggestions: [co, fellow_sep.get_ch(src)?]
                .map(|cf| {
                    Suggestion::ReplaceWith(cf.iter().chain(worker.iter()).copied().collect())
                })
                .into(),
            message: "Using `fellow` with `co` is redundant.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects redundant use of `fellow` with `co-`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_good_and_bad_suggestions;

    use super::FellowCoRedundancy;

    #[test]
    fn fix_coworker() {
        assert_good_and_bad_suggestions(
            "My fellow coworker discovered a vagueness in the salt docs at least from his perspective using this state module:",
            FellowCoRedundancy::default(),
            &[
                "My coworker discovered a vagueness in the salt docs at least from his perspective using this state module:",
                "My fellow worker discovered a vagueness in the salt docs at least from his perspective using this state module:",
            ],
            &[],
        );
    }

    #[test]
    fn fix_coworkers() {
        assert_good_and_bad_suggestions(
            "Actually, my fellow coworkers use putty, solarputty, snowflake and asbru.",
            FellowCoRedundancy::default(),
            &[
                "Actually, my coworkers use putty, solarputty, snowflake and asbru.",
                "Actually, my fellow workers use putty, solarputty, snowflake and asbru.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_co_hy_worker() {
        assert_good_and_bad_suggestions(
            "GitFit is a Git Hook that randomly selects an Exercise and Reps with an option to challenge a fellow co-worker.",
            FellowCoRedundancy::default(),
            &[
                "GitFit is a Git Hook that randomly selects an Exercise and Reps with an option to challenge a co-worker.",
                "GitFit is a Git Hook that randomly selects an Exercise and Reps with an option to challenge a fellow worker.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_co_sp_worker() {
        assert_good_and_bad_suggestions(
            "Can you please help me find a book that can inspire a fellow co worker? ",
            FellowCoRedundancy::default(),
            &[
                "Can you please help me find a book that can inspire a co worker? ",
                "Can you please help me find a book that can inspire a fellow worker? ",
            ],
            &[],
        );
    }

    #[test]
    fn fix_fellow_co_hy_workers() {
        assert_good_and_bad_suggestions(
            "Enable fellow co-workers with no command line experience to utilize python scripts.",
            FellowCoRedundancy::default(),
            &[
                "Enable co-workers with no command line experience to utilize python scripts.",
                "Enable fellow workers with no command line experience to utilize python scripts.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_co_sp_workers() {
        assert_good_and_bad_suggestions(
            "and if your interests align with 50.1% of your fellow co workers than great",
            FellowCoRedundancy::default(),
            &[
                "and if your interests align with 50.1% of your co workers than great",
                "and if your interests align with 50.1% of your fellow workers than great",
            ],
            &[],
        );
    }

    #[test]
    fn fix_co_sp_workers_all_caps() {
        assert_good_and_bad_suggestions(
            "GOOD MORNING MY FELLOW CO WORKERS.",
            FellowCoRedundancy::default(),
            &[
                "GOOD MORNING MY FELLOW WORKERS.",
                "GOOD MORNING MY CO WORKERS.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_fellow_co_sp_hosts() {
        assert_good_and_bad_suggestions(
            "sliding into the commentary booth with me are my fellow Co hosts",
            FellowCoRedundancy::default(),
            &[
                "sliding into the commentary booth with me are my Co hosts",
                "sliding into the commentary booth with me are my fellow hosts",
            ],
            &[],
        );
    }

    #[test]
    fn fix_fellow_co_hy_host() {
        assert_good_and_bad_suggestions(
            "James Martell along with wife and fellow co-host Arlene discuss WiFi in the cars",
            FellowCoRedundancy::default(),
            &[
                "James Martell along with wife and co-host Arlene discuss WiFi in the cars",
                "James Martell along with wife and fellow host Arlene discuss WiFi in the cars",
            ],
            &[],
        );
    }

    #[test]
    fn fix_fellow_co_hy_conspirators() {
        assert_good_and_bad_suggestions(
            "it is so you can talk to yourself and your fellow co-conspirators",
            FellowCoRedundancy::default(),
            &[
                "it is so you can talk to yourself and your co-conspirators",
                "it is so you can talk to yourself and your fellow conspirators",
            ],
            &[],
        );
    }
}
