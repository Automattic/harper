mod offset_cursor;
mod typst_translator;

use offset_cursor::OffsetCursor;
use typst_translator::TypstTranslator;

use harper_core::{Token, parsers::Parser};
use itertools::Itertools;
use typst_syntax::{
    Source, SyntaxNode,
    ast::{AstNode, Expr, Markup},
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

        // Recurse through AST to create tokens
        let parse_helper = TypstTranslator::new(&typst_document);
        let mut buf = Vec::new();
        let exprs = typst_tree.exprs().collect_vec();
        let exprs = convert_parbreaks(&mut buf, &exprs);
        exprs
            .into_iter()
            .filter_map(|ex| parse_helper.parse_expr(ex, OffsetCursor::new(&typst_document)))
            .flatten()
            .collect_vec()
    }
}

/// Converts newlines after certain elements to paragraph breaks
/// This is accomplished here instead of in the translating module because at this point there is
/// still semantic information associated with the elements.
///
/// Newlines are separate expressions in the parse tree (as the Space variant)
fn convert_parbreaks<'a>(buf: &'a mut Vec<SyntaxNode>, exprs: &'a [Expr]) -> Vec<Expr<'a>> {
    // Owned collection of nodes forcibly casted to paragraph breaks
    *buf = exprs
        .iter()
        .map(|e| {
            let mut node = SyntaxNode::placeholder(typst_syntax::SyntaxKind::Parbreak);
            node.synthesize(e.span());
            node
        })
        .collect_vec();

    let should_parbreak = |e1, e2, e3| {
        matches!(e2, Expr::Space(_))
            && (matches!(e1, Expr::Heading(_) | Expr::List(_))
                || matches!(e3, Expr::Heading(_) | Expr::List(_)))
    };

    let mut res: Vec<Expr> = Vec::new();
    let mut last_element: Option<Expr> = None;
    for ((i, expr), (_, next_expr)) in exprs.iter().enumerate().tuple_windows() {
        let mut current_expr = *expr;
        if let Some(last_element) = last_element
            && should_parbreak(last_element, *expr, *next_expr)
        {
            let pbreak = typst_syntax::ast::Parbreak::from_untyped(&buf[i])
                .expect("Unable to convert expression to Parbreak");
            current_expr = Expr::Parbreak(pbreak);
        }
        res.push(current_expr);
        last_element = Some(*expr)
    }
    // Push last element because it will be excluded by tuple_windows() above
    if let Some(last) = exprs.iter().last() {
        res.push(*last);
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use harper_core::parsers::StrParser;

    #[test]
    fn issue_1898() {
        Typst.parse_str("#for ");
        Typst.parse_str("#(.$#$$$. ");
        Typst.parse_str("=#{m\"\".'m\"\"#p#");
    }
}
