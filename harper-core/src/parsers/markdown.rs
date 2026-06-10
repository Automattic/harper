use std::collections::VecDeque;

use markdown::mdast::Node;
use markdown::{Constructs, MdxSignal, ParseOptions};
use serde::{Deserialize, Serialize};

use super::{Parser, PlainEnglish};
use crate::{Span, Token, TokenKind, TokenStringExt, VecExt, offsets::build_byte_to_char_map};

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
    /// Whether to parse the input as [MDX](https://mdxjs.com).
    #[serde(default)]
    pub mdx: bool,
}

impl MarkdownOptions {
    /// Returns a copy of these options with MDX parsing enabled or disabled.
    pub fn with_mdx(mut self, mdx: bool) -> Self {
        self.mdx = mdx;
        self
    }
}

// Clippy rule excepted because this can easily be expanded later
#[allow(clippy::derivable_impls)]
impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            ignore_link_title: false,
            mdx: false,
        }
    }
}

/// Build the [`ParseOptions`] used to parse ordinary (non-MDX) Markdown.
fn markdown_parse_options() -> ParseOptions {
    ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            math_flow: true,
            math_text: true,
            ..Constructs::gfm()
        },
        ..ParseOptions::default()
    }
}

/// Build the [`ParseOptions`] used to parse MDX.
fn mdx_parse_options() -> ParseOptions {
    let gfm = Constructs::gfm();

    ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            math_flow: true,
            math_text: true,
            gfm_autolink_literal: gfm.gfm_autolink_literal,
            gfm_footnote_definition: gfm.gfm_footnote_definition,
            gfm_label_start_footnote: gfm.gfm_label_start_footnote,
            gfm_strikethrough: gfm.gfm_strikethrough,
            gfm_table: gfm.gfm_table,
            gfm_task_list_item: gfm.gfm_task_list_item,
            ..Constructs::mdx()
        },
        // Accept any ESM block without validating the embedded JavaScript.
        // We only need to know its extent so we can mark it unlintable.
        mdx_esm_parse: Some(Box::new(|_value| MdxSignal::Ok)),
        ..ParseOptions::default()
    }
}

/// Determine whether the raw source of an inline math node looks like it was intended to be math.
fn is_plausible_inline_math(raw: &[char]) -> bool {
    // Only single-dollar delimiters are ambiguous with currency.
    if raw.len() < 3 || raw.first() != Some(&'$') || raw.get(1) == Some(&'$') {
        return true;
    }

    let inner = &raw[1..raw.len() - 1];

    let Some((first, last)) = inner.first().zip(inner.last()) else {
        return false;
    };

    if first.is_whitespace() || last.is_whitespace() {
        return false;
    }

    // An odd number of trailing backslashes means the closing dollar sign
    // was escaped.
    inner.iter().rev().take_while(|c| **c == '\\').count() % 2 == 0
}

/// Converts a Markdown [`Node`] tree (mdast) into Harper [`Token`]s.
struct MdastTokenizer<'a> {
    source: &'a [char],
    byte_to_char: Vec<usize>,
    options: MarkdownOptions,
    tokens: Vec<Token>,
    /// How many inline contexts (paragraphs, headings, table cells) we are currently nested inside of.
    inline_depth: usize,
}

impl MdastTokenizer<'_> {
    /// Get the char-based span of a node, if it has position information.
    fn span_of(&self, node: &Node) -> Option<Span<char>> {
        let pos = node.position()?;
        Some(Span::new(
            self.byte_to_char[pos.start.offset],
            self.byte_to_char[pos.end.offset],
        ))
    }

    /// Append a `ParagraphBreak` anchored at the end of the previous token.
    fn push_paragraph_break(&mut self) {
        self.tokens.push(Token {
            span: Span::empty(self.tokens.last().map_or(0, |last| last.span.end)),
            kind: TokenKind::ParagraphBreak,
        });
    }

    /// Append an `Unlintable` token covering the entire node.
    fn push_unlintable(&mut self, node: &Node) {
        if let Some(span) = self.span_of(node) {
            self.tokens.push(Token {
                span,
                kind: TokenKind::Unlintable,
            });
        }
    }

    /// Run the [`PlainEnglish`] parser over a char range of the source.
    fn parse_english(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }

        let mut new_tokens = PlainEnglish.parse(&self.source[start..end]);

        new_tokens
            .iter_mut()
            .for_each(|token| token.span.push_by(start));

        self.tokens.append(&mut new_tokens);
    }

    /// Tokenize a raw source range as English prose, skipping Markdown syntax (escapes, brackets, and line prefixes).
    fn parse_text_range(&mut self, span: Span<char>) {
        let mut chunk_start = span.start;
        let mut idx = span.start;

        while idx < span.end {
            match self.source[idx] {
                '\\' if idx + 1 < span.end && self.source[idx + 1].is_ascii_punctuation() => {
                    self.parse_english(chunk_start, idx);
                    chunk_start = idx + 1;
                    idx += 2;
                }
                '[' | ']' => {
                    self.parse_english(chunk_start, idx);
                    self.parse_english(idx, idx + 1);
                    chunk_start = idx + 1;
                    idx += 1;
                }
                '\r' | '\n' => {
                    self.parse_english(chunk_start, idx);
                    self.tokens.push(Token {
                        span: Span::new_with_len(idx, 1),
                        kind: TokenKind::Newline(1),
                    });

                    // Move past the line ending.
                    if self.source[idx] == '\r'
                        && self.source.get(idx + 1) == Some(&'\n')
                        && idx + 1 < span.end
                    {
                        idx += 1;
                    }
                    idx += 1;

                    // Skip the continuation line's block prefix: indentation
                    // and any blockquote markers.
                    while idx < span.end && matches!(self.source[idx], ' ' | '\t' | '>') {
                        idx += 1;
                    }

                    chunk_start = idx;
                }
                _ => idx += 1,
            }
        }

        self.parse_english(chunk_start, span.end);
    }

    /// Walk the children of a parent node.
    fn walk_children(&mut self, node: &Node, tight_item: bool) {
        if let Some(children) = node.children() {
            for child in children {
                self.walk(child, tight_item);
            }
        }
    }

    /// Recursively tokenize a node.
    fn walk(&mut self, node: &Node, tight_item: bool) {
        match node {
            Node::Root(_) => self.walk_children(node, false),
            Node::Paragraph(_) => {
                self.inline_depth += 1;
                self.walk_children(node, false);
                self.inline_depth -= 1;

                // Paragraphs implicitly wrapping the contents of a tight list
                // item are invisible in the source, so they don't end with a
                // break of their own.
                if !tight_item {
                    self.push_paragraph_break();
                }
            }
            Node::Heading(_) => {
                if let Some(span) = self.span_of(node) {
                    self.tokens.push(Token {
                        span: Span::empty(span.start),
                        kind: TokenKind::HeadingStart,
                    });
                }

                self.inline_depth += 1;
                self.walk_children(node, false);
                self.inline_depth -= 1;
                self.push_paragraph_break();
            }
            Node::List(list) => {
                if let Some(span) = self.span_of(node) {
                    self.tokens.push(Token {
                        span: Span::empty(span.start),
                        kind: TokenKind::Newline(2),
                    });
                }

                // List items are tight when neither the list nor the item
                // itself is "spread" (separated by blank lines).
                self.walk_children(node, !list.spread);
            }
            Node::ListItem(item) => {
                self.walk_children(node, tight_item && !item.spread);
                self.push_paragraph_break();
            }
            Node::TableCell(_) => {
                self.inline_depth += 1;
                self.walk_children(node, false);
                self.inline_depth -= 1;
                self.push_paragraph_break();
            }
            Node::Code(_) => {
                self.push_unlintable(node);
                self.push_paragraph_break();
            }
            Node::Math(_) | Node::InlineCode(_) => {
                self.push_unlintable(node);
            }
            Node::Html(_) => {
                self.push_unlintable(node);

                // Block-level HTML ends the surrounding "paragraph", while
                // inline HTML lives within one.
                if self.inline_depth == 0 {
                    self.push_paragraph_break();
                }
            }
            Node::InlineMath(_) => {
                if let Some(span) = self.span_of(node) {
                    // The inner parser's single-dollar math detection is
                    // aggressive enough to swallow ordinary currency (e.g.
                    // `$25 $24`). Demote implausible math back to plain text.
                    if is_plausible_inline_math(&self.source[span.start..span.end]) {
                        self.push_unlintable(node);
                    } else {
                        self.parse_text_range(span);
                    }
                }
            }
            Node::MdxjsEsm(_) | Node::MdxFlowExpression(_) => {
                self.push_unlintable(node);
                self.push_paragraph_break();
            }
            Node::MdxTextExpression(_) => {
                self.push_unlintable(node);
            }
            Node::Break(_) => {
                if let Some(span) = self.span_of(node) {
                    self.tokens.push(Token {
                        span: Span::new_with_len(span.start, 1),
                        kind: TokenKind::Newline(2),
                    });
                }
            }
            Node::Link(_) => {
                if self.options.ignore_link_title {
                    self.push_unlintable(node);
                } else {
                    self.walk_children(node, false);
                }
            }
            Node::Text(_) => {
                if let Some(span) = self.span_of(node) {
                    self.parse_text_range(span);
                }
            }
            // Containers whose prose should be linted.
            Node::Blockquote(_)
            | Node::FootnoteDefinition(_)
            | Node::Emphasis(_)
            | Node::Strong(_)
            | Node::Delete(_)
            | Node::LinkReference(_)
            | Node::Table(_)
            | Node::TableRow(_)
            | Node::MdxJsxFlowElement(_)
            | Node::MdxJsxTextElement(_) => self.walk_children(node, false),
            // Nodes that produce no tokens at all.
            Node::Image(_)
            | Node::ImageReference(_)
            | Node::FootnoteReference(_)
            | Node::Definition(_)
            | Node::ThematicBreak(_)
            | Node::Yaml(_)
            | Node::Toml(_) => (),
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

            while let Some((a, b)) = tokens.get(cursor).zip(tokens.get(cursor + 1)) {
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

            while let Some((a, b)) = tokens.get(cursor).zip(tokens.get(cursor + 1)) {
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

        while let Some((a, b)) = tokens.get(cursor).zip(tokens.get(cursor + 1)) {
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
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let source_str: String = source.iter().collect();

        // Attempt MDX first when requested, falling back to ordinary Markdown
        // if the input is not valid MDX (e.g. malformed JSX).
        let ast = if self.options.mdx {
            markdown::to_mdast(&source_str, &mdx_parse_options())
                .or_else(|_| markdown::to_mdast(&source_str, &markdown_parse_options()))
        } else {
            markdown::to_mdast(&source_str, &markdown_parse_options())
        };

        let Ok(ast) = ast else {
            // Parsing ordinary Markdown is infallible, so this is unreachable
            // in practice. Fall back to plain English to be safe.
            return PlainEnglish.parse(source);
        };

        let mut tokenizer = MdastTokenizer {
            source,
            // Build a mapping from the inner parser's byte-based indexing to
            // Harper's char-based indexing.
            byte_to_char: build_byte_to_char_map(&source_str),
            options: self.options,
            tokens: Vec::new(),
            inline_depth: 0,
        };

        tokenizer.walk(&ast, false);

        let mut tokens = tokenizer.tokens;

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

    #[test]
    fn hang() {
        let opts = MarkdownOptions::default();
        let parser = Markdown::new(opts);
        let _res = parser.parse_str("[[#|]]:A]");
    }

    #[test]
    fn hang2() {
        // This seems to only be a java specific problem…
        let opts = MarkdownOptions::default();
        let parser = Markdown::new(opts);
        let _res = parser.parse_str("//{@j");
    }

    #[test]
    fn simple_headings_are_marked() {
        let opts = MarkdownOptions::default();
        let parser = Markdown::new(opts);
        let tokens = parser.parse_str("# This is a simple heading");

        assert_eq!(tokens.iter_heading_starts().count(), 1);
        assert_eq!(tokens.iter_headings().count(), 1);
    }

    fn mdx_parser() -> Markdown {
        Markdown::new(MarkdownOptions {
            mdx: true,
            ..MarkdownOptions::default()
        })
    }

    #[test]
    fn mdx_esm_is_unlintable() {
        let source = "import {Chart} from './chart.js'\n\nThis is prose.";
        let tokens = mdx_parser().parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind.clone()).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Unlintable,
                TokenKind::ParagraphBreak,
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::Period),
            ]
        ));
    }

    #[test]
    fn mdx_jsx_text_contents_are_linted() {
        let source = "Hello <Em>beautiful</Em> world.";
        let tokens = mdx_parser().parse_str(source);
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
                TokenKind::Punctuation(Punctuation::Period),
            ]
        ));
    }

    #[test]
    fn mdx_jsx_flow_contents_are_linted() {
        let source = "<Note>\n  This is prose inside a component.\n</Note>";
        let tokens = mdx_parser().parse_str(source);

        assert!(tokens.iter_words().count() >= 6);
        assert_eq!(tokens.iter_unlintables().count(), 0);
    }

    #[test]
    fn mdx_text_expression_is_unlintable() {
        let source = "The answer is {6 * 7} indeed.";
        let tokens = mdx_parser().parse_str(source);
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
                TokenKind::Space(1),
                TokenKind::Unlintable,
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::Period),
            ]
        ));
    }

    #[test]
    fn mdx_flow_expression_is_unlintable() {
        let source = "{/* a comment */}\n\nProse here.";
        let tokens = mdx_parser().parse_str(source);

        assert_eq!(tokens.iter_unlintables().count(), 1);
        assert_eq!(tokens.iter_words().count(), 2);
    }

    #[test]
    fn mdx_jsx_attributes_are_not_linted() {
        let source = "<Chart kind=\"scatter\" data={data} /> Words after.";
        let tokens = mdx_parser().parse_str(source);

        // Only the prose outside the element should produce words.
        assert_eq!(tokens.iter_words().count(), 2);
    }

    #[test]
    fn mdx_invalid_jsx_falls_back_to_markdown() {
        // `<ctrl-g>` is not valid JSX, so MDX parsing fails and we fall back
        // to ordinary Markdown, where it is raw HTML.
        let source = "The range of inputs from <ctrl-g> to ctrl-z";
        let tokens = mdx_parser().parse_str(source);

        assert_eq!(tokens.iter_unlintables().count(), 1);
        assert!(tokens.iter_words().count() >= 5);
    }

    #[test]
    fn mdx_disabled_by_default() {
        // With MDX off, `{6 * 7}` is just text.
        let source = "The answer is {6 * 7} indeed.";
        let tokens = Markdown::default().parse_str(source);

        assert_eq!(tokens.iter_unlintables().count(), 0);
    }

    #[test]
    fn mdx_nested_jsx_and_emphasis() {
        let source = "<Wrapper>\n  Some *emphasized* text.\n</Wrapper>";
        let tokens = mdx_parser().parse_str(source);

        assert_eq!(tokens.iter_words().count(), 3);
        assert_eq!(tokens.iter_unlintables().count(), 0);
    }

    #[test]
    fn mdx_export_is_unlintable() {
        let source = "export const x = 1\n\nReal prose.";
        let tokens = mdx_parser().parse_str(source);

        assert_eq!(tokens.iter_unlintables().count(), 1);
        assert_eq!(tokens.iter_words().count(), 2);
    }

    #[test]
    fn mdx_gfm_table_still_works() {
        let source = "| Head |\n| ---- |\n| Cell |";
        let tokens = mdx_parser().parse_str(source);

        assert_eq!(tokens.iter_words().count(), 2);
    }

    #[test]
    fn mdx_code_block_still_unlintable() {
        let source = "```js\nconst x = 1;\n```\n\nProse.";
        let tokens = mdx_parser().parse_str(source);

        assert_eq!(tokens.iter_unlintables().count(), 1);
        assert_eq!(tokens.iter_words().count(), 1);
    }

    #[test]
    fn mdx_heading_is_marked() {
        let source = "# Heading\n\n<Component>body</Component>";
        let tokens = mdx_parser().parse_str(source);

        assert_eq!(tokens.iter_heading_starts().count(), 1);
    }

    #[test]
    fn mdx_survives_pathological_input() {
        // Should not panic or hang, even on invalid fragments.
        for source in ["<", "{", "</>", "{{}", "<a b={>", "[[#|]]:A]"] {
            let _ = mdx_parser().parse_str(source);
        }
    }

    #[test]
    fn multiple_headings_are_marked() {
        let opts = MarkdownOptions::default();
        let parser = Markdown::new(opts);
        let tokens = parser.parse_str(
            r#"# This is a simple heading

## This is a second simple heading"#,
        );

        assert_eq!(tokens.iter_heading_starts().count(), 2);
        assert_eq!(tokens.iter_headings().count(), 2);
    }
}
