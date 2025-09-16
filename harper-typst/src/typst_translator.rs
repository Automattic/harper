use crate::OffsetCursor;
use harper_core::{
    Punctuation, Token, TokenKind,
    parsers::{PlainEnglish, StrParser},
};
use itertools::Itertools;
use typst_syntax::{
    Source,
    ast::{
        Arg, ArrayItem, AstNode, DestructuringItem, DictItem, Expr, FuncCall, Ident,
        LetBindingKind, Param, Pattern, Spread,
    },
};

/// Directly translate a span ($a) in a Typst source ($doc) to a token.
macro_rules! def_token {
    ($doc:expr, $a:expr, $kind:expr, $offset:ident) => {{
        let range = $doc.range($a.span())?;
        let start = $offset.push_to(range.start)?;
        let end_char_loc = start.push_to(range.end)?.char;

        Some(vec![Token {
            span: harper_core::Span::new(start.char, end_char_loc),
            kind: $kind,
        }])
    }};
}

/// Combine the results of multiple parsing calls.
macro_rules! merge {
    [$($inner:expr),*] => {
        Some(
            [$($inner),*]
                .into_iter()
                .flatten()
                .flatten()
                .collect_vec(),
        )
    };
}

/// Contains values used in parsing so they don't have to be passed around so much.
#[derive(Clone, Copy)]
pub struct TypstTranslator<'a> {
    doc: &'a Source,
}

impl<'a> TypstTranslator<'a> {
    pub fn new(doc: &'a Source) -> Self {
        Self { doc }
    }

    /// Use the [`PlainEnglish`] parser to parse plain text from a Typst expression.
    fn parse_english(self, str: impl Into<String>, offset: OffsetCursor) -> Option<Vec<Token>> {
        Some(
            PlainEnglish
                .parse_str(str.into())
                .into_iter()
                .map(|mut t| {
                    t.span.push_by(offset.char);
                    t
                })
                .collect_vec(),
        )
    }

    /// Parse a pattern, one of the elements of Typst syntax
    fn parse_pattern(self, pat: Pattern, offset: OffsetCursor) -> Option<Vec<Token>> {
        /// Simplification of [`def_token!`] that bakes-in local variables
        macro_rules! token {
            ($a:expr, $kind:expr) => {
                def_token!(self.doc, $a, $kind, offset)
            };
        }

        match pat {
            Pattern::Normal(expr) => self.parse_expr(expr, offset),
            Pattern::Placeholder(underscore) => token!(underscore, TokenKind::Unlintable),
            Pattern::Parenthesized(parenthesized) => merge![
                self.parse_expr(parenthesized.expr(), offset),
                self.parse_pattern(parenthesized.pattern(), offset)
            ],
            Pattern::Destructuring(destructuring) => Some(
                destructuring
                    .items()
                    .filter_map(|item| match item {
                        DestructuringItem::Pattern(pattern) => self.parse_pattern(pattern, offset),
                        DestructuringItem::Named(named) => merge![
                            token!(named.name(), TokenKind::Word(None)),
                            self.parse_pattern(named.pattern(), offset)
                        ],
                        DestructuringItem::Spread(spread) => merge![
                            spread
                                .sink_ident()
                                .and_then(|ident| self.parse_ident(ident, offset)),
                            spread
                                .sink_expr()
                                .and_then(|expr| self.parse_expr(expr, offset))
                        ],
                    })
                    .flatten()
                    .collect(),
            ),
        }
    }

    /// Convenience wrapper of [`Self::parse_expr`] that packages the identifier as an expression
    fn parse_ident(self, ident: Ident, offset: OffsetCursor) -> Option<Vec<Token>> {
        self.parse_expr(Expr::Ident(ident), offset)
    }

    /// Do not use for spreads contained in DestructuringItem
    fn parse_spread(self, spread: Spread, offset: OffsetCursor) -> Option<Vec<Token>> {
        merge![
            self.parse_expr(spread.expr(), offset),
            spread
                .sink_ident()
                .and_then(|ident| self.parse_ident(ident, offset))
        ]
    }

    pub fn parse_expr(self, expr: Expr, offset: OffsetCursor) -> Option<Vec<Token>> {
        // Update the offset that will be passed to other functions by moving it to the beginning
        // of the current expression's span.
        let offset = offset.push_to_span(expr.span())?;

        /// Simplification of [`def_token!`] that bakes-in local variables
        macro_rules! token {
            ($a:expr, $kind:expr) => {
                def_token!(self.doc, $a, $kind, offset)
            };
        }

        /// Quickly recurse without needing to pass in local variables.
        /// Matches both single and many expressions.
        macro_rules! recurse {
            ($inner:expr) => {
                self.parse_expr($inner, offset)
            };
            ($($inner:expr),+) => {
                merge![
                    $(recurse!($inner)),*
                ]
            };
        }

        macro_rules! get_text {
            ($expr:expr) => {
                self.doc
                    .get(self.doc.range($expr.span())?)
                    .expect("Unable to get text from typst document span!")
            };
        }

        fn parbreak() -> Option<Vec<Token>> {
            Some(vec![Token {
                span: harper_core::Span::EMPTY,
                kind: TokenKind::ParagraphBreak,
            }])
        }

        fn isolate(inner: Option<Vec<Token>>) -> Option<Vec<Token>> {
            merge![parbreak(), inner, parbreak()]
        }

        // Recurse on each element of an iterator
        let iter_recurse = |exprs: &mut dyn Iterator<Item = Expr>| {
            let mut buf = Vec::new();
            let exprs = exprs.collect_vec();
            let exprs = super::convert_parbreaks(&mut buf, &exprs);
            Some(
                exprs
                    .into_iter()
                    .filter_map(|e| recurse!(e))
                    .flatten()
                    .collect_vec(),
            )
        };

        // Parse the parameters of a function or closure
        let parse_params = |params: &mut dyn Iterator<Item = Param>| {
            Some(
                params
                    .filter_map(|p| match p {
                        Param::Pos(pattern) => self.parse_pattern(pattern, offset),
                        Param::Named(named) => merge![
                            self.parse_ident(named.name(), offset),
                            recurse!(named.expr())
                        ],
                        Param::Spread(spread) => self.parse_spread(spread, offset),
                    })
                    .flatten()
                    .collect_vec(),
            )
        };

        // Parse the arguments passed to a function or closure call
        let parse_args = |params: &mut dyn Iterator<Item = Arg>| {
            Some(
                params
                    .filter_map(|a| match a {
                        Arg::Pos(expr) => recurse!(expr),
                        Arg::Named(named) => merge![
                            self.parse_ident(named.name(), offset),
                            recurse!(named.expr())
                        ],
                        Arg::Spread(spread) => self.parse_spread(spread, offset),
                    })
                    .flatten()
                    .collect_vec(),
            )
        };

        // Parse a function call
        let parse_func_call = |func: FuncCall| {
            let parse_args_ignored = |ignore_pos: bool, ignore_nameds: &[&str]| {
                let (dead, alive): (Vec<_>, Vec<_>) = func.args().items().partition(|a| match a {
                    Arg::Pos(_) => ignore_pos,
                    Arg::Named(named) => ignore_nameds.contains(&named.name().as_str()),
                    Arg::Spread(_) => false,
                });

                Some(
                    dead.iter()
                        .flat_map(|a| token!(a, TokenKind::Unlintable))
                        .chain(parse_args(&mut alive.into_iter()))
                        .flatten()
                        .collect_vec(),
                )
            };

            let text = get_text!(func.callee());
            merge![
                token!(func.callee(), TokenKind::Unlintable),
                match text {
                    "std.rgb" | "color.rgb" | "rgb" => parse_args_ignored(true, &[]),
                    "std.plugin" | "plugin" => parse_args_ignored(true, &[]),
                    "std.bibliography" | "bibliography" => parse_args_ignored(true, &["style"]),
                    "std.cite" | "cite" => parse_args_ignored(true, &["style"]),
                    "std.raw" | "raw" => parse_args_ignored(false, &["syntaxes", "theme"]),
                    "std.image" | "image" => parse_args_ignored(true, &[]),
                    "std.regex" | "regex" => parse_args_ignored(true, &[]),
                    _ if text.ends_with(".display") => parse_args_ignored(true, &[]),
                    _ => parse_args(&mut func.args().items()),
                }
            ]
        };

        // Delegate parsing based on the kind of Typst expression.
        // Not all expression kinds have defined behavior, so the default behavior is
        // an [`harper_core::TokenKind::Unlintable`] token.
        //
        // A full list of variants is available in the [typst_syntax docs](https://docs.rs/typst/latest/typst/syntax/ast/enum.Expr.html)
        match expr {
            Expr::Text(text) => self.parse_english(text.get(), offset.push_to_span(text.span())?),
            Expr::Space(a) => {
                let mut chars = get_text!(a).chars();
                let first_char = chars.next().unwrap();
                let length = chars.count() + 1;

                if first_char == '\n' {
                    token!(a, TokenKind::Newline(1))
                } else {
                    token!(a, TokenKind::Space(length))
                }
            }
            Expr::Linebreak(a) => token!(a, TokenKind::Newline(1)),
            Expr::Parbreak(a) => token!(a, TokenKind::ParagraphBreak),
            Expr::SmartQuote(quote) => {
                if quote.double() {
                    token!(
                        quote,
                        TokenKind::Punctuation(Punctuation::Quote(harper_core::Quote {
                            twin_loc: None
                        }))
                    )
                } else {
                    token!(quote, TokenKind::Punctuation(Punctuation::Apostrophe))
                }
            }
            Expr::Strong(strong) => iter_recurse(&mut strong.body().exprs()),
            Expr::Emph(emph) => iter_recurse(&mut emph.body().exprs()),
            Expr::Link(a) => token!(a, TokenKind::Url),
            Expr::Heading(heading) => iter_recurse(&mut heading.body().exprs()),
            Expr::List(list_item) => iter_recurse(&mut list_item.body().exprs()),
            Expr::Enum(enum_item) => iter_recurse(&mut enum_item.body().exprs()),
            Expr::Term(term_item) => iter_recurse(
                &mut term_item
                    .term()
                    .exprs()
                    .chain(term_item.description().exprs()),
            ),
            Expr::Str(text) => {
                let offset = offset.push_to_span(text.span())?.char + 1;
                let string = text.to_untyped().text();

                Some(
                    PlainEnglish
                        .parse_str(&string[1..string.len() - 1])
                        .into_iter()
                        .map(|mut t| {
                            t.span.push_by(offset);
                            t
                        })
                        .collect_vec(),
                )
            }
            Expr::Content(content_block) => {
                isolate(iter_recurse(&mut content_block.body().exprs()))
            }
            Expr::Parenthesized(parenthesized) => recurse!(parenthesized.expr()),
            Expr::Array(array) => Some(
                array
                    .items()
                    .filter_map(|i| {
                        if let ArrayItem::Pos(e) = i {
                            recurse!(e)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect_vec(),
            ),
            Expr::Dict(dict) => Some(
                dict.items()
                    .filter_map(|di| match di {
                        DictItem::Named(named) => {
                            merge![
                                self.parse_ident(named.name(), offset),
                                recurse!(named.expr())
                            ]
                        }
                        DictItem::Keyed(keyed) => recurse!(keyed.key(), keyed.expr()),
                        DictItem::Spread(spread) => self.parse_spread(spread, offset),
                    })
                    .flatten()
                    .collect_vec(),
            ),
            Expr::FieldAccess(field_access) => merge![
                recurse!(field_access.target()),
                token!(field_access.field(), TokenKind::Word(None))
            ],
            Expr::Let(let_binding) => merge![
                match let_binding.kind() {
                    LetBindingKind::Normal(pattern) => self.parse_pattern(pattern, offset),
                    LetBindingKind::Closure(ident) => self.parse_ident(ident, offset),
                },
                let_binding.init().and_then(|e| recurse!(e))
            ],
            Expr::DestructAssign(destruct_assignment) => {
                recurse!(destruct_assignment.value())
            }
            Expr::Set(set_rule) => merge![
                recurse!(set_rule.target()),
                set_rule.condition().and_then(|expr| recurse!(expr)),
                parse_args(&mut set_rule.args().items())
            ],
            Expr::Show(show_rule) => merge![
                recurse!(show_rule.transform()),
                show_rule.selector().and_then(|expr| recurse!(expr))
            ],
            Expr::Contextual(contextual) => recurse!(contextual.body()),
            Expr::Conditional(conditional) => merge![
                recurse!(conditional.condition(), conditional.if_body()),
                conditional.else_body().and_then(|expr| recurse!(expr))
            ],
            Expr::While(while_loop) => recurse!(while_loop.condition(), while_loop.body()),
            Expr::For(for_loop) => recurse!(for_loop.iterable(), for_loop.body()),
            Expr::Code(code) => iter_recurse(&mut code.body().exprs()),
            Expr::Closure(closure) => merge![
                closure
                    .name()
                    .and_then(|ident| self.parse_ident(ident, offset)),
                parse_params(&mut closure.params().children()),
                recurse!(closure.body())
            ],
            Expr::FuncCall(func) => parse_func_call(func),
            a => token!(a, TokenKind::Unlintable),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typst_syntax::ast::None;

    #[test]
    fn parse_none_returns_none() {
        let source = Source::detached("");
        let translator = TypstTranslator::new(&source);

        assert!(
            translator
                .parse_expr(Expr::None(None::default()), OffsetCursor::new(&source))
                .is_none()
        )
    }
}
