use itertools::Itertools;

use crate::indefinite_article::{InitialSound, starts_with_vowel};
use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Dialect, Document, TokenStringExt};

#[derive(Debug)]
pub struct AnA {
    dialect: Dialect,
}

impl AnA {
    pub fn new(dialect: Dialect) -> Self {
        Self { dialect }
    }
}

impl Linter for AnA {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            for (first_idx, second_idx) in chunk.iter_word_indices().tuple_windows() {
                // [`TokenKind::Unlintable`] might have semantic meaning.
                if chunk[first_idx..second_idx].iter_unlintables().count() > 0
                    || chunk[first_idx + 1..second_idx]
                        .iter_word_like_indices()
                        .count()
                        > 0
                {
                    continue;
                }

                let first = &chunk[first_idx];
                let second = &chunk[second_idx];

                let chars_first = document.get_span_content(&first.span);
                let chars_second = document.get_span_content(&second.span);
                // Break the second word on hyphens for this lint.
                // Example: "An ML-based" is an acceptable noun phrase.
                let chars_second = chars_second
                    .split(|c| !c.is_alphanumeric())
                    .next()
                    .unwrap_or(chars_second);

                let is_a_an = match chars_first {
                    ['a'] => Some(true),
                    ['A'] => Some(true),
                    ['a', 'n'] => Some(false),
                    ['A', 'n'] => Some(false),
                    _ => None,
                };

                let Some(a_an) = is_a_an else {
                    continue;
                };

                let should_be_a_an = match starts_with_vowel(chars_second, self.dialect)
                    .expect("No empty word tokens")
                {
                    InitialSound::Vowel => false,
                    InitialSound::Consonant => true,
                    InitialSound::Either => return lints,
                };

                if a_an != should_be_a_an {
                    let replacement = match a_an {
                        true => vec!['a', 'n'],
                        false => vec!['a'],
                    };

                    lints.push(Lint {
                        span: first.span,
                        lint_kind: LintKind::Miscellaneous,
                        suggestions: vec![Suggestion::replace_with_match_case(
                            replacement,
                            chars_first,
                        )],
                        message: "Incorrect indefinite article.".to_string(),
                        priority: 31,
                    })
                }
            }
        }

        lints
    }

    fn description(&self) -> &'static str {
        "A rule that looks for incorrect indefinite articles. For example, `this is an mule` would be flagged as incorrect."
    }
}

#[cfg(test)]
mod tests {
    use super::AnA;
    use crate::Dialect;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn detects_html_as_vowel() {
        assert_lint_count("Here is a HTML document.", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn detects_llm_as_vowel() {
        assert_lint_count("Here is a LLM document.", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn detects_llm_hyphen_as_vowel() {
        assert_lint_count(
            "Here is a LLM-based system.",
            AnA::new(Dialect::American),
            1,
        );
    }

    #[test]
    fn detects_euler_as_vowel() {
        assert_lint_count("This is an Euler brick.", AnA::new(Dialect::American), 0);
        assert_lint_count(
            "The graph has an Eulerian tour.",
            AnA::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn capitalized_fourier() {
        assert_lint_count(
            "Then, perform a Fourier transform.",
            AnA::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn once_over() {
        assert_lint_count("give this a once-over.", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn issue_196() {
        assert_lint_count(
            "This is formatted as an `ext4` file system.",
            AnA::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn allows_lowercase_vowels() {
        assert_lint_count("not an error", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn allows_lowercase_consonants() {
        assert_lint_count("not a crash", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn disallows_lowercase_vowels() {
        assert_lint_count("not a error", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn disallows_lowercase_consonants() {
        assert_lint_count("not an crash", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn allows_uppercase_vowels() {
        assert_lint_count("not an Error", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn allows_uppercase_consonants() {
        assert_lint_count("not a Crash", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn disallows_uppercase_vowels() {
        assert_lint_count("not a Error", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn disallows_uppercase_consonants() {
        assert_lint_count("not an Crash", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn disallows_a_interface() {
        assert_lint_count(
            "A interface for an object that can perform linting actions.",
            AnA::new(Dialect::American),
            1,
        );
    }

    #[test]
    fn allow_issue_751() {
        assert_lint_count(
            "He got a 52% approval rating.",
            AnA::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn allow_an_mp_and_an_mp3() {
        assert_lint_count("an MP and an MP3?", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn disallow_a_mp_and_a_mp3() {
        assert_lint_count("a MP and a MP3?", AnA::new(Dialect::American), 2);
    }

    #[test]
    fn recognize_acronyms() {
        // a
        assert_lint_count("using a MAC address", AnA::new(Dialect::American), 0);
        assert_lint_count("a NASA spacecraft", AnA::new(Dialect::American), 0);
        assert_lint_count("a NAT", AnA::new(Dialect::American), 0);
        assert_lint_count("a REST API", AnA::new(Dialect::American), 0);
        assert_lint_count("a LIBERO", AnA::new(Dialect::American), 0);
        assert_lint_count("a README", AnA::new(Dialect::American), 0);
        assert_lint_count("a LAN", AnA::new(Dialect::American), 0);

        // an
        assert_lint_count("an RA message", AnA::new(Dialect::American), 0);
        assert_lint_count("an SI unit", AnA::new(Dialect::American), 0);
        assert_lint_count(
            "he is an MA of both Oxford and Cambridge",
            AnA::new(Dialect::American),
            0,
        );
        assert_lint_count(
            "in an FA Cup 6th Round match",
            AnA::new(Dialect::American),
            0,
        );
        assert_lint_count("a AM transmitter", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn dont_misrecognize_as_acronym() {
        assert_lint_count("a UPD connection", AnA::new(Dialect::American), 0);
        assert_lint_count("a UPB device", AnA::new(Dialect::American), 0);
        assert_lint_count("a UPS or power device", AnA::new(Dialect::American), 0);
        assert_lint_count("a USB 2.0 port", AnA::new(Dialect::American), 0);
        assert_lint_count("an HEVC HLS stream", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn a_udev() {
        assert_lint_count("a udev rule", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn an_mdns() {
        assert_lint_count("an mDNS tool", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn an_rflink() {
        assert_lint_count("an RFLink device", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn an_ffmpeg() {
        assert_lint_count(
            "an FFmpeg-compatible input file",
            AnA::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn a_honey() {
        assert_lint_count("a Honeywell alarm panel", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn an_onedrive() {
        assert_lint_count("a OneDrive folder", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn a_ubiquiti() {
        assert_lint_count(
            "a Ubiquiti UniFi Network application",
            AnA::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn an_honest() {
        assert_lint_count("an honest mistake", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn dont_flag_an_herb_for_american() {
        assert_lint_count("an herb", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn dont_flag_a_herb_for_british() {
        assert_lint_count("a herb", AnA::new(Dialect::British), 0);
    }

    #[test]
    fn correct_an_herb_for_australian() {
        assert_suggestion_result("an herb", AnA::new(Dialect::Australian), "a herb");
    }

    #[test]
    fn correct_a_herb_for_canadian() {
        assert_suggestion_result("a herb", AnA::new(Dialect::Canadian), "an herb");
    }

    #[test]
    fn dont_flag_a_sql() {
        assert_lint_count("a SQL query", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn dont_flag_an_sql() {
        assert_lint_count("an SQL query", AnA::new(Dialect::Australian), 0);
    }

    #[test]
    fn allow_an_and_a_for_led_2550() {
        assert_lint_count("an LED", AnA::new(Dialect::American), 0);
        assert_lint_count("a LED", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn allow_a_and_an_for_url() {
        assert_lint_count(
            "I pronounce URL as 'yoo-are-ell' so for me it's 'a URL'",
            AnA::new(Dialect::American),
            0,
        );
        assert_lint_count(
            "But some people pronounce it like 'earl' so for them it's 'an URL'",
            AnA::new(Dialect::American),
            0,
        );
    }
}
