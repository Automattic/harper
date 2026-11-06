use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_index_matching},
    },
};

pub struct HereAreACollection {
    expr: SequenceExpr,
}

enum Latitude {
    Strict,
    Lenient,
}

const COLLECTIVE_NOUNS: &[(&str, Latitude)] = &[
    ("array", Latitude::Lenient),
    ("bag", Latitude::Lenient),
    ("box", Latitude::Strict),
    ("bunch", Latitude::Lenient),
    ("collection", Latitude::Strict),
    ("group", Latitude::Strict),
    ("grouping", Latitude::Strict),
    ("list", Latitude::Strict),
    ("load", Latitude::Lenient),
    ("matrix", Latitude::Lenient),
    ("mound", Latitude::Lenient),
    ("pile", Latitude::Lenient),
    ("repo", Latitude::Strict),
    ("repository", Latitude::Strict),
    ("set", Latitude::Lenient),
    ("ton", Latitude::Lenient),
    ("tonne", Latitude::Lenient),
    ("vector", Latitude::Lenient),
];

impl Default for HereAreACollection {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["here", "there"])
                .t_ws()
                .t_aco("are")
                .t_ws()
                .t_set(["a", "an", "the"])
                .t_ws()
                .then_optional(SequenceExpr::default().then_adjective().t_ws())
                .t_set(COLLECTIVE_NOUNS.iter().map(|(s, _)| s).collect::<Vec<_>>())
                .t_ws()
                .t_aco("of")
                .t_ws()
                .then_optional(SequenceExpr::default().then_adjective().t_ws())
                .then_plural_noun(),
        }
    }
}

impl ExprLinter for HereAreACollection {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let are = toks.get(2)?;
        let of =
            find_the_only_token_index_matching(toks, src, |t, s| t.get_ch(s).eq_ch(&['o', 'f']))?;
        let coll = toks.get(of - 2)?;

        let mut correction = are.get_ch(src).to_vec();
        let a_to_i_delta = b'i' - b'a';
        correction[0] = (correction[0] as u8 + a_to_i_delta) as char;
        correction[1] = (correction[1] as u8 + 1) as char;
        correction.pop();

        let latitude = COLLECTIVE_NOUNS
            .iter()
            .find(|(s, _)| coll.get_ch(src).eq_str(s))
            .map(|(_, lat)| lat)?;

        let msg = match latitude {
            Latitude::Strict => {
                "This verb should be singular to agree with the collective noun container (e.g., `there is a repository`)."
            }
            Latitude::Lenient => {
                "Consider using a singular verb here. While plural verbs are common informally, formal grammar treats this collective noun as singular."
            }
        };

        Some(Lint {
            span: are.span,
            lint_kind: LintKind::Agreement,
            suggestions: vec![Suggestion::ReplaceWith(correction)],
            message: msg.to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Enforces singular verb agreement for collective noun phrases like `a list of` or `a bunch of`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::HereAreACollection;

    #[test]
    fn here_list_things() {
        assert_suggestion_result(
            "Here are a list of things DOOMFORMER can do",
            HereAreACollection::default(),
            "Here is a list of things DOOMFORMER can do",
        );
    }

    #[test]
    fn there_bunch_programs() {
        assert_suggestion_result(
            "And where I live there are a bunch of programs that provide free cell phones to homeless people.",
            HereAreACollection::default(),
            "And where I live there is a bunch of programs that provide free cell phones to homeless people.",
        );
    }

    #[test]
    fn there_whole_bunch_terms() {
        assert_suggestion_result(
            "There are a whole bunch of terms here that really don't have to be approved",
            HereAreACollection::default(),
            "There is a whole bunch of terms here that really don't have to be approved",
        );
    }

    #[test]
    fn here_list_adj() {
        assert_suggestion_result(
            "Here are a list of headless browsers that I know about",
            HereAreACollection::default(),
            "Here is a list of headless browsers that I know about",
        );
    }

    #[test]
    fn there_ton_things() {
        assert_suggestion_result(
            "There are a ton of things you can play around with making at home",
            HereAreACollection::default(),
            "There is a ton of things you can play around with making at home",
        );
    }

    #[test]
    fn there_ton_things_all_caps() {
        assert_suggestion_result(
            "Yeah, but there are a TON of things that trademark lawyers do that are counter-productive.",
            HereAreACollection::default(),
            "Yeah, but there is a TON of things that trademark lawyers do that are counter-productive.",
        );
    }

    // avoid false positives

    #[test]
    fn avoid_couple() {
        assert_no_lints(
            "there are a couple of things you can do to improve security",
            HereAreACollection::default(),
        );
    }

    #[test]
    fn avoid_lot() {
        assert_no_lints(
            "There are a lot of apps way, way worse than Spotify",
            HereAreACollection::default(),
        );
    }

    #[test]
    fn avoid_number() {
        assert_no_lints(
            "There are a number of utilities for this.",
            HereAreACollection::default(),
        );
    }
}
