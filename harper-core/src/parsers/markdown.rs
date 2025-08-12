use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use super::{Parser, PlainEnglish};
use crate::{Span, Token, TokenKind, TokenStringExt, VecExt};

/// A parser that wraps the [`PlainEnglish`] parser that allows one to parse
/// CommonMark files.
///
/// Will ignore code blocks and tables.
#[derive(Default, Clone, Debug, Copy)]
pub struct Markdown {
    options: MarkdownOptions,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarkdownOptions {
    pub ignore_link_title: bool,
}

// Clippy rule excepted because this can easily be expanded later
#[allow(clippy::derivable_impls)]
impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            ignore_link_title: false,
        }
    }
}

impl Markdown {
    pub fn new(options: MarkdownOptions) -> Self {
        Self { options }
    }

    /// Remove hidden Wikilink target text.
    ///
    /// As in the stuff to the left of the pipe operator:
    ///
    /// ```markdown
    /// [[Target text|Display Text]]
    /// ```
    fn remove_hidden_wikilink_tokens(tokens: &mut Vec<Token>) {
        let mut to_remove = VecDeque::new();

        for pipe_idx in tokens.iter_pipe_indices() {
            if pipe_idx < 2 {
                continue;
            }

            // Locate preceding `[[`
            let mut cursor = pipe_idx - 2;
            let mut open_bracket = None;

            loop {
                let Some((a, b)) = tokens.get(cursor).zip(tokens.get(cursor + 1)) else {
                    break;
                };

                if a.kind.is_newline() {
                    break;
                }

                if a.kind.is_open_square() && b.kind.is_open_square() {
                    open_bracket = Some(cursor);
                    break;
                } else if cursor == 0 {
                    break;
                } else {
                    cursor -= 1;
                }
            }

            // Locate succeeding `[[`
            cursor = pipe_idx + 1;
            let mut close_bracket = None;

            loop {
                let Some((a, b)) = tokens.get(cursor).zip(tokens.get(cursor + 1)) else {
                    break;
                };

                if a.kind.is_newline() {
                    break;
                }

                if a.kind.is_close_square() && b.kind.is_close_square() {
                    close_bracket = Some(cursor);
                    break;
                } else {
                    cursor += 1;
                }
            }

            if let Some(open_bracket_idx) = open_bracket
                && let Some(close_bracket_idx) = close_bracket
            {
                to_remove.extend(open_bracket_idx..=pipe_idx);
                to_remove.push_back(close_bracket_idx);
                to_remove.push_back(close_bracket_idx + 1);
            }
        }

        tokens.remove_indices(to_remove);
    }

    /// Remove the brackets from Wikilinks without pipe operators.
    /// For __those__ Wikilinks, see [`Self::remove_hidden_wikilink_tokens`]
    fn remove_wikilink_brackets(tokens: &mut Vec<Token>) {
        let mut to_remove = VecDeque::new();
        let mut open_brackets = None;

        let mut cursor = 0;

        loop {
            let Some((a, b)) = tokens.get(cursor).zip(tokens.get(cursor + 1)) else {
                break;
            };

            if let Some(open_brackets_idx) = open_brackets {
                if a.kind.is_newline() {
                    open_brackets = None;
                    cursor += 1;
                    continue;
                }

                if a.kind.is_close_square() && b.kind.is_close_square() {
                    to_remove.push_back(open_brackets_idx);
                    to_remove.push_back(open_brackets_idx + 1);

                    to_remove.push_back(cursor);
                    to_remove.push_back(cursor + 1);

                    open_brackets = None;
                }
            } else if a.kind.is_open_square() && b.kind.is_open_square() {
                open_brackets = Some(cursor);
            }

            cursor += 1;
        }

        tokens.remove_indices(to_remove);
    }
}

impl Parser for Markdown {
    /// This implementation is quite gross to look at, but it works.
    /// If any issues arise, it would likely help to refactor this out first.
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let english_parser = PlainEnglish;

        let source_str: String = source.iter().collect();
        let md_parser = pulldown_cmark::Parser::new_ext(
            &source_str,
            pulldown_cmark::Options::all()
                .difference(pulldown_cmark::Options::ENABLE_SMART_PUNCTUATION),
        );

        let mut tokens = Vec::new();

        let mut traversed_bytes = 0;
        let mut traversed_chars = 0;

        let mut stack = Vec::new();

        // NOTE: the range spits out __byte__ indices, not char indices.
        // This is why we keep track above.
        for (event, range) in md_parser.into_offset_iter() {
            if range.start > traversed_bytes {
                traversed_chars += source_str[traversed_bytes..range.start].chars().count();
                traversed_bytes = range.start;
            }

            match event {
                pulldown_cmark::Event::SoftBreak => {
                    tokens.push(Token {
                        span: Span::new_with_len(traversed_chars, 1),
                        kind: TokenKind::Newline(1),
                    });
                }
                pulldown_cmark::Event::HardBreak => {
                    tokens.push(Token {
                        span: Span::new_with_len(traversed_chars, 1),
                        kind: TokenKind::Newline(2),
                    });
                }
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::List(v)) => {
                    tokens.push(Token {
                        span: Span::new_with_len(traversed_chars, 0),
                        kind: TokenKind::Newline(2),
                    });
                    stack.push(pulldown_cmark::Tag::List(v));
                }
                pulldown_cmark::Event::Start(tag) => stack.push(tag),
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Paragraph)
                | pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Item)
                | pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Heading(_))
                | pulldown_cmark::Event::End(pulldown_cmark::TagEnd::CodeBlock)
                | pulldown_cmark::Event::End(pulldown_cmark::TagEnd::TableCell) => {
                    tokens.push(Token {
                        // We cannot use `traversed_chars` here, as it will still point to the
                        // first character of the `Event` at this point. Instead, we use the
                        // position of the previous token's last character. This ensures the
                        // paragraph break is placed at the end of the content, not its beginning.
                        // For more info, see: https://github.com/Automattic/harper/pull/1239.
                        span: Span::new_with_len(tokens.last().map_or(0, |last| last.span.end), 0),
                        kind: TokenKind::ParagraphBreak,
                    });
                    stack.pop();
                }
                pulldown_cmark::Event::End(_) => {
                    stack.pop();
                }
                pulldown_cmark::Event::InlineMath(code)
                | pulldown_cmark::Event::DisplayMath(code)
                | pulldown_cmark::Event::Code(code) => {
                    let chunk_len = code.chars().count();

                    tokens.push(Token {
                        span: Span::new_with_len(traversed_chars, chunk_len),
                        kind: TokenKind::Unlintable,
                    });
                }
                pulldown_cmark::Event::Text(text) => {
                    let chunk_len = text.chars().count();

                    if let Some(tag) = stack.last() {
                        use pulldown_cmark::Tag;

                        if matches!(tag, Tag::CodeBlock(..)) {
                            tokens.push(Token {
                                span: Span::new_with_len(traversed_chars, text.chars().count()),
                                kind: TokenKind::Unlintable,
                            });
                            continue;
                        }
                        if matches!(tag, Tag::Link { .. }) && self.options.ignore_link_title {
                            tokens.push(Token {
                                span: Span::new_with_len(traversed_chars, text.chars().count()),
                                kind: TokenKind::Unlintable,
                            });
                            continue;
                        }
                        if !(matches!(tag, Tag::Paragraph)
                            || matches!(tag, Tag::Link { .. }) && !self.options.ignore_link_title
                            || matches!(tag, Tag::Heading { .. })
                            || matches!(tag, Tag::Item)
                            || matches!(tag, Tag::TableCell)
                            || matches!(tag, Tag::Emphasis)
                            || matches!(tag, Tag::Strong)
                            || matches!(tag, Tag::Strikethrough))
                        {
                            continue;
                        }
                    }

                    let mut new_tokens =
                        english_parser.parse(&source[traversed_chars..traversed_chars + chunk_len]);

                    new_tokens
                        .iter_mut()
                        .for_each(|token| token.span.push_by(traversed_chars));

                    tokens.append(&mut new_tokens);
                }
                // TODO: Support via `harper-html`
                pulldown_cmark::Event::Html(_content)
                | pulldown_cmark::Event::InlineHtml(_content) => {
                    let size = _content.chars().count();
                    tokens.push(Token {
                        span: Span::new_with_len(traversed_chars, size),
                        kind: TokenKind::Unlintable,
                    });
                }
                _ => (),
            }
        }

        if matches!(
            tokens.last(),
            Some(Token {
                kind: TokenKind::Newline(_) | TokenKind::ParagraphBreak,
                ..
            })
        ) && source.last() != Some(&'\n')
        {
            tokens.pop();
        }

        Self::remove_hidden_wikilink_tokens(&mut tokens);
        Self::remove_wikilink_brackets(&mut tokens);

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::super::StrParser;
    use super::Markdown;
    use crate::{Punctuation, TokenKind, TokenStringExt, parsers::markdown::MarkdownOptions};

    #[test]
    fn survives_emojis() {
        let source = r"🤷.";

        Markdown::default().parse_str(source);
    }

    /// Check whether the Markdown parser will emit a breaking newline
    /// at the end of each input.
    ///
    /// It should _not_ do this.
    #[test]
    fn ends_with_newline() {
        let source = "This is a test.";

        let tokens = Markdown::default().parse_str(source);
        assert_ne!(tokens.len(), 0);
        assert!(!tokens.last().unwrap().kind.is_newline());
    }

    #[test]
    fn math_becomes_unlintable() {
        let source = r"$\Katex$ $\text{is}$ $\text{great}$.";

        let tokens = Markdown::default().parse_str(source);
        assert_eq!(
            tokens.iter().map(|t| t.kind.clone()).collect::<Vec<_>>(),
            vec![
                TokenKind::Unlintable,
                TokenKind::Space(1),
                TokenKind::Unlintable,
                TokenKind::Space(1),
                TokenKind::Unlintable,
                TokenKind::Punctuation(Punctuation::Period)
            ]
        )
    }

    #[test]
    fn hidden_wikilink_text() {
        let source = r"[[this is hidden|this is not]]";

        let tokens = Markdown::default().parse_str(source);

        let token_kinds = tokens.iter().map(|t| t.kind.clone()).collect::<Vec<_>>();

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
            ]
        ))
    }

    #[test]
    fn just_pipe() {
        let source = r"|";

        let tokens = Markdown::default().parse_str(source);

        let token_kinds = tokens.iter().map(|t| t.kind.clone()).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(
            token_kinds.as_slice(),
            &[TokenKind::Punctuation(Punctuation::Pipe)]
        ))
    }

    #[test]
    fn empty_wikilink_text() {
        let source = r"[[|]]";

        let tokens = Markdown::default().parse_str(source);

        let token_kinds = tokens.iter().map(|t| t.kind.clone()).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(token_kinds.as_slice(), &[]))
    }

    #[test]
    fn improper_wikilink_text() {
        let source = r"this is shown|this is also shown]]";

        let tokens = Markdown::default().parse_str(source);

        let token_kinds = tokens.iter().map(|t| t.kind.clone()).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::Pipe),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::CloseSquare),
                TokenKind::Punctuation(Punctuation::CloseSquare),
            ]
        ))
    }

    #[test]
    fn normal_wikilink() {
        let source = r"[[Wikilink]]";
        let tokens = Markdown::default().parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind.clone()).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(token_kinds.as_slice(), &[TokenKind::Word(_)]))
    }

    #[test]
    fn html_is_unlintable() {
        let source = r"The range of inputs from <ctrl-g> to ctrl-z";
        let tokens = Markdown::default().parse_str(source);
        assert_eq!(tokens.iter_unlintables().count(), 1);
    }

    #[test]
    fn link_title_unlintable() {
        let parser = Markdown::new(MarkdownOptions {
            ignore_link_title: true,
            ..MarkdownOptions::default()
        });
        let source = r"[elijah-potter/harper](https://github.com/elijah-potter/harper)";
        let tokens = parser.parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind.clone()).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(token_kinds.as_slice(), &[TokenKind::Unlintable]))
    }

    #[test]
    fn issue_194() {
        let source = r"<http://localhost:9093>";
        let parser = Markdown::new(MarkdownOptions {
            ignore_link_title: true,
            ..MarkdownOptions::default()
        });
        let token_kinds = parser
            .parse_str(source)
            .iter()
            .map(|t| t.kind.clone())
            .collect::<Vec<_>>();

        assert!(matches!(token_kinds.as_slice(), &[TokenKind::Unlintable]));
    }

    #[test]
    fn respects_link_title_config() {
        let source = r"[elijah-potter/harper](https://github.com/elijah-potter/harper)";
        let parser = Markdown::new(MarkdownOptions {
            ignore_link_title: true,
            ..MarkdownOptions::default()
        });
        let token_kinds = parser
            .parse_str(source)
            .iter()
            .map(|t| t.kind.clone())
            .collect::<Vec<_>>();

        assert!(matches!(token_kinds.as_slice(), &[TokenKind::Unlintable]));

        let parser = Markdown::new(MarkdownOptions {
            ignore_link_title: false,
            ..MarkdownOptions::default()
        });
        let token_kinds = parser
            .parse_str(source)
            .iter()
            .map(|t| t.kind.clone())
            .collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::Hyphen),
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::ForwardSlash),
                TokenKind::Word(_)
            ]
        ));
    }

    /// Test that code blocks are immediately followed by a paragraph break.
    #[test]
    fn issue_880() {
        let source = r#"
Paragraph.

```
Code block
```
Paragraph.
        "#;
        let parser = Markdown::new(MarkdownOptions::default());
        let tokens = parser.parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind.clone()).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::Punctuation(_),
                TokenKind::ParagraphBreak,
                TokenKind::Unlintable,
                TokenKind::ParagraphBreak,
                TokenKind::Word(_),
                TokenKind::Punctuation(_),
            ]
        ))
    }

    /// Helps ensure that ending tokens (like `ParagraphBreak`) don't get erroneously placed at
    /// the beginning of a sentence. This kind of behavior can cause crashes, as seen in
    /// [#1181](https://github.com/Automattic/harper/issues/1181).
    #[test]
    fn no_end_token_incorrectly_ending_at_zero() {
        let source = "Something\n";
        let parser = Markdown::new(MarkdownOptions::default());
        let tokens = parser.parse_str(source);
        assert_ne!(tokens.last().unwrap().span.end, 0);
    }
}
