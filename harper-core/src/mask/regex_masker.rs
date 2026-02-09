use regex::Regex;

use crate::Span;

use super::{Mask, Masker};

pub struct RegexMasker {
    regex: Regex,
    exclude_matches: bool,
}

impl RegexMasker {
    /// Parses and compiles the provided Regex expression. Returns None if an invalid expression
    /// was provided.
    pub fn new(regex: &str, exclude_matches: bool) -> Option<Self> {
        Some(Self {
            regex: Regex::new(regex).ok()?,
            exclude_matches,
        })
    }
}

impl Masker for RegexMasker {
    fn create_mask(&self, source: &[char]) -> Mask {
        let source_s: String = source.iter().collect();
        let mut byte_to_char = vec![0; source_s.len() + 1];

        let mut char_idx = 0;
        for (byte_idx, c) in source_s.char_indices() {
            byte_to_char[byte_idx] = char_idx;
            char_idx += 1;
            byte_to_char[byte_idx + c.len_utf8()] = char_idx;
        }

        let mut mask = Mask::new_blank();

        if self.exclude_matches {
            let mut allowed_start = 0;

            for m in self.regex.find_iter(&source_s) {
                let match_start = byte_to_char[m.start()];
                let match_end = byte_to_char[m.end()];

                if allowed_start < match_start {
                    mask.push_allowed(Span::new(allowed_start, match_start));
                }

                allowed_start = match_end;
            }

            if allowed_start < source.len() {
                mask.push_allowed(Span::new(allowed_start, source.len()));
            }
        } else {
            for m in self.regex.find_iter(&source_s) {
                let match_start = byte_to_char[m.start()];
                let match_end = byte_to_char[m.end()];

                if match_start < match_end {
                    mask.push_allowed(Span::new(match_start, match_end));
                }
            }
        }

        mask
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use super::RegexMasker;
    use crate::{Masker, Span};

    #[test]
    fn include_matches() {
        let source: Vec<_> = "foo [ignore] bar [drop]".chars().collect();
        let masker = RegexMasker::new(r"\[[^\]]+\]", false).unwrap();

        let allowed = masker
            .create_mask(&source)
            .iter_allowed(&source)
            .map(|(_, chars)| chars.iter().collect::<String>())
            .collect::<Vec<_>>();

        assert_eq!(allowed, vec!["[ignore]", "[drop]"]);
    }

    #[test]
    fn exclude_matches() {
        let source: Vec<_> = "foo [ignore] bar [drop]".chars().collect();
        let masker = RegexMasker::new(r"\[[^\]]+\]", true).unwrap();

        let allowed = masker
            .create_mask(&source)
            .iter_allowed(&source)
            .map(|(_, chars)| chars.iter().collect::<String>())
            .collect::<Vec<_>>();

        assert_eq!(allowed, vec!["foo ", " bar "]);
    }

    #[test]
    fn unicode_offsets_are_converted_to_char_spans() {
        let source: Vec<_> = "A🙂B🙂C".chars().collect();
        let masker = RegexMasker::new(r"🙂B🙂", false).unwrap();

        let allowed = masker
            .create_mask(&source)
            .iter_allowed(&source)
            .map(|(_, chars)| chars.iter().collect::<String>())
            .collect::<Vec<_>>();

        assert_eq!(allowed, vec!["🙂B🙂"]);
    }

    #[quickcheck]
    fn can_match_everything(source: String) -> TestResult {
        if source.contains(|s: char| !s.is_ascii() || s.is_control()) {
            return TestResult::discard();
        }

        let masker = RegexMasker::new(".*", false).unwrap();

        let chars: Vec<_> = source.chars().collect();
        let mask = masker.create_mask(&chars);

        if !chars.is_empty() {
            assert_eq!(mask.allowed, vec![Span::new_with_len(0, chars.len())]);
            TestResult::passed()
        } else {
            TestResult::discard()
        }
    }
}
