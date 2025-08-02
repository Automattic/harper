use hashbrown::HashSet;

use crate::{
    CharStringExt, Token, TokenStringExt,
    expr::{All, Expr, FirstMatchOf, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind},
    spell::Dictionary,
};

pub struct MassPlurals<D> {
    expr: Box<dyn Expr>,
    dict: D,
}

impl<D> MassPlurals<D>
where
    D: Dictionary,
{
    pub fn new(dict: D) -> Self {
        let oov = SequenceExpr::default().then_oov();
        let looks_plural = SequenceExpr::default().then(|tok: &Token, _src: &[char]| {
            let lchars = tok.span.get_content(_src).to_lower();
            lchars.last().is_some_and(|c| *c == 's')
        });
        let oov_looks_plural = All::new(vec![Box::new(oov), Box::new(looks_plural)]);

        let source_codes = FixedPhrase::from_phrase("source codes");

        Self {
            expr: Box::new(FirstMatchOf::new(vec![
                Box::new(oov_looks_plural),
                Box::new(source_codes),
            ])),
            dict,
        }
    }

    fn is_mass_noun_in_dictionary(&self, chars: &[char]) -> bool {
        self.dict
            .get_word_metadata(chars)
            .is_some_and(|wmd| wmd.is_mass_noun_only())
    }

    fn is_mass_noun_in_dictionary_str(&self, s: &str) -> bool {
        self.dict
            .get_word_metadata_str(s)
            .is_some_and(|wmd| wmd.is_mass_noun_only())
    }
}

impl<D> ExprLinter for MassPlurals<D>
where
    D: Dictionary,
{
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let mistake_toks = toks;

        let mut legit_words_found: HashSet<Box<[char]>> = HashSet::new();

        if mistake_toks.len() == 1 {
            let mistake_tok = &mistake_toks[0];
            // Not a fixed phrase, so it's a single word that's not in the dictionary and ends with -s
            let mut remaining_chars = mistake_tok.span.get_content(src);

            // -s
            if remaining_chars.ends_with(&['s']) {
                remaining_chars = &remaining_chars[..remaining_chars.len() - 1];

                if self.is_mass_noun_in_dictionary(remaining_chars) {
                    legit_words_found.insert(remaining_chars.into());
                }

                // -es
                if remaining_chars.ends_with(&['e']) {
                    remaining_chars = &remaining_chars[..remaining_chars.len() - 1];

                    if self.is_mass_noun_in_dictionary(remaining_chars) {
                        legit_words_found.insert(remaining_chars.into());
                    }

                    // -ies -> -y
                    if remaining_chars.ends_with(&['i']) {
                        remaining_chars = &remaining_chars[..remaining_chars.len() - 1];

                        let y_singular = format!("{}y", remaining_chars.to_string());
                        if self.is_mass_noun_in_dictionary_str(&y_singular) {
                            let y_singular_chars: Box<[char]> =
                                y_singular.chars().collect::<Vec<char>>().into_boxed_slice();
                            legit_words_found.insert(y_singular_chars.clone());
                        }
                    }
                }
            }
        } else {
            // Multiple tokens means we matched a fixed phrase
            let the_fixed_phrase = mistake_toks.span()?.get_content(src);
            // For now the only one is "source codes" and the singular is "source code"
            if the_fixed_phrase
                .to_string()
                .eq_ignore_ascii_case("source codes")
            {
                let source_code_chars: Box<[char]> = "source code"
                    .chars()
                    .collect::<Vec<char>>()
                    .into_boxed_slice();
                legit_words_found.insert(source_code_chars);
            }
        }

        if legit_words_found.is_empty() {
            return None;
        }

        let message = format!(
            "The {} `{}` is a mass noun and should not be pluralized.",
            if mistake_toks.len() == 1 {
                "word"
            } else {
                "term"
            },
            legit_words_found
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("`, `")
        );

        Some(Lint {
            span: mistake_toks.span()?,
            lint_kind: LintKind::Grammar,
            suggestions: vec![],
            message,
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Looks for plural forms of mass nouns that have no plural."
    }
}

#[cfg(test)]
mod tests {
    use crate::{linting::tests::assert_lint_count, spell::FstDictionary};

    use super::MassPlurals;

    #[test]
    fn flag_advicess() {
        assert_lint_count(
            "You gave me bad advices.",
            MassPlurals::new(FstDictionary::curated()),
            1,
        );
    }

    #[test]
    fn flag_source_codes_and_softwares() {
        assert_lint_count(
            "Do we have the source codes for these softwares?",
            MassPlurals::new(FstDictionary::curated()),
            2,
        );
    }

    #[test]
    fn flag_noun_ending_in_ies() {
        assert_lint_count(
            "Celibacies are better than sex.",
            MassPlurals::new(FstDictionary::curated()),
            1,
        );
    }
}
