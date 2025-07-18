use super::{Lint, LintKind, Linter, Suggestion};
use crate::TokenStringExt;
use crate::{Document, Token, TokenKind};

#[derive(Debug, Default)]
pub struct Spaces;

impl Linter for Spaces {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut output = Vec::new();
        let mut sentences = document.iter_sentences().peekable();

        while let Some(sentence) = sentences.next() {
            let maybe_next_sentence = sentences.peek();

            for space in sentence.iter_spaces() {
                let TokenKind::Space(count) = space.kind else {
                    panic!("The space iterator should only return spaces.")
                };

                if count > 1 {
                    output.push(Lint {
                        span: space.span,
                        lint_kind: LintKind::Formatting,
                        suggestions: vec![Suggestion::ReplaceWith(vec![' '])],
                        message: format!(
                            "There are {count} spaces where there should be only one."
                        ),
                        priority: 15,
                    })
                }
            }

            if matches!(
                sentence,
                [
                    ..,
                    Token {
                        kind: TokenKind::Word(_),
                        ..
                    },
                    Token {
                        kind: TokenKind::Space(_),
                        ..
                    },
                    Token {
                        kind: TokenKind::Punctuation(_),
                        ..
                    }
                ]
            ) && !is_file_extension(document, sentence, maybe_next_sentence)
            {
                output.push(Lint {
                    span: sentence[sentence.len() - 2..sentence.len() - 1]
                        .span()
                        .unwrap(),
                    lint_kind: LintKind::Formatting,
                    suggestions: vec![Suggestion::Remove],
                    message: "Unnecessary space at the end of the sentence.".to_string(),
                    priority: 63,
                });
            }
        }

        output
    }

    fn description(&self) -> &'static str {
        "Words should be separated by at most one space."
    }
}

fn is_file_extension(
    document: &Document,
    sentence: &[Token],
    maybe_next_sentence: Option<&&[Token]>,
) -> bool {
    if !sentence.last().unwrap().kind.is_period() {
        return false;
    }

    let next_sentence = match maybe_next_sentence {
        Some(s) => s,
        None => return false,
    };

    let first_token = match next_sentence.first() {
        Some(t) if t.kind.is_word() && t.span.len() == 3 => t,
        _ => return false,
    };

    let chars = first_token.span.get_content(document.get_source());
    chars.iter().all(|c| c.is_ascii_lowercase()) || chars.iter().all(|c| c.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::Spaces;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn detects_space_before_period() {
        let source = "There is a space at the end of this sentence .";

        assert_lint_count(source, Spaces, 1)
    }

    #[test]
    fn allows_period_without_space() {
        let source = "There isn't a space at the end of this sentence.";

        assert_lint_count(source, Spaces, 0)
    }

    #[test]
    fn doesnt_flag_file_extensions() {
        let source = "Windows users can download an .exe file.";

        assert_lint_count(source, Spaces, 0)
    }

    #[test]
    fn doesnt_flag_caps_file_extensions() {
        let source = "DOS users can download an .exe file.";

        assert_lint_count(source, Spaces, 0)
    }

    #[test]
    fn flags_non_file_extensions() {
        let source = "None of .xx or .abcd or .Xyz or .ab1 are acceptable as file extensions.";

        assert_lint_count(source, Spaces, 4)
    }

    #[test]
    fn doesnt_flag_file_extensions_inside_markdown_link() {
        let source = "Windows users can [download an .exe file](https://yt-dl.org/latest/youtube-dl.exe) and stuff.";

        assert_lint_count(source, Spaces, 0)
    }

    #[test]
    fn doesnt_flag_file_extensions_inside_markdown_but_flag_ellipsis_after_space() {
        let source = "Windows users can [download an .exe file](https://yt-dl.org/latest/youtube-dl.exe) and ...";

        assert_lint_count(source, Spaces, 1)
    }

    #[test]
    fn flags_file_extensions_after_question_mark() {
        let source = "So ?exe won't work.";

        assert_lint_count(source, Spaces, 1)
    }

    #[test]
    fn flags_file_extensions_after_exclamation_mark() {
        let source = "And !COM won't work either.";

        assert_lint_count(source, Spaces, 1)
    }

    #[test]
    fn doesnt_flag_sentence_ending_with_ellipsis() {
        let source = "Is this gonna work...";

        assert_lint_count(source, Spaces, 0)
    }
}
