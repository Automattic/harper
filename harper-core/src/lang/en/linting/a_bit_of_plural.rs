use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_index_matching, followed_by_hyphen},
    },
};

const LITTLE_ETC: &[&str] = &["little", "tiny", "small"];

pub struct ABitOfPlural {
    expr: SequenceExpr,
}

impl Default for ABitOfPlural {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["a", "an"])
                .t_ws()
                .then_optional(
                    SequenceExpr::optional(SequenceExpr::aco("very").t_ws())
                        .t_set(LITTLE_ETC)
                        .t_ws(),
                )
                .then_word_seq(&["bit", "of"])
                .t_ws()
                .then_optional(SequenceExpr::default().then_adjective().t_ws())
                .then_plural_noun(),
        }
    }
}

impl ExprLinter for ABitOfPlural {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // Avoid flagging "a bit of hands-on experience"
        if followed_by_hyphen(ctx) {
            return None;
        }

        let bit_index = find_the_only_token_index_matching(toks, src, |t, s| {
            t.get_ch(s).eq_ch(&['b', 'i', 't'])
        })?;

        // Use slice pattern matching to dynamically look backward from `bit_index`
        let mut span_start = bit_index;
        if let [.., very, _, little, _] = &toks[..bit_index]
            && little.get_ch(src).eq_any_ignore_ascii_case_str(LITTLE_ETC)
            && toks[bit_index - 1].kind.is_whitespace()
            && toks[bit_index - 3].kind.is_whitespace()
            && very.get_ch(src).eq_ch(&['v', 'e', 'r', 'y'])
        {
            span_start -= 4;
        } else if let [.., little, _] = &toks[..bit_index]
            && little.get_ch(src).eq_any_ignore_ascii_case_str(LITTLE_ETC)
            && toks[bit_index - 1].kind.is_whitespace()
        {
            span_start -= 2;
        }

        // Commented-out logic left intact to match original logic precisely
        let span_end = bit_index + 2;

        let span = toks[span_start..=span_end].span()?;
        let content = span.get_content(src);

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("couple of", content),
                Suggestion::replace_with_match_case_str("few", content),
            ],
            message: "`A bit of` can only be used with mass nouns, with plural nouns use `a couple of`, etc.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags `a bit of` used with plural nouns."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::ABitOfPlural;

    // Real-world examples from techy sites

    #[test]
    fn things() {
        assert_suggestion_result(
            "Silence the test output a bit of things we ignore",
            ABitOfPlural::default(),
            "Silence the test output a few things we ignore",
        );
    }

    #[test]
    fn general_tasks() {
        assert_suggestion_result(
            "it seems to be able to handle a bit of general tasks such as Generating text,",
            ABitOfPlural::default(),
            "it seems to be able to handle a couple of general tasks such as Generating text,",
        );
    }

    #[test]
    fn chatbots() {
        assert_suggestion_result(
            "Repository to study a bit of chatbots with artificial intelligence using python and nltk.",
            ABitOfPlural::default(),
            "Repository to study a few chatbots with artificial intelligence using python and nltk.",
        );
    }

    #[test]
    fn little_bugs() {
        assert_suggestion_result(
            "I have DTM-08 installed, and im experiencing a little bit of bugs.",
            ABitOfPlural::default(),
            "I have DTM-08 installed, and im experiencing a few bugs.",
        );
    }

    // Real-world examples from non-techy sites

    #[test]
    fn slow_days() {
        assert_suggestion_result(
            "A bit of slow days, a bit of family, and obviously… knitting.",
            ABitOfPlural::default(),
            "A few slow days, a bit of family, and obviously… knitting.",
        );
    }

    #[test]
    fn big_things() {
        assert_suggestion_result(
            "Quite a bit of big things to load and unload so need stronger helpers.",
            ABitOfPlural::default(),
            "Quite a few big things to load and unload so need stronger helpers.",
        );
    }

    #[test]
    fn little_good_things() {
        assert_suggestion_result(
            "Sometimes you get a little bit of good things here and there done",
            ABitOfPlural::default(),
            "Sometimes you get a couple of good things here and there done",
        );
    }

    #[test]
    fn little_bad_days() {
        assert_suggestion_result(
            "I live here a lot of nice days with a little bit of bad days.",
            ABitOfPlural::default(),
            "I live here a lot of nice days with a few bad days.",
        );
    }
}
