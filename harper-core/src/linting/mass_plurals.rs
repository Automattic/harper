use hashbrown::HashSet;

use crate::{
    CharStringExt, Token, TokenStringExt,
    expr::{All, Expr, FirstMatchOf, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
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

        let phrases = FirstMatchOf::new(vec![
            Box::new(FixedPhrase::from_phrase("real estates")),
            Box::new(FixedPhrase::from_phrase("source codes")),
            Box::new(FixedPhrase::from_phrase("wear and tears")),
        ]);

        Self {
            expr: Box::new(FirstMatchOf::new(vec![
                Box::new(oov_looks_plural),
                Box::new(phrases),
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
        let invalid_plural_toks = toks;

        let mut valid_singulars: HashSet<Box<[char]>> = HashSet::new();

        if invalid_plural_toks.len() != 1 {
            // Multiple tokens means we matched a fixed phrase
            let phrase = invalid_plural_toks.span()?.get_content(src);
            valid_singulars.insert(phrase[..phrase.len() - 1].into());
        } else {
            let invalid_plural_tok = &invalid_plural_toks[0];
            // Not a fixed phrase, so it's a single word that's not in the dictionary and ends with -s
            let mut remaining_chars = invalid_plural_tok.span.get_content(src);

            // -s
            if remaining_chars.ends_with(&['s']) {
                remaining_chars = &remaining_chars[..remaining_chars.len() - 1];

                if self.is_mass_noun_in_dictionary(remaining_chars) {
                    valid_singulars.insert(remaining_chars.into());
                }

                // -es
                if remaining_chars.ends_with(&['e']) {
                    remaining_chars = &remaining_chars[..remaining_chars.len() - 1];

                    if self.is_mass_noun_in_dictionary(remaining_chars) {
                        valid_singulars.insert(remaining_chars.into());
                    }

                    // -ies -> -y
                    if remaining_chars.ends_with(&['i']) {
                        remaining_chars = &remaining_chars[..remaining_chars.len() - 1];

                        let y_singular = format!("{}y", remaining_chars.to_string());
                        if self.is_mass_noun_in_dictionary_str(&y_singular) {
                            let y_singular_chars: Box<[char]> =
                                y_singular.chars().collect::<Vec<char>>().into_boxed_slice();
                            valid_singulars.insert(y_singular_chars.clone());
                        }
                    }
                }
            }
        }

        if valid_singulars.is_empty() {
            return None;
        }

        let message = format!(
            "The {} `{}` is a mass noun and should not be pluralized.",
            if invalid_plural_toks.len() == 1 {
                "word"
            } else {
                "term"
            },
            valid_singulars
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("`, `")
        );

        let span = invalid_plural_toks.span()?;

        let suggestions: Vec<Suggestion> = valid_singulars
            .iter()
            .map(|sing| {
                Suggestion::replace_with_match_case(sing.clone().into(), span.get_content(src))
            })
            .collect();

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Looks for plural forms of mass nouns that have no plural."
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        linting::tests::{assert_lint_count, assert_suggestion_result},
        spell::FstDictionary,
    };

    use super::MassPlurals;

    #[test]
    fn flag_advicess() {
        assert_lint_count(
            "You gave me bad advices.",
            MassPlurals::new(FstDictionary::curated(crate::languages::LanguageFamily::English)),
            1,
        );
    }

    #[test]
    fn flag_source_codes_and_softwares() {
        assert_lint_count(
            "Do we have the source codes for these softwares?",
            MassPlurals::new(FstDictionary::curated(crate::languages::LanguageFamily::English)),
            2,
        );
    }

    #[test]
    fn flag_noun_ending_in_ies() {
        assert_lint_count(
            "Celibacies are better than sex.",
            MassPlurals::new(FstDictionary::curated(crate::languages::LanguageFamily::English)),
            1,
        );
    }

    #[test]
    fn flag_real_estates() {
        assert_lint_count(
            "Instead of giving any of her many luxury real estates or multi-million dollar fortune ...",
            MassPlurals::new(FstDictionary::curated(crate::languages::LanguageFamily::English)),
            1,
        );
    }

    #[test]
    fn flag_wear_and_tears() {
        assert_lint_count(
            "Transit costs were high in terms of time, finances, and vehicle wear and tears, which posed significant obstacles to international commerce",
            MassPlurals::new(FstDictionary::curated(crate::languages::LanguageFamily::English)),
            1,
        );
    }

    #[test]
    fn fix_wear_and_tears() {
        assert_suggestion_result("Transit costs were high in terms of time, finances, and vehicle wear and tears, which posed significant obstacles to international commerce", MassPlurals::new(FstDictionary::curated(crate::languages::LanguageFamily::English)), "Transit costs were high in terms of time, finances, and vehicle wear and tear, which posed significant obstacles to international commerce", crate::languages::Language::English(crate::EnglishDialect::American));
    }
}
