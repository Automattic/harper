use crate::{
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    {CharStringExt, Lint, Token, TokenStringExt},
};

pub struct FleshOutVsFullFledged {
    expr: Box<dyn Expr>,
}

impl Default for FleshOutVsFullFledged {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::optional(SequenceExpr::word_set(&["full", "fully"]).t_ws_h())
                    .then_word_set(&[
                        "fledge", "fledged", "fledged", "fledges", "fledging", "flesh", "fleshed",
                        "fleshed", "fleshes", "fleshing", "pledge", "pledged", "pledged",
                        "pledges", "pledging",
                    ])
                    .then_optional(SequenceExpr::default().t_ws_h().t_aco("out")),
            ),
        }
    }
}

impl ExprLinter for FleshOutVsFullFledged {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // Is the first word is "full" or "fully"?
        let has_full_y = toks
            .first()
            .map(|t| {
                t.span
                    .get_content(src)
                    .eq_any_ignore_ascii_case_str(&["full", "fully"])
            })
            .unwrap_or(false);

        // Is the last word is "out"?
        let mut has_out = toks
            .last()
            .map(|t| t.span.get_content(src).eq_ignore_ascii_case_str("out"))
            .unwrap_or(false);

        // Adjust tokens to exclude "out" when it's part of a hyphenated compound
        let toks = if has_out
            && ctx
                .is_some_and(|(_, next)| next.first().map(|t| t.kind.is_hyphen()).unwrap_or(false))
        {
            has_out = false;
            &toks[..toks.len() - 2]
        } else {
            toks
        };

        // Parse the verb form (tense)

        enum Form {
            Lemma,
            Past,
            ThirdPersonSingular,
            Ing,
        }

        let vtok_idx = if has_full_y { 2 } else { 0 };
        let vtok = &toks[vtok_idx];
        let vtok_chars = vtok.span.get_content(src);

        let form = match vtok_chars.last() {
            Some('d') => Form::Past,
            Some('s') => Form::ThirdPersonSingular,
            Some('g') => Form::Ing,
            _ => Form::Lemma,
        };

        // Parse which verb

        enum Verb {
            Fledge,
            Flesh,
            Pledge,
        }

        let verb = if vtok_chars.starts_with_ignore_ascii_case_str("fledg") {
            Verb::Fledge
        } else if vtok_chars.starts_with_ignore_ascii_case_str("flesh") {
            Verb::Flesh
        } else {
            Verb::Pledge
        };

        // Separated by spaces or hyphens? Abort if it's mixed.

        let mut sep_flags = 0;
        for sep_tok in toks.iter().skip(1).step_by(2) {
            if sep_tok.kind.is_hyphen() {
                sep_flags |= 1;
            } else if sep_tok.kind.is_whitespace() {
                sep_flags |= 2;
            } else {
                sep_flags |= 4;
            }
        }
        let is_hy = match sep_flags {
            1 => true,
            2 => false,
            _ => return None,
        };

        match (has_full_y, verb, &form, has_out) {
            // full pledge(d) -> full fledged
            // full fledge -> full fledged
            (true, Verb::Pledge, Form::Lemma | Form::Past, false)
            | (true, Verb::Fledge, Form::Lemma, false) => {
                let verb_and_sep_toks = &toks[0..2];
                let verb_and_sep_span = verb_and_sep_toks.span()?;

                Some(Lint {
                    span: toks.span()?,
                    lint_kind: LintKind::Usage,
                    suggestions: vec![Suggestion::replace_with_match_case(
                        format!("{}fledged", verb_and_sep_span.get_content_string(src))
                            .chars()
                            .collect(),
                        verb_and_sep_span.get_content(src),
                    )],
                    message: "This idiom uses the word `fledged`.".to_string(),
                    ..Default::default()
                })
            }
            // fledge out -> flesh out
            (false, Verb::Fledge | Verb::Pledge, _, true) => Some(Lint {
                span: vtok.span,
                lint_kind: LintKind::Usage,
                suggestions: vec![Suggestion::replace_with_match_case_str(
                    match &form {
                        Form::Lemma => "flesh",
                        Form::Past => "fleshed",
                        Form::Ing => "fleshing",
                        Form::ThirdPersonSingular => "fleshes",
                    },
                    vtok_chars,
                )],
                message: "This idiom uses the word `flesh`.".to_string(),
                ..Default::default()
            }),
            // TODO: only with "fully" and not "full"?
            // fully fledged/pledged out -> fully fledged / fully fleshed out
            // fully fleshed             -> fully fledged / fully fleshed out
            (true, Verb::Fledge | Verb::Pledge, Form::Past, true)
            | (true, Verb::Flesh, Form::Past, false) => Some(Lint {
                span: toks[vtok_idx..].span()?,
                lint_kind: LintKind::Usage,
                suggestions: vec![
                    Suggestion::replace_with_match_case_str("fledged", vtok_chars),
                    Suggestion::replace_with_match_case(
                        format!("fleshed{}out", if is_hy { '-' } else { ' ' })
                            .chars()
                            .collect(),
                        vtok_chars,
                    ),
                ],
                message: "Perhaps you're confusing `fully fledged` and `fleshed out`?".to_string(),
                ..Default::default()
            }),
            _ => None,
        }
    }

    fn description(&self) -> &str {
        "Corrects mixing up `flesh out` and `full fledged`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{
        flesh_out_vs_full_fledged::FleshOutVsFullFledged,
        tests::{assert_good_and_bad_suggestions, assert_suggestion_result},
    };

    // FULL

    // full Vlemma

    #[test]
    fn full_fledge_hyphen() {
        assert_suggestion_result(
            "Or do we want to become a full-fledge out-of-core ml library?",
            FleshOutVsFullFledged::default(),
            "Or do we want to become a full-fledged out-of-core ml library?",
        );
    }

    // full Vpast

    #[test]
    fn full_fleshed_space() {
        assert_suggestion_result(
            "Run a full fleshed ubuntu in termux without rooting your android.",
            FleshOutVsFullFledged::default(),
            "Run a full fledged ubuntu in termux without rooting your android.",
        );
    }

    #[test]
    fn full_fleshed_webscraper_hyphen() {
        assert_suggestion_result(
            "A full-fleshed webscraper web app build on Next.js13 with tracking the prices of different product you want",
            FleshOutVsFullFledged::default(),
            "A full-fledged webscraper web app build on Next.js13 with tracking the prices of different product you want",
        );
    }

    #[test]
    fn full_fleshed_implementation_hyphen() {
        assert_suggestion_result(
            "almost provides a full-fleshed implementation allowing to read binary files into a tensor in a torchscript-compatible way.",
            FleshOutVsFullFledged::default(),
            "almost provides a full-fledged implementation allowing to read binary files into a tensor in a torchscript-compatible way.",
        );
    }

    #[test]
    fn full_pledged_space() {
        assert_suggestion_result(
            "Any plan to make full pledged php server with swoole?",
            FleshOutVsFullFledged::default(),
            "Any plan to make full fledged php server with swoole?",
        );
    }

    #[test]
    fn full_pledged_hyphen() {
        assert_suggestion_result(
            "Not yet, but I am considering the full-pledged snapshotting built-in.",
            FleshOutVsFullFledged::default(),
            "Not yet, but I am considering the full-fledged snapshotting built-in.",
        );
    }

    // FULLY

    // fully Vpast - out

    #[test]
    fn not_fully_fledged_out() {
        assert_good_and_bad_suggestions(
            "There, it's not fully fledged out yet, but it's just an idea for now.",
            FleshOutVsFullFledged::default(),
            &[
                "There, it's not fully fleshed out yet, but it's just an idea for now.",
                "There, it's not fully fledged yet, but it's just an idea for now.",
            ],
            &[],
        );
    }

    #[test]
    fn fully_fledged_out() {
        assert_good_and_bad_suggestions(
            "Is the spawning process fully fledged out yet or am I joining the party too early?",
            FleshOutVsFullFledged::default(),
            &[
                "Is the spawning process fully fleshed out yet or am I joining the party too early?",
                "Is the spawning process fully fledged yet or am I joining the party too early?",
            ],
            &[],
        );
    }

    #[test]
    fn fully_fleshed_space() {
        assert_good_and_bad_suggestions(
            "A Fully Fleshed E-Commerce web application, built with React, Redux and Firebase",
            FleshOutVsFullFledged::default(),
            &[
                "A Fully Fledged E-Commerce web application, built with React, Redux and Firebase",
                "A Fully Fleshed out E-Commerce web application, built with React, Redux and Firebase",
            ],
            &[],
        );
    }

    #[test]
    fn fully_fleshed_hyphen() {
        assert_good_and_bad_suggestions(
            "This issue tracks the current progress towards publishing a fully-fleshed Fabric version of Gallery",
            FleshOutVsFullFledged::default(),
            &[
                "This issue tracks the current progress towards publishing a fully-fleshed-out Fabric version of Gallery",
                "This issue tracks the current progress towards publishing a fully-fledged Fabric version of Gallery",
            ],
            &[],
        );
    }

    #[test]
    fn fully_pledged_space() {
        assert_suggestion_result(
            "Overall, we are already moving closer to having a fully pledged distributed recording and replay.",
            FleshOutVsFullFledged::default(),
            "Overall, we are already moving closer to having a fully fledged distributed recording and replay.",
        );
    }

    // V.LEMMA - out

    #[test]
    fn fledge_out() {
        assert_suggestion_result(
            "For this we could fledge out the rule evaluation in a library and link this library to the server.",
            FleshOutVsFullFledged::default(),
            "For this we could flesh out the rule evaluation in a library and link this library to the server.",
        );
    }

    // V.PAST - out

    #[test]
    fn fledged_out_space() {
        assert_suggestion_result(
            "We will be talking more about this when the technical details a more fledged out.",
            FleshOutVsFullFledged::default(),
            "We will be talking more about this when the technical details a more fleshed out.",
        );
    }
}
