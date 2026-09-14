use crate::{
    CharStringExt, Lint, Token,
    expr::Expr,
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk, spell_check},
    spell::Dictionary,
};

const NEGATIVE_PREFIXES: &[&str] = &["anti", "de", "dis", "dys", "il", "im", "in", "non", "un"];

fn looks_negative_but_oov(token: &Token, source: &[char]) -> bool {
    token.kind.is_oov()
        && token
            .get_ch(source)
            .starts_with_any_ignore_ascii_case_str(NEGATIVE_PREFIXES)
}

pub struct WrongNegative<D: Dictionary + 'static> {
    pattern: fn(&Token, &[char]) -> bool,
    dict: D,
}

impl<D: Dictionary + 'static> WrongNegative<D> {
    pub fn new(dict: D) -> Self {
        Self {
            pattern: looks_negative_but_oov,
            dict,
        }
    }
}

impl<D: Dictionary + 'static> ExprLinter for WrongNegative<D> {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if toks.len() != 1 {
            return None;
        }
        let span = toks[0].span;
        let word = toks[0].get_ch(src);

        let new_negs: Vec<Vec<char>> = NEGATIVE_PREFIXES
            .iter()
            .filter(|&prefix| word.starts_with_ignore_ascii_case_str(prefix))
            .filter_map(|&prefix| {
                let rest = &word[prefix.len()..];
                (!rest.is_empty()).then_some((prefix, rest))
            })
            .flat_map(|(prefix, rest)| {
                NEGATIVE_PREFIXES
                    .iter()
                    .filter(move |&&new_prefix| new_prefix != prefix)
                    .filter_map(move |&new_prefix| {
                        let new_neg: Vec<char> =
                            new_prefix.chars().chain(rest.iter().copied()).collect();

                        self.dict.contains_word(&new_neg).then_some(new_neg)
                    })
            })
            .collect();

        let suggestions: Vec<Suggestion> = new_negs
            .into_iter()
            .map(|value| Suggestion::replace_with_match_case(value, span.get_content(src)))
            .collect();

        if suggestions.is_empty() {
            return None;
        }

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            message: if suggestions.len() == 1 {
                "Could this be the negative word you intended?"
            } else {
                "Could one of these be the negative word you intended?"
            }
            .to_string(),
            suggestions,
            priority: spell_check::SPELL_CHECK_PRIORITY - 1, // higher priority (lower number) than spell check
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.pattern
    }

    fn description(&self) -> &str {
        "If an unknown word looks like it might be a negative word, suggests correct words that are in the dictionary."
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Dialect,
        document::Document,
        linting::{LintGroup, Linter, spell_check, tests::assert_suggestion_result},
        remove_overlaps,
        spell::FstDictionary,
    };

    use super::WrongNegative;

    #[test]
    fn fix_insecured() {
        assert_suggestion_result(
            "Problem with insecured chart repositories",
            WrongNegative::new(FstDictionary::curated()),
            "Problem with unsecured chart repositories",
        );
    }

    #[test]
    fn fix_unsecure() {
        assert_suggestion_result(
            "Our one and only host is showing up as unsecure, but connected.",
            WrongNegative::new(FstDictionary::curated()),
            "Our one and only host is showing up as insecure, but connected.",
        );
    }

    #[test]
    fn fix_unpossible() {
        assert_suggestion_result(
            "This System makes a individual design unpossible.",
            WrongNegative::new(FstDictionary::curated()),
            "This System makes a individual design impossible.",
        );
    }

    #[test]
    fn fix_unvisible() {
        assert_suggestion_result(
            "Sure, I'm using \"deleted\" here to mean a third state more unvisible than \"yanked\"",
            WrongNegative::new(FstDictionary::curated()),
            "Sure, I'm using \"deleted\" here to mean a third state more invisible than \"yanked\"",
        );
    }

    #[test]
    fn wrong_negative_wins_over_spell_check() {
        let dict = FstDictionary::curated();
        let mut lint_group = LintGroup::empty();

        // Add both linters
        lint_group.add(
            "SpellCheck",
            spell_check::SpellCheck::new(dict.clone(), Dialect::American),
        );
        lint_group.add_chunk_expr_linter("WrongNegative", WrongNegative::new(dict));

        // Enable both linters in the config
        lint_group.config.set_rule_enabled("SpellCheck", true);
        lint_group.config.set_rule_enabled("WrongNegative", true);

        let document = Document::new_plain_english_curated("unvisible");
        let mut lints = lint_group.lint(&document);

        // Remove overlapping lints - this should keep WrongNegative due to higher priority
        remove_overlaps(&mut lints);

        // Should only get one lint from WrongNegative, not SpellCheck
        assert_eq!(lints.len(), 1);
        assert_eq!(
            lints[0].message,
            "Could this be the negative word you intended?"
        );
    }
}
