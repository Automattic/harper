use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
    patterns::{InflectionOfBe, WordSet},
};

pub struct WithOpenArms {
    expr: All,
}

impl Default for WithOpenArms {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_of([
                Box::new(InflectionOfBe::default()) as Box<dyn Expr>,
                Box::new(WordSet::new(["get", "gets", "getting", "got", "gotten"])),
            ])
            .t_ws()
            .t_set(["greet", "greeted", "welcome", "welcomed"])
            .t_ws()
            .t_aco("with")
            .t_ws()
            .t_set(["open", "opened"])
            .t_ws()
            .t_set(["arm", "arms"])
            .but_not(
                SequenceExpr::word_set(["greeted", "welcomed"])
                    .t_any()
                    .t_any()
                    .t_any()
                    .t_aco("open")
                    .t_any()
                    .t_aco("arms"),
            ),
        }
    }
}

impl ExprLinter for WithOpenArms {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));
        let [verb, ws_with_ws @ .., open, ws4, arms] = &toks[2..] else {
            return None;
        };

        let (verb_ch, open_ch, arms_ch) = (verb.get_ch(src), open.get_ch(src), arms.get_ch(src));

        let not_passive = !verb_ch.ends_with_ignore_ascii_case_chars(&['d']);
        let open_past = open_ch.ends_with_ignore_ascii_case_chars(&['d']);
        let arms_singular = !arms_ch.ends_with_ignore_ascii_case_chars(&['s']);

        let verb: Vec<char> = if not_passive {
            if verb_ch.starts_with_ignore_ascii_case_str("g") {
                let mut greet = verb_ch.to_vec();
                let t_to_e_delta = ('t' as i32) - ('e' as i32);

                if let [.., t] = greet.as_slice() {
                    let e = ((*t as i32) - t_to_e_delta) as u8 as char;
                    let d = ((e as i32) - 1) as u8 as char;
                    greet.extend([e, d])
                }
                greet
            } else {
                let mut welcome = verb_ch.to_vec();
                if let [.., e] = welcome.as_slice() {
                    let d = ((*e as i32) - 1) as u8 as char;
                    welcome.push(d);
                }
                welcome
            }
        } else {
            verb_ch.to_vec()
        };

        let open: Vec<char> = if open_past {
            open_ch[..open_ch.len() - 2].to_vec()
        } else {
            open_ch.to_vec()
        };

        let arms: Vec<char> = if arms_singular {
            let mut arm = arms_ch.to_vec();
            if let [.., m] = arm.as_slice() {
                let m_to_s_delta = ('s' as i32) - ('m' as i32);
                let s = ((*m as i32) + m_to_s_delta) as u8 as char;
                arm.push(s);
            }
            arm
        } else {
            arms_ch.to_vec()
        };

        let correction: Vec<char> = verb
            .into_iter()
            .chain(ws_with_ws.get_ch(src)?.iter().copied())
            .chain(open)
            .chain(ws4.get_ch(src).iter().copied())
            .chain(arms)
            .collect();

        let message = if not_passive {
            "The idiom `with open arms` requires the passive mood, so the verb must be in the past tense."
        } else {
            "The correct form of the idiom is `with open arms`."
        }.to_owned();

        Some(Lint {
            span: toks[2..].span()?,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::ReplaceWith(correction)],
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects wrong variants of the idiom `welcome/greet with open arms`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::WithOpenArms;

    #[test]
    fn fix_welcome() {
        assert_suggestion_result(
            "Contributions to this project are welcome with open arms!",
            WithOpenArms::default(),
            "Contributions to this project are welcomed with open arms!",
        );
    }

    #[test]
    fn fix_opened() {
        assert_suggestion_result(
            "This PR will be welcomed with opened arms <3.",
            WithOpenArms::default(),
            "This PR will be welcomed with open arms <3.",
        );
    }

    #[test]
    fn fix_arm() {
        assert_suggestion_result(
            "there are no maxis and Vitalk is welcomed with open arm in to building on top of Bitcoin",
            WithOpenArms::default(),
            "there are no maxis and Vitalk is welcomed with open arms in to building on top of Bitcoin",
        )
    }

    #[test]
    fn fix_greet_and_arm() {
        assert_suggestion_result(
            "I'm excited to be greet with open arm and the same eagerness and affection that has always awaited me",
            WithOpenArms::default(),
            "I'm excited to be greeted with open arms and the same eagerness and affection that has always awaited me",
        );
    }
}
