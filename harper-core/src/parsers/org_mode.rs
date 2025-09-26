use super::{Parser, PlainEnglish};
use crate::{Span, Token, TokenKind};

#[derive(Debug, PartialEq, Copy, Clone)]
enum SourceBlockMarker {
    Begin,
    End,
}

// Check if a line starts with a header (starts with one or more '*')
fn is_header_line(chars: &[char], start: usize) -> bool {
    chars.get(start).is_some_and(|c| *c == '*')
}

// Check if a line starts with a source block begin/end
fn is_source_block_marker(chars: &[char], start: usize) -> Option<SourceBlockMarker> {
    let line = get_line_from_start(chars, start);
    let line_str: String = line.iter().collect();
    let line_str = line_str.trim();

    if line_str.starts_with("#+BEGIN_SRC") || line_str.starts_with("#+begin_src") {
        Some(SourceBlockMarker::Begin)
    } else if line_str.starts_with("#+END_SRC") || line_str.starts_with("#+end_src") {
        Some(SourceBlockMarker::End)
    } else {
        None
    }
}

// Check if a line is a directive (starts with #+)
fn is_directive(chars: &[char], start: usize) -> bool {
    if start + 1 >= chars.len() {
        return false;
    }
    chars[start] == '#' && chars[start + 1] == '+'
}

// Check if a line is a list item (starts with -, +, or number)
fn is_list_item(chars: &[char], start: usize) -> bool {
    let mut pos = start;

    // initial whitespaces or tabs
    while pos < chars.len() && (chars[pos] == ' ' || chars[pos] == '\t') {
        pos += 1;
    }

    if pos >= chars.len() {
        return false;
    }

    // Check for - or + followed by space
    if (chars[pos] == '-' || chars[pos] == '+') && pos + 1 < chars.len() && chars[pos + 1] == ' ' {
        return true;
    }

    // Check for numbered list
    if chars[pos].is_ascii_digit() {
        let mut num_pos = pos;
        while num_pos < chars.len() && chars[num_pos].is_ascii_digit() {
            num_pos += 1;
        }

        if num_pos < chars.len()
            && (chars[num_pos] == '.' || chars[num_pos] == ')')
            && num_pos + 1 < chars.len()
            && chars[num_pos + 1] == ' '
        {
            return true;
        }
    }

    false
}

// Convert tabs to spaces in list items to avoid French spaces error
fn normalize_list_item_whitespace(chars: &[char]) -> Vec<char> {
    let mut result = Vec::new();
    let mut init_list = false;
    for &ch in chars {
        if !init_list && ch == '\t' {
            result.push(' ');
            init_list = true;
        } else {
            result.push(ch);
        }
    }
    result
}

// Get the rest of the line from a starting position
fn get_line_from_start(chars: &[char], start: usize) -> &[char] {
    let mut end = start;
    while end < chars.len() && chars[end] != '\n' {
        end += 1;
    }
    &chars[start..end]
}

// Find the end of the current line starting from position
fn find_line_end(chars: &[char], start: usize) -> usize {
    let mut pos = start;
    while pos < chars.len() && chars[pos] != '\n' {
        pos += 1;
    }
    if pos < chars.len() && chars[pos] == '\n' {
        pos + 1 // Include the newline
    } else {
        pos
    }
}

// Find the start of the line containing the given position
fn find_line_start(chars: &[char], pos: usize) -> usize {
    let mut start = pos;
    while start > 0 && chars[start - 1] != '\n' {
        start -= 1;
    }
    start
}

/// A parser that wraps the [`PlainEnglish`] parser that allows one to parse
/// Org-mode files.
///
/// Will ignore code blocks, source blocks, and other org-mode specific elements
/// that should not be linted for prose.
#[derive(Default, Clone, Debug, Copy)]
pub struct OrgMode;

impl OrgMode {}

impl Parser for OrgMode {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let english_parser = PlainEnglish;
        let mut tokens = Vec::new();
        let mut cursor = 0;
        let mut in_source_block = false;

        while cursor < source.len() {
            let line_start = find_line_start(source, cursor);

            // Check for source block markers
            let source_marker = is_source_block_marker(source, line_start);
            if let Some(marker) = source_marker {
                in_source_block = marker == SourceBlockMarker::Begin;
            }

            // If we're in a source block or found a source block marker, make the line unlintable
            if in_source_block || source_marker.is_some() {
                let line_end = find_line_end(source, line_start);
                tokens.push(Token {
                    span: Span::new(line_start, line_end),
                    kind: TokenKind::Unlintable,
                });
                cursor = line_end;
                continue;
            }

            // Check for headers
            if is_header_line(source, line_start) {
                let line_end = find_line_end(source, line_start);

                // Find where the header text starts (after the stars and spaces)
                let mut header_text_start = line_start;
                while header_text_start < line_end
                    && (source[header_text_start] == '*' || source[header_text_start] == ' ')
                {
                    header_text_start += 1;
                }

                // If there's actual text after the stars, parse it
                if header_text_start < line_end {
                    let mut header_tokens =
                        english_parser.parse(&source[header_text_start..line_end]);
                    header_tokens
                        .iter_mut()
                        .for_each(|token| token.span.push_by(header_text_start));
                    tokens.append(&mut header_tokens);
                }

                // Add paragraph break after header
                tokens.push(Token {
                    span: Span::new_with_len(line_end.saturating_sub(1), 0),
                    kind: TokenKind::ParagraphBreak,
                });

                cursor = line_end;
                continue;
            }

            // Check for directives (#+SOMETHING)
            if is_directive(source, line_start) {
                let line_end = find_line_end(source, line_start);
                tokens.push(Token {
                    span: Span::new(line_start, line_end),
                    kind: TokenKind::Unlintable,
                });
                cursor = line_end;
                continue;
            }

            // Check for list items and normalize tabs to avoid French spaces
            if is_list_item(source, line_start) {
                let line_end = find_line_end(source, line_start);
                let line_chars = &source[line_start..line_end];
                let normalized_chars = normalize_list_item_whitespace(line_chars);

                let mut line_tokens = english_parser.parse(&normalized_chars);
                line_tokens
                    .iter_mut()
                    .for_each(|token| token.span.push_by(line_start));
                tokens.append(&mut line_tokens);

                cursor = line_end;
                continue;
            }

            // For normal text, parse with the English parser
            let line_end = find_line_end(source, cursor);
            if cursor < line_end {
                let mut line_tokens = english_parser.parse(&source[cursor..line_end]);
                line_tokens
                    .iter_mut()
                    .for_each(|token| token.span.push_by(cursor));
                tokens.append(&mut line_tokens);
            }

            cursor = line_end;
        }

        // Remove trailing newline/paragraph break tokens if the source doesn't actually end with a newline.
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

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::super::StrParser;
    use super::OrgMode;
    use crate::TokenKind;

    #[test]
    fn simple_text() {
        let source = "This is simple text.";
        let tokens = OrgMode.parse_str(source);
        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Word(_))));
    }

    #[test]
    fn header_parsing() {
        let source = "* This is a header\nThis is regular text.";
        let tokens = OrgMode.parse_str(source);
        let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

        // Should have words from header and paragraph break
        assert!(token_kinds.iter().any(|k| matches!(k, TokenKind::Word(_))));
        assert!(
            token_kinds
                .iter()
                .any(|k| matches!(k, TokenKind::ParagraphBreak))
        );
    }

    #[test]
    fn multiple_level_headers() {
        let source = "** Second level header\n*** Third level header";
        let tokens = OrgMode.parse_str(source);
        let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

        // Should parse text from both headers
        let word_count = token_kinds
            .iter()
            .filter(|k| matches!(k, TokenKind::Word(_)))
            .count();
        assert!(word_count >= 4); // "Second", "level", "Third", "header"
    }

    #[test]
    fn source_block_unlintable() {
        let source = r#"Regular text.
#+BEGIN_SRC rust
fn main() {
    println!("Hello, world!");
}
#+END_SRC
More regular text."#;

        let tokens = OrgMode.parse_str(source);
        let unlintable_count = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Unlintable))
            .count();

        // Should have unlintable tokens for the source block lines
        assert!(unlintable_count > 0);

        // Should still have regular words from the non-source-block text
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Word(_))));
    }

    #[test]
    fn directive_unlintable() {
        let source = r#"#+TITLE: My Document
#+AUTHOR: Test Author
This is regular text."#;

        let tokens = OrgMode.parse_str(source);
        let unlintable_count = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Unlintable))
            .count();

        // Should have unlintable tokens for directives
        assert_eq!(unlintable_count, 2);

        // Should still have regular words
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Word(_))));
    }

    #[test]
    fn case_insensitive_source_blocks() {
        let source = r#"#+begin_src python
print("hello")
#+end_src"#;

        let tokens = OrgMode.parse_str(source);
        let unlintable_count = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Unlintable))
            .count();

        // All lines should be unlintable
        assert_eq!(unlintable_count, 3);
    }

    #[test]
    fn empty_header() {
        let source = "*\nRegular text.";
        let tokens = OrgMode.parse_str(source);

        // Should handle empty headers gracefully
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Word(_))));
    }

    #[test]
    fn no_trailing_newline() {
        let source = "Simple text without newline";
        let tokens = OrgMode.parse_str(source);

        // Should not end with newline token if source doesn't
        assert!(!tokens.last().unwrap().kind.is_newline());
    }

    #[test]
    fn list_items_with_tabs() {
        let source = "- First item\n\t- Indented with tab\n+ Second item\n1. Numbered item";
        let tokens = OrgMode.parse_str(source);

        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Word(_))));

        let unlintable_count = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Unlintable))
            .count();
        assert_eq!(unlintable_count, 0);
    }

    #[test]
    fn mixed_list_formats() {
        let source = r#"- Bullet item
1. Numbered item
+ Plus item
2) Parenthesis numbered"#;

        let tokens = OrgMode.parse_str(source);

        // Should recognize all list formats
        let word_count = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Word(_)))
            .count();

        assert!(word_count == 8, "{:?}", tokens); // "Bullet", "item", "Numbered", "item", "Plus", "item", "Parenthesis", "numbered"
    }
}
