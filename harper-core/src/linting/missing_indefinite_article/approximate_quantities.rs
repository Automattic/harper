use super::phrase::{ArticlePhrase, suggest_article};
use crate::{
    CharStringExt, Dialect, Document, TokenStringExt,
    linting::{Lint, Linter},
};

/// Checks an approximate singular scale number, such as `nearly thousand`.
/// Ordinary noun countability does not apply to this construction.
pub(super) struct ApproximateQuantities(pub Dialect);

impl Linter for ApproximateQuantities {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let source = document.get_source();
        let mut lints = Vec::new();
        for sentence in document.iter_sentences() {
            for (index, token) in sentence.iter().enumerate().skip(2) {
                if sentence[index - 2]
                    .get_ch(source)
                    .eq_any_ignore_ascii_case_str(&[
                        "nearly",
                        "almost",
                        "approximately",
                        "roughly",
                        "about",
                    ])
                    && token.get_ch(source).eq_any_ignore_ascii_case_str(&[
                        "hundred", "thousand", "million", "billion", "trillion",
                    ])
                {
                    let phrase = ArticlePhrase {
                        first: token,
                        head: token,
                        start: index,
                        end: index,
                        determined: false,
                    };
                    if let Some(lint) = suggest_article(sentence, &phrase, source, self.0) {
                        lints.push(lint);
                    }
                }
            }
        }
        lints
    }
    fn description(&self) -> &'static str {
        "Checks missing articles in approximate quantities."
    }
}
