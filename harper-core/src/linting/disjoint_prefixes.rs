use crate::{
    Lint, Token, TokenKind, TokenStringExt,
    expr::{Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    spell::Dictionary,
};

pub struct DisjointPrefixes<D> {
    expr: Box<dyn Expr>,
    dict: D,
}

// Known false positives not to join to these prefixes:
const OUT_EXCEPTIONS: &[&str] = &["boxes", "facing", "live", "numbers", "playing"];
const OVER_EXCEPTIONS: &[&str] = &["all", "joy", "long", "night", "reading", "steps", "time"];
const UNDER_EXCEPTIONS: &[&str] = &["development", "mine"];
const UP_EXCEPTIONS: &[&str] = &["loading", "right", "state", "time", "trend"];

impl<D> DisjointPrefixes<D>
where
    D: Dictionary,
{
    pub fn new(dict: D) -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&[
                    // These prefixes rarely cause false positives
                    "anti", "auto", "bi", "counter", "de", "dis", "extra", "fore", "hyper", "il",
                    "im", "inter", "ir", "macro", "mal", "micro", "mid", "mini", "mis", "mono",
                    "multi", "non", "omni", "post", "pre", "pro", "re", "semi", "sub", "super",
                    "trans", "tri", "ultra", "un", "uni",
                    // "co" has one very common false positive: co-op != coop
                    "co",
                    // These prefixes are all also words in their own right, which leads to more false positives.
                    "out", "over", "under",
                    "up",
                    // These prefixes are commented out due to too many false positives
                    // or incorrect transformations:
                    // "a": a live -> alive
                    // "in": in C -> inc; in action -> inaction
                ])
                .t_ws_h()
                .then_kind_either(TokenKind::is_verb, TokenKind::is_noun)
                .then_optional_hyphen()
                .and_not(SequenceExpr::any_of(vec![
                    // No trailing hyphen. Ex: Custom patterns take precedence over built-in patterns -> overbuilt
                    Box::new(SequenceExpr::anything().t_any().t_any().then_hyphen()),
                    // Don't merge "co op" whether separated by space or hyphen.
                    Box::new(SequenceExpr::aco("co").t_any().t_set(&["op", "ops"])),
                    // Merge these if they're separated by hyphen, but not space.
                    Box::new(SequenceExpr::aco("out").t_ws().t_set(OUT_EXCEPTIONS)),
                    Box::new(SequenceExpr::aco("over").t_ws().t_set(OVER_EXCEPTIONS)),
                    Box::new(SequenceExpr::aco("under").t_ws().t_set(UNDER_EXCEPTIONS)),
                    Box::new(SequenceExpr::aco("up").t_ws().t_set(UP_EXCEPTIONS)),
                ])),
            ),
            dict,
        }
    }
}

impl<D> ExprLinter for DisjointPrefixes<D>
where
    D: Dictionary,
{
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
        let toks_span = toks.span()?;
        let (pre, _) = ctx?;

        // Cloud Native Pub-Sub System at Pinterest -> subsystem
        if pre.last().is_some_and(|p| p.kind.is_hyphen()) {
            return None;
        }

        // Avoid including text from unlintable sections between tokens
        // that could result from naively using toks.span()?.get_content_string(src)
        let original = format!(
            "{}{}{}",
            toks[0].span.get_content_string(src),
            if toks[1].kind.is_hyphen() { '-' } else { ' ' },
            toks[2].span.get_content_string(src)
        );

        // If the original form is in the dictionary, return None
        if self.dict.contains_word_str(&original) {
            return None;
        }

        let mut hyphenated = None;
        if !toks[1].kind.is_hyphen() {
            hyphenated = Some(format!(
                "{}-{}",
                toks[0].span.get_content_string(src),
                toks[2].span.get_content_string(src)
            ));
        }
        let joined = Some(format!(
            "{}{}",
            toks[0].span.get_content_string(src),
            toks[2].span.get_content_string(src)
        ));

        // Check if either joined or hyphenated form is in the dictionary
        let joined_valid = joined
            .as_ref()
            .map_or(false, |j| self.dict.contains_word_str(j));
        let hyphenated_valid = hyphenated
            .as_ref()
            .map_or(false, |h| self.dict.contains_word_str(h));

        if !joined_valid && !hyphenated_valid {
            return None;
        }

        // Joining with a hyphen when original is separated by space is more likely correct
        //   if hyphenated form is in the dictionary. So add first if verified.
        // Joining when separated by a space is more common but also has more false positives, so add them second.
        let suggestions = [(&hyphenated, hyphenated_valid), (&joined, joined_valid)]
            .into_iter()
            .filter_map(|(word, is_valid)| word.as_ref().filter(|_| is_valid))
            .collect::<Vec<_>>();

        let suggestions = suggestions
            .iter()
            .map(|s| {
                Suggestion::replace_with_match_case(s.chars().collect(), toks_span.get_content(src))
            })
            .collect();

        Some(Lint {
            span: toks_span,
            lint_kind: LintKind::Spelling,
            suggestions,
            message: "This looks like a prefix that can be joined with the rest of the word."
                .to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Looks for words with their prefixes written with a space or hyphen between instead of joined."
    }
}

#[cfg(test)]
mod tests {
    use super::DisjointPrefixes;
    use crate::{
        linting::tests::{assert_no_lints, assert_suggestion_result},
        spell::FstDictionary,
    };

    #[test]
    fn fix_hyphenated_to_joined() {
        assert_suggestion_result(
            "Download pre-built binaries or build from source.",
            DisjointPrefixes::new(FstDictionary::curated()),
            "Download prebuilt binaries or build from source.",
        );
    }

    #[test]
    fn fix_open_to_joined() {
        assert_suggestion_result(
            "Advanced Nginx configuration available for super users",
            DisjointPrefixes::new(FstDictionary::curated()),
            "Advanced Nginx configuration available for superusers",
        );
    }

    #[test]
    fn dont_join_open_co_op() {
        assert_no_lints(
            "They are cheaper at the co op.",
            DisjointPrefixes::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_join_hyphenated_co_op() {
        assert_no_lints(
            "Almost everything is cheaper at the co-op.",
            DisjointPrefixes::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn fix_open_to_hyphenated() {
        assert_suggestion_result(
            "My hobby is de extinction of the dinosaurs.",
            DisjointPrefixes::new(FstDictionary::curated()),
            "My hobby is de-extinction of the dinosaurs.",
        );
    }
}
