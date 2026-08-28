use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

/// Like `Itertools::Either`, but with three variants
enum Fork<K, S, T> {
    Kind(K),
    Sort(S),
    Type(T),
}

impl<K, S, T, Item> Iterator for Fork<K, S, T>
where
    K: Iterator<Item = Item>,
    S: Iterator<Item = Item>,
    T: Iterator<Item = Item>,
{
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Fork::Kind(k) => k.next(),
            Fork::Sort(s) => s.next(),
            Fork::Type(t) => t.next(),
        }
    }
}

const DEGREE_ADVERB_BLACKLIST: &[&str] = &["comparably", "indeed", "notably", "really", "so"];

pub struct VeryLess {
    expr: All,
}

impl Default for VeryLess {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_degree_adverb()
                .t_ws()
                .t_aco("less")
                .but_not(WordSet::new(DEGREE_ADVERB_BLACKLIST)),
        }
    }
}

impl ExprLinter for VeryLess {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.span()?;

        let [deg_tok, ws_tok, less_tok] = toks else {
            return None;
        };

        let replace_less = ["few", "little"];
        let replace_very = ["a lot", "far", "much"];
        let replace_quite = ["quite a bit", "quite a lot"];

        let corrections = if deg_tok.get_ch(src).eq_str("very") {
            Fork::Kind(replace_very.iter().map(|v| {
                v.chars()
                    .chain(ws_tok.get_ch(src).iter().copied())
                    .chain(less_tok.get_ch(src).iter().copied())
                    .collect::<Vec<char>>()
            }))
        } else if deg_tok.get_ch(src).eq_str("quite") {
            Fork::Sort(replace_quite.iter().map(|q| {
                q.chars()
                    .chain(ws_tok.get_ch(src).iter().copied())
                    .chain(less_tok.get_ch(src).iter().copied())
                    .collect::<Vec<char>>()
            }))
        } else {
            Fork::Type(replace_less.iter().map(|l| {
                deg_tok
                    .get_ch(src)
                    .iter()
                    .copied()
                    .chain(ws_tok.get_ch(src).iter().copied())
                    .chain(l.chars())
                    .collect::<Vec<char>>()
            }))
        };

        let suggestions = corrections
            .map(|c| Suggestion::replace_with_match_case(c, span.get_content(src)))
            .collect();

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "English does not use words like ‘too’ or ‘very’ to modify ‘less’. \
                • For comparisons, use ‘far less’ or ‘much less’, etc. \
                • If describing a small amoung or quantity, use ‘very little’ (furniture, information, traffic, etc.) or ‘very few’ (items, people, things, etc.)."
                .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects adverbs of degree (`too`, `very`, etc.) used with `less` mostly in the writing of native German speakers.`"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::VeryLess;

    // Old tests from the `VeryLess` Weir rule that only replaced `very`

    #[test]
    fn very_to_much() {
        assert_suggestion_result(
            "here is a simple way to do it with very less coding ... ;)",
            VeryLess::default(),
            "here is a simple way to do it with much less coding ... ;)",
        );
    }

    #[test]
    fn very_to_far() {
        assert_suggestion_result(
            "algorithm for processing large datasets with very less pre-configuration",
            VeryLess::default(),
            "algorithm for processing large datasets with far less pre-configuration",
        );
    }

    #[test]
    fn very_to_a_lot() {
        assert_suggestion_result(
            "Also the gpu memory usage is very less.",
            VeryLess::default(),
            "Also the gpu memory usage is a lot less.",
        );
    }

    // Additional tests that may replace `less` or the adverb of degree

    #[test]
    fn too_less() {
        assert_suggestion_result(
            "Too much code for too less, maybe too many requests for only 10$.. who knows?!",
            VeryLess::default(),
            "Too much code for too little, maybe too many requests for only 10$.. who knows?!",
        );
    }

    #[test]
    fn too_less_info() {
        assert_suggestion_result(
            "codex showing too less information when he is think chain.",
            VeryLess::default(),
            "codex showing too little information when he is think chain.",
        );
    }

    #[test]
    fn very_less_time() {
        assert_suggestion_result(
            "Connection also takes very less time.",
            VeryLess::default(),
            "Connection also takes a lot less time.",
        );
    }

    #[test]
    fn too_less_space() {
        assert_suggestion_result(
            "Auto-abbreviate text if there is too less space",
            VeryLess::default(),
            "Auto-abbreviate text if there is too little space",
        );
    }

    #[test]
    fn cant_fix_is_quite_less() {
        assert_suggestion_result(
            "Based on the result, the number of genes in the braker.gtf from braker3 is quite less (25943) than braker2 (48396).",
            VeryLess::default(),
            "Based on the result, the number of genes in the braker.gtf from braker3 is quite a lot less (25943) than braker2 (48396).",
        );
    }

    #[test]
    fn cant_fix_become_quite_less() {
        assert_suggestion_result(
            "But those things became quite less important compared to having a consistent formatting style across a team",
            VeryLess::default(),
            "But those things became quite a lot less important compared to having a consistent formatting style across a team",
        );
    }

    // Avoid false positives with certain adverbs of degree

    #[test]
    fn dont_flag_comparably_less() {
        assert_no_lints(
            "This is comparably less retraining, but the incorrect keypresses are the bigger issue.",
            VeryLess::default(),
        );
    }

    // Edge cases and false positives we don't or can't yet address

    #[test]
    #[ignore = "May be beyond the typical Germlish usage? Requires 'very low/small' etc."]
    fn cant_fix_very_less_ccr() {
        assert_suggestion_result(
            "[Bug]: Disabled primary button having very less color contrast ratio",
            VeryLess::default(),
            "[Bug]: Disabled primary button having very low color contrast ratio",
        );
    }

    #[test]
    #[ignore = "Too hard to parse. Does it mean the log is too small?"]
    fn cant_fix_log_too_less() {
        assert_no_lints(
            "build error log too less to find the key position",
            VeryLess::default(),
        );
    }

    #[test]
    #[ignore = "Proper fix would be something like 'pretty few items' or 'a pretty small number of items', etc."]
    fn cant_fix_pretty_less() {
        assert_suggestion_result(
            "I have pretty less number of items in x-axis",
            VeryLess::default(),
            "I have pretty few items in x-axis",
        );
    }
}
