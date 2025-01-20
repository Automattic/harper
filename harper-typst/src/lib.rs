mod offset_cursor;
mod typst_translator;

use offset_cursor::OffsetCursor;
use typst_translator::TypstTranslator;

use harper_core::{parsers::Parser, Token};
use itertools::Itertools;
use typst_syntax::{
    ast::{AstNode, Expr, Markup},
    Source, SyntaxNode,
};

/// A parser that wraps Harper's `PlainEnglish` parser allowing one to ingest Typst files.
pub struct Typst;

impl Parser for Typst {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let source_str: String = source.iter().collect();

        // Transform the source into an AST through the `typst_syntax` crate
        let typst_document = Source::detached(source_str);
        let typst_tree = Markup::from_untyped(typst_document.root())
            .expect("Unable to create typst document from parsed tree!");

        // Owned collection of nodes forcibly casted to paragraph breaks
        let untyped_nodes = typst_tree
            .exprs()
            .map(|e| {
                let mut node = SyntaxNode::placeholder(typst_syntax::SyntaxKind::Parbreak);
                node.synthesize(e.span());
                node
            })
            .collect_vec();

        // Converts newlines after certain elements to paragraph breaks
        // This is accomplished here instead of in the translating module because at this point there is
        // still semantic information associated with the elements.
        //
        // Newlines are separate expressions in the parse tree (as the Space variant)
        let should_parbreak = |e1, e2, e3| {
            matches!(e2, Expr::Space(_))
                && (matches!(e1, Expr::Heading(_) | Expr::List(_))
                    || matches!(e3, Expr::Heading(_) | Expr::List(_)))
        };

        let mut exprs: Vec<Expr> = Vec::new();
        let mut last_element: Option<Expr> = None;
        for ((i, expr), (_, next_expr)) in typst_tree.exprs().enumerate().tuple_windows() {
            let mut current_expr = expr;
            if let Some(last_element) = last_element {
                if should_parbreak(last_element, expr, next_expr) {
                    let pbreak = typst_syntax::ast::Parbreak::from_untyped(&untyped_nodes[i])
                        .expect("Unable to convert expression to Parbreak");
                    current_expr = Expr::Parbreak(pbreak);
                }
            }
            exprs.push(current_expr);
            last_element = Some(expr)
        }
        // Push last element because it will be excluded by tuple_windows() above
        if let Some(last) = typst_tree.exprs().last() {
            exprs.push(last);
        }

        dbg!(&exprs);

        // Recurse through AST to create tokens
        let parse_helper = TypstTranslator::new(&typst_document);
        exprs
            .into_iter()
            .filter_map(|ex| parse_helper.parse_expr(ex, OffsetCursor::new(&typst_document)))
            .flatten()
            .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use ordered_float::OrderedFloat;

    use super::Typst;
    use harper_core::{Document, NounData, Punctuation, TokenKind, WordMetadata};

    #[test]
    fn number() {
        let source = "12 is larger than 11, but much less than 11!";

        let document = Document::new_curated(source, &Typst);
        let token_kinds = document.tokens().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Number(OrderedFloat(12.0), None),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Number(OrderedFloat(11.0), None),
                TokenKind::Punctuation(Punctuation::Comma),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Number(OrderedFloat(11.0), None),
                TokenKind::Punctuation(Punctuation::Bang),
            ]
        ))
    }

    #[test]
    fn math_unlintable() {
        let source = "$12 > 11$, $12 << 11!$";

        let document = Document::new_curated(source, &Typst);
        let token_kinds = document.tokens().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Unlintable,
                TokenKind::Punctuation(Punctuation::Comma),
                TokenKind::Space(1),
                TokenKind::Unlintable,
            ]
        ))
    }

    #[test]
    fn dict_parsing() {
        let source = r#"#let dict = (
                          name: "Typst",
                          born: 2019,
                        )"#;

        let document = Document::new_curated(source, &Typst);
        let token_kinds = document.tokens().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        let charslice = source.chars().collect_vec();
        let tokens = document.tokens().collect_vec();
        assert_eq!(tokens[2].span.get_content_string(&charslice), "Typst");

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Unlintable, // dict
                TokenKind::Unlintable, // name (key 1)
                TokenKind::Word(_),    // Typst (value 1)
                TokenKind::Unlintable, // born (key 2)
                TokenKind::Unlintable, // 2019 (value 2)
            ]
        ))
    }

    #[test]
    fn str_parsing() {
        let source = r#"#let ident = "This is a string""#;

        let document = Document::new_curated(source, &Typst);
        let token_kinds = document.tokens().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert!(matches!(
            &token_kinds.as_slice(),
            &[
                TokenKind::Unlintable, // ident
                TokenKind::Word(_),    // This
                TokenKind::Space(1),   //
                TokenKind::Word(_),    // is
                TokenKind::Space(1),   //
                TokenKind::Word(_),    // a
                TokenKind::Space(1),   //
                TokenKind::Word(_),    // string
            ]
        ))
    }

    #[test]
    fn non_adjacent_spaces_not_condensed() {
        let source = r#"#authors_slice.join(", ", last: ", and ")  bob"#;

        let document = Document::new_curated(source, &Typst);
        let token_kinds = document.tokens().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert!(matches!(
            &token_kinds.as_slice(),
            &[
                TokenKind::Unlintable, // authors_slice.join
                TokenKind::Punctuation(Punctuation::Comma),
                TokenKind::Space(1),
                TokenKind::Unlintable, // last
                TokenKind::Punctuation(Punctuation::Comma),
                TokenKind::Space(1),
                TokenKind::Word(_), // and
                TokenKind::Space(1),
                TokenKind::Space(2),
                TokenKind::Word(_), // bob
            ]
        ))
    }

    #[test]
    fn header_parsing() {
        let source = "= Header
                      Paragraph";

        let document = Document::new_curated(source, &Typst);
        let token_kinds = document.tokens().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        let charslice = source.chars().collect_vec();
        let tokens = document.tokens().collect_vec();
        assert_eq!(tokens[0].span.get_content_string(&charslice), "Header");
        assert_eq!(tokens[2].span.get_content_string(&charslice), "Paragraph");

        assert!(matches!(
            &token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::ParagraphBreak,
                TokenKind::Word(_)
            ]
        ))
    }

    #[test]
    fn parbreak() {
        let source = "Paragraph

                      Paragraph";

        let document = Document::new_curated(source, &Typst);
        let token_kinds = document.tokens().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert!(matches!(
            &token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::ParagraphBreak,
                TokenKind::Word(_),
            ]
        ))
    }

    #[test]
    fn label_ref_unlintable() {
        let source = "= Header
                      <label>
                      Paragraph @label";

        let document = Document::new_curated(source, &Typst);
        let token_kinds = document.tokens().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert!(matches!(
            &token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::ParagraphBreak,
                TokenKind::Unlintable,
                TokenKind::Newline(_),
                TokenKind::Word(_),
                TokenKind::Space(_),
                TokenKind::Unlintable,
            ]
        ))
    }

    #[test]
    fn sentence() {
        let source = "This is a sentence, it is not interesting.";

        let document = Document::new_curated(source, &Typst);
        let token_kinds = document.tokens().map(|t| t.kind).collect_vec();
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
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::Comma),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::Period),
            ]
        ))
    }

    #[test]
    fn smart_apostrophe_newline() {
        let source = "group’s
                      writing";

        let document = Document::new_curated(source, &Typst);
        let token_kinds = document.tokens().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        let charslice = source.chars().collect_vec();
        let tokens = document.tokens().collect_vec();
        assert_eq!(tokens[2].span.get_content_string(&charslice), "writing");

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Word(WordMetadata {
                    noun: Some(NounData {
                        is_possessive: Some(true),
                        ..
                    }),
                    ..
                }),
                TokenKind::Newline(1),
                TokenKind::Word(_),
            ]
        ));
    }

    #[test]
    fn newline_in_paragraph() {
        let source = "Paragraph with
newlines
not paragraph breaks";

        let document = Document::new_curated(source, &Typst);
        let token_kinds = document.tokens().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert!(matches!(
            &token_kinds.as_slice(),
            &[
                TokenKind::Word(_), // Paragraph
                TokenKind::Space(_),
                TokenKind::Word(_), // with
                TokenKind::Newline(1),
                TokenKind::Word(_), // newlines
                TokenKind::Newline(1),
                TokenKind::Word(_), // not
                TokenKind::Space(_),
                TokenKind::Word(_), // paragraph
                TokenKind::Space(_),
                TokenKind::Word(_), // breaks
            ]
        ))
    }

    #[test]
    fn parbreaks_in_list() {
        let source = "This is a list:
- p1
- p2
- p3";

        let document = Document::new_curated(source, &Typst);
        let token_kinds = document.tokens().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert!(matches!(
            &token_kinds.as_slice(),
            &[
                TokenKind::Word(_), // This
                TokenKind::Space(_),
                TokenKind::Word(_), // is
                TokenKind::Space(_),
                TokenKind::Word(_), // a
                TokenKind::Space(_),
                TokenKind::Word(_), // list
                TokenKind::Punctuation(Punctuation::Colon),
                TokenKind::ParagraphBreak,
                TokenKind::Word(_),
                TokenKind::ParagraphBreak,
                TokenKind::Word(_),
                TokenKind::ParagraphBreak,
                TokenKind::Word(_)
            ]
        ))
    }
}
