use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::Expr,
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::DictionaryToken,
    spell::Dictionary,
};

type PluralGerundPattern<D> =
    DictionaryToken<Box<dyn Fn(&Token, &[char], &D) -> bool + Send + Sync>, D>;

pub struct PluralGerunds<D: Dictionary + Clone + 'static> {
    pattern: PluralGerundPattern<D>,
    dict: D,
}

impl<D: Dictionary + Clone + 'static> PluralGerunds<D> {
    pub fn new(dict: D) -> Self {
        let dict_clone = dict.clone();
        let pattern = move |tok: &Token, src: &[char], dict: &D| {
            tok.kind
                .is_oov()
                .then(|| tok.get_ch(src))
                .filter(|word| word.ends_with_ignore_ascii_case_chars(&['i', 'n', 'g', 's']))
                .is_some_and(|word| {
                    dict.get_word_metadata(&word[..word.len() - 1])
                        .is_some_and(|md| md.is_verb_progressive_form())
                })
        };

        Self {
            pattern: PluralGerundPattern::new(Box::new(pattern), dict_clone),
            dict,
        }
    }
}

impl<D: Dictionary + Clone + 'static> ExprLinter for PluralGerunds<D> {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if toks.len() == 1 {
            let token = &toks[0];
            let plural_gerund = token.get_ch(src);
            let singular = &plural_gerund[..plural_gerund.len() - 1];

            let (seen_as, replacements) = if plural_gerund.eq_str("campings") {
                (Some("Euro-English"), &["campgrounds", "campsites"][..])
            } else if plural_gerund.eq_str("learnings") {
                (
                    Some("seen as business jargon"),
                    &["insights", "lessons", "takeaways"][..],
                )
            } else if plural_gerund.eq_str("parkings") {
                (Some("Euro-English"), &["car parks", "parking lots"][..])
            } else if plural_gerund.eq_str("smokings") {
                (Some("Euro-English"), &["tuxedos", "dinner jackets"][..])
            } else if plural_gerund.eq_str("trainings") {
                (
                    Some("seen as business jargon"),
                    &[
                        "courses",
                        "training sessions",
                        "programs",
                        "workshops",
                        "seminars",
                    ][..],
                )
            } else {
                (None, &[][..])
            };

            let suggestions = replacements
                .iter()
                .map(|r| Suggestion::replace_with_match_case(r.chars().collect(), plural_gerund))
                .collect::<Vec<_>>();

            let message = if let Some(seen_context) = seen_as {
                format!(
                    "The word “{}” is {}. Consider using a more standard term such as “{}”.",
                    singular.to_string(),
                    seen_context,
                    replacements.first().unwrap()
                )
            } else {
                format!(
                    "This might be a nonstandard use of a verbal noun. “{}” doesn't normally have a plural.",
                    singular.to_string()
                )
            };

            return Some(Lint {
                span: toks.span()?,
                lint_kind: LintKind::Nonstandard,
                suggestions,
                message,
                ..Default::default()
            });
        }
        None
    }

    fn expr(&self) -> &dyn Expr {
        &self.pattern
    }

    fn description(&self) -> &str {
        "Flags verbal nouns (gerunds) that are not used in the plural in standard English."
    }
}

#[cfg(test)]
mod tests {
    use super::PluralGerunds;
    use crate::{
        linting::tests::{assert_lint_count, assert_lint_text, assert_no_lints},
        spell::FstDictionary,
    };

    #[test]
    fn flag_parkings() {
        assert_lint_count(
            "Application tracks user location and finds the nearest parkings.",
            PluralGerunds::new(FstDictionary::curated()),
            1,
        );
    }

    #[test]
    fn dont_flag_savings() {
        assert_no_lints(
            "This dashboard to display the savings made due to the purchase of Reservations (RI) or Savings Plans (SP) or by signing the agreement with Microsoft.",
            PluralGerunds::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_killings() {
        assert_no_lints(
            "Code and data to analyze drug-related killings in the Philippines.",
            PluralGerunds::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn flag_campings_but_not_bookings() {
        assert_lint_text(
            "The front-end provides a user-friendly interface for managing bookings, customers, staff, campings, payments, emplacements and categories",
            PluralGerunds::new(FstDictionary::curated()),
            "campings",
        );
    }

    #[test]
    fn flag_learnings() {
        assert_lint_count(
            "A compilation of all my learnings from different sources.",
            PluralGerunds::new(FstDictionary::curated()),
            1,
        );
    }

    #[test]
    // Datascience Trainings and Tutorials.
    fn flag_trainings() {
        assert_lint_count(
            "Datascience Trainings and Tutorials.",
            PluralGerunds::new(FstDictionary::curated()),
            1,
        );
    }
}
