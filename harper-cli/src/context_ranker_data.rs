//! Gives the context ranker's data preparation Harper's parser and curated dictionary.

use std::sync::Arc;

use harper_context_ranker::{SentenceWord, Speller};
use harper_core::language_detection::is_likely_english;
use harper_core::parsers::{Parser, PlainEnglish};
use harper_core::spell::{
    Dictionary, FstDictionary, MAX_RANKED_CANDIDATES, context_word, spelling_candidates,
};
use harper_core::{CharStringExt, Dialect, TokenStringExt};

pub struct CuratedSpeller {
    dictionary: Arc<FstDictionary>,
}

impl CuratedSpeller {
    pub fn curated() -> Self {
        Self {
            dictionary: FstDictionary::curated(),
        }
    }
}

impl Speller for CuratedSpeller {
    fn sentences(&self, text: &str) -> Option<Vec<Vec<SentenceWord>>> {
        let source: Vec<char> = text.chars().collect();
        let tokens = PlainEnglish.parse(&source);

        is_likely_english(&tokens, &source, self.dictionary.as_ref()).then(|| {
            tokens
                .iter_sentences()
                .map(|sentence| {
                    sentence
                        .iter()
                        .filter_map(|token| {
                            context_word(token, &source).map(|context| SentenceWord {
                                context,
                                written: token
                                    .kind
                                    .is_word()
                                    .then(|| token.get_ch(&source).iter().collect()),
                            })
                        })
                        .collect()
                })
                .collect()
        })
    }

    fn is_word(&self, word: &[char]) -> bool {
        self.dictionary.contains_word(word)
    }

    fn candidates(&self, typo: &[char]) -> Vec<(String, i32)> {
        spelling_candidates(
            typo,
            self.dictionary.as_ref(),
            Dialect::American,
            MAX_RANKED_CANDIDATES,
        )
        .into_iter()
        .map(|(word, score)| (word.to_lower().iter().collect(), score))
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use harper_context_ranker::{CONTEXT_RADIUS, SentenceWord, Speller};
    use harper_core::Document;
    use harper_core::spell::{MAX_RANKED_CANDIDATES, context_word, surrounding_words};

    use super::CuratedSpeller;

    fn chars(word: &str) -> Vec<char> {
        word.chars().collect()
    }

    fn nearest(sentence: &[SentenceWord], position: usize) -> (Vec<String>, Vec<String>) {
        let contexts: Vec<String> = sentence.iter().map(|word| word.context.clone()).collect();
        let start = position.saturating_sub(CONTEXT_RADIUS);
        let end = (position + 1 + CONTEXT_RADIUS).min(contexts.len());

        (
            contexts[start..position].to_vec(),
            contexts[position + 1..end].to_vec(),
        )
    }

    #[test]
    fn training_reads_the_same_words_as_lint_time() {
        let text = "One, TWO three: four fibe! He has 3 dogs (and a cat) at home. \
                    Please save the fike to your desktop, then close every window \
                    that you opened during the long meeting this morning.";
        let document = Document::new_plain_english_curated(text);
        let (tokens, source) = (document.get_tokens(), document.get_source());

        let at_lint_time: Vec<_> = tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| context_word(token, source).is_some())
            .map(|(index, _)| surrounding_words(tokens, source, index, CONTEXT_RADIUS))
            .collect();
        let in_training: Vec<_> = CuratedSpeller::curated()
            .sentences(text)
            .unwrap()
            .iter()
            .flat_map(|sentence| (0..sentence.len()).map(|position| nearest(sentence, position)))
            .collect();

        assert_eq!(in_training, at_lint_time);
    }

    #[test]
    fn words_keep_their_written_form() {
        let sentences = CuratedSpeller::curated()
            .sentences("He has 3 Dogs, and they bark.")
            .unwrap();
        let words: Vec<(&str, Option<&str>)> = sentences[0]
            .iter()
            .map(|word| (word.context.as_str(), word.written.as_deref()))
            .collect();

        assert_eq!(
            words,
            [
                ("he", Some("He")),
                ("has", Some("has")),
                ("<NUM>", None),
                ("dogs", Some("Dogs")),
                ("and", Some("and")),
                ("they", Some("they")),
                ("bark", Some("bark")),
            ]
        );
    }

    #[test]
    fn text_in_another_language_has_no_sentences() {
        assert!(
            CuratedSpeller::curated()
                .sentences("Der schnelle braune Fuchs springt über den faulen Hund.")
                .is_none()
        );
    }

    #[test]
    fn candidates_are_lowercase_and_in_spelling_order() {
        let candidates = CuratedSpeller::curated().candidates(&chars("Fibe"));
        let words: Vec<&str> = candidates.iter().map(|(word, _)| word.as_str()).collect();

        assert_eq!(words[..3], ["fife", "file", "fine"]);
        assert!(words.contains(&"five"));
        assert!(words.len() <= MAX_RANKED_CANDIDATES);
        assert!(words.iter().all(|word| *word == word.to_lowercase()));
        assert!(candidates.windows(2).all(|pair| pair[0].1 <= pair[1].1));
    }

    #[test]
    fn is_word_ignores_capitalization() {
        let speller = CuratedSpeller::curated();

        assert!(speller.is_word(&chars("five")));
        assert!(speller.is_word(&chars("Five")));
        assert!(!speller.is_word(&chars("fibe")));
    }
}
