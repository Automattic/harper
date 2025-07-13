use crate::{
    Lrc, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct RedundantAdditiveAdverbs {
    expr: Box<dyn Expr>,
}

impl Default for RedundantAdditiveAdverbs {
    fn default() -> Self {
        let also_too = WordSet::new(&["also", "too"]);
        let as_well = FixedPhrase::from_phrase("as well");

        let additive_adverb = Lrc::new(FirstMatchOf::new(vec![
            Box::new(also_too),
            Box::new(as_well),
        ]));

        let multiple_additive_adverbs = SequenceExpr::default()
            .then(additive_adverb.clone())
            .then_one_or_more(
                SequenceExpr::default()
                    .then_whitespace()
                    .then(additive_adverb.clone()),
            )
            .then_optional(SequenceExpr::default().then_whitespace().t_aco("as"));

        Self {
            expr: Box::new(multiple_additive_adverbs),
        }
    }
}

impl ExprLinter for RedundantAdditiveAdverbs {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let phrase_string = toks.span()?.get_content_string(src).to_lowercase();

        // Rule out `also too` as in `This is also too slow`.
        if phrase_string.eq("also too") {
            return None;
        }

        let mut toks = toks;

        // Check for the `as well as` false positive at the end
        if phrase_string.ends_with(" as well as") {
            // three word tokens and three whitespace tokens
            if toks.len() >= 6 {
                toks = &toks[..toks.len() - 6];
            }
        }

        let mut additive_adverbs: Vec<&[char]> = vec![];

        for word in toks
            .iter()
            .filter(|tok| tok.kind.is_word())
            .map(|tok| tok.span.get_content(src))
            .collect::<Vec<_>>()
        {
            let term: &[char] = match word {
                ['a', 's'] | ['w', 'e', 'l', 'l'] => &['a', 's', ' ', 'w', 'e', 'l', 'l'],
                _ => word,
            };
            if !additive_adverbs.contains(&term) {
                additive_adverbs.push(term);
            }
        }

        // Because of the possible `as well as` false positive, we might only have one additive adverb left.
        if additive_adverbs.len() < 2 {
            return None;
        }

        let suggestions = additive_adverbs
            .iter()
            .filter_map(|adverb| {
                Some(Suggestion::replace_with_match_case(
                    adverb.to_vec(),
                    toks.span()?.get_content(src),
                ))
            })
            .collect::<Vec<_>>();

        let message = format!(
            "Use just one of `{}`.",
            additive_adverbs
                .iter()
                .map(|s| s.iter().collect::<String>())
                .collect::<Vec<_>>()
                .join("` or `")
        );

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Redundancy,
            suggestions,
            message,
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Detects redundant additive adverbs."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{
        RedundantAdditiveAdverbs,
        tests::{assert_lint_count, assert_top3_suggestion_result},
    };

    // Basic unit tests

    #[test]
    fn flag_as_well_too() {
        assert_top3_suggestion_result(
            "Yeah, we definitely miss him on this episode here, but you could probably get him on a podcast that's more focused on what Equinix is doing as well too, specifically.",
            RedundantAdditiveAdverbs::default(),
            "Yeah, we definitely miss him on this episode here, but you could probably get him on a podcast that's more focused on what Equinix is doing as well, specifically.",
        );
    }

    #[test]
    fn flag_too_also() {
        assert_top3_suggestion_result(
            "The #1 uptime service with many servers and is easy to setup. It is free too also.",
            RedundantAdditiveAdverbs::default(),
            "The #1 uptime service with many servers and is easy to setup. It is free also.",
        );
    }

    #[test]
    fn dont_flag_also_too() {
        assert_lint_count(
            "The version update is also too slow.",
            RedundantAdditiveAdverbs::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_also_as_well_as() {
        assert_lint_count(
            "Believe there are stable packages in the readme also as well as a link to an old version of forge in the ...",
            RedundantAdditiveAdverbs::default(),
            0,
        );
    }

    #[test]
    fn do_flag_too_also_as_well_as() {
        assert_lint_count(
            "What would happen with a sentence that included too also as well as?",
            RedundantAdditiveAdverbs::default(),
            1,
        );
    }

    #[test]
    fn flag_too_as_well() {
        assert_top3_suggestion_result(
            "Module name itself was changed too as well.",
            RedundantAdditiveAdverbs::default(),
            "Module name itself was changed as well.",
        );
    }
}
