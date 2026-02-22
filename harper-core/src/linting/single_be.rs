use crate::dict_word_metadata::VerbFormFlags;
use crate::linting::expr_linter::Chunk;
use crate::{
    Span, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::{DerivedFrom, InflectionOfBe},
};

fn be_forms(token: &Token) -> Option<VerbFormFlags> {
    let metadata = token
        .kind
        .as_word()
        .and_then(|metadata| metadata.as_ref())?;
    let verb_data = metadata.verb.as_ref()?;

    verb_data.verb_forms
}

fn is_past_flag(forms: VerbFormFlags) -> bool {
    forms.intersects(VerbFormFlags::PAST | VerbFormFlags::PRETERITE)
}

fn looks_like_be_contraction(token: &Token, source: &[char]) -> bool {
    let Some(_) = token.kind.as_word() else {
        return false;
    };

    if token.kind.is_possessive_nominal() && token.kind.is_proper_noun() {
        return false;
    }

    let content = token.span.get_content(source);
    let Some(apostrophe_idx) = content.iter().rposition(|c| matches!(*c, '\'' | '’')) else {
        return false;
    };
    let base_slice = &content[..apostrophe_idx];
    if token.kind.is_possessive_nominal() && token.kind.is_proper_noun() {
        return false;
    }
    if base_slice
        .first()
        .is_some_and(|c| c.is_uppercase() && token.kind.is_nominal() && !token.kind.is_pronoun())
    {
        return false;
    }
    let base: Vec<char> = base_slice.iter().map(|c| c.to_ascii_lowercase()).collect();
    if base == ['l', 'e', 't'] {
        return false;
    }
    let suffix: Vec<char> = content[apostrophe_idx + 1..]
        .iter()
        .map(|c| c.to_ascii_lowercase())
        .collect();

    matches!(suffix.as_slice(), ['s'] | ['r', 'e'] | ['m']) && apostrophe_idx > 0
}

pub struct SingleBe {
    expr: Box<dyn Expr>,
}

impl Default for SingleBe {
    fn default() -> Self {
        fn be_like_expr() -> SequenceExpr {
            SequenceExpr::any_of(vec![
                Box::new(InflectionOfBe::new()),
                Box::new(DerivedFrom::new_from_str("be")),
                Box::new(looks_like_be_contraction),
            ])
        }

        let expr = SequenceExpr::with(be_like_expr())
            .t_ws()
            .then(be_like_expr());

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for SingleBe {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let first = matched_tokens.first()?;
        let second = matched_tokens.last()?;

        if first.kind.is_possessive_nominal() && first.kind.is_proper_noun() {
            return None;
        }
        if first.kind.is_possessive_nominal()
            && first
                .span
                .get_content(source)
                .first()
                .is_some_and(|c| c.is_uppercase())
        {
            return None;
        }

        let progressive_like = |tok: &Token| {
            be_forms(tok).map_or_else(
                || false,
                |forms| {
                    forms.intersects(VerbFormFlags::PROGRESSIVE | VerbFormFlags::PAST_PARTICIPLE)
                },
            )
        };
        if progressive_like(first) || progressive_like(second) {
            return None;
        }

        let first_is_past = be_forms(first)
            .map(is_past_flag)
            .unwrap_or_else(|| first.kind.is_verb_past_form());
        let second_is_past = be_forms(second)
            .map(is_past_flag)
            .unwrap_or_else(|| second.kind.is_verb_past_form());

        let first_chars = first.span.get_content(source);
        let base_before_apostrophe = first_chars
            .iter()
            .rposition(|c| matches!(*c, '\'' | '’'))
            .map(|idx| &first_chars[..idx]);

        if let Some(base) = base_before_apostrophe {
            let base_first_upper = base.first().is_some_and(|c| c.is_uppercase());
            let base_lower: Vec<char> = base.iter().map(|c| c.to_ascii_lowercase()).collect();
            let is_common_pronoun = matches!(
                base_lower.as_slice(),
                ['i']
                    | ['w', 'e']
                    | ['t', 'h', 'e', 'y']
                    | ['y', 'o', 'u']
                    | ['h', 'e']
                    | ['s', 'h', 'e']
                    | ['i', 't']
                    | ['t', 'h', 'a', 't']
                    | ['t', 'h', 'e', 'r', 'e']
            );

            if base_first_upper && !first.kind.is_pronoun() && !is_common_pronoun {
                return None;
            }
        }

        if first_is_past && second_is_past {
            return None;
        }

        let whitespace_start = matched_tokens.get(1)?.span.start;
        let second_be_end = second.span.end;

        Some(Lint {
            span: Span::new(whitespace_start, second_be_end),
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::Remove],
            message: "Drop the repeated verb form so only one instance of `be` remains.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Removes adjacent duplicate inflections of `be`, including contracted forms followed by another `be` verb."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::SingleBe;

    #[test]
    fn removes_double_is() {
        assert_suggestion_result(
            "The server is is slow.",
            SingleBe::default(),
            "The server is slow.",
        );
    }

    #[test]
    fn removes_is_are() {
        assert_suggestion_result(
            "This is are unusual.",
            SingleBe::default(),
            "This is unusual.",
        );
    }

    #[test]
    fn removes_are_were_mismatch() {
        assert_suggestion_result(
            "They are were excited.",
            SingleBe::default(),
            "They are excited.",
        );
    }

    #[test]
    fn removes_mismatched_pair() {
        assert_suggestion_result("That is was odd.", SingleBe::default(), "That is odd.");
    }

    #[test]
    fn handles_s_contraction() {
        assert_suggestion_result(
            "The error's are gone.",
            SingleBe::default(),
            "The error's gone.",
        );
    }

    #[test]
    fn handles_re_contraction() {
        assert_suggestion_result(
            "We're are ready to ship.",
            SingleBe::default(),
            "We're ready to ship.",
        );
    }

    #[test]
    fn handles_m_contraction() {
        assert_suggestion_result("I'm am aware.", SingleBe::default(), "I'm aware.");
    }

    #[test]
    fn handles_future_repetition() {
        assert_suggestion_result(
            "That will be be an issue.",
            SingleBe::default(),
            "That will be an issue.",
        );
    }

    #[test]
    fn skips_being_chain() {
        assert_no_lints("It's been being rebuilt for months.", SingleBe::default());
    }

    #[test]
    fn allows_simple_be_statement() {
        assert_no_lints("Let's be honest.", SingleBe::default());
    }

    #[test]
    fn allows_possessive_before_are() {
        assert_no_lints(
            "Stories like Mateo's are the heart of what we do.",
            SingleBe::default(),
        );
    }

    #[test]
    fn removes_across_newline() {
        assert_suggestion_result(
            "That is\nis tricky.",
            SingleBe::default(),
            "That is tricky.",
        );
    }

    #[test]
    fn ignores_separated_forms() {
        assert_no_lints("The server is not is down.", SingleBe::default());
    }

    #[test]
    fn ignores_single_be() {
        assert_no_lints("This is ready.", SingleBe::default());
    }
}
