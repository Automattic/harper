use itertools::Itertools;
use typst_syntax::{
    ast::{
        Arg, ArrayItem, AstNode, DestructuringItem, DictItem, Expr, Ident, LetBindingKind, Markup,
        Param, Pattern, Spread,
    },
    Source,
};

use super::{Parser, PlainEnglish};
use crate::{
    parsers::StrParser,
    patterns::{PatternExt, SequencePattern},
    ConjunctionData, Lrc, NounData, Punctuation, Token, TokenKind, VecExt, WordMetadata,
};

/// A parser that wraps the [`PlainEnglish`] parser allowing one to parse Typst files.
pub struct Typst;

/// Encapsulation of the translation between byte-based spans and char-based spans
#[derive(Debug, Clone, Copy)]
struct OffsetCursor<'a> {
    doc: &'a Source,
    pub char: usize,
    pub byte: usize,
}

impl<'a> OffsetCursor<'a> {
    pub fn new(doc: &'a Source) -> Self {
        Self {
            doc,
            char: 0,
            byte: 0,
        }
    }

    /// Returns a new [`OffsetCursor`] at the given byte based on the current cursor.
    pub fn push_to(self, new_byte: usize) -> Self {
        assert!(new_byte >= self.byte);

        if new_byte == self.byte {
            return self;
        }

        Self {
            char: self.char + self.doc.get(self.byte..new_byte).unwrap().chars().count(),
            byte: new_byte,
            ..self
        }
    }

    /// Returns a new [`OffsetCursor`] at the beginning of the given [`typst_syntax::Span`] based
    /// on the current cursor.
    pub fn push_to_span(self, span: typst_syntax::Span) -> Self {
        let new_byte = self.doc.range(span).unwrap().start;

        self.push_to(new_byte)
    }
}

macro_rules! def_token {
    ($doc:expr, $a:expr, $kind:expr, $offset:ident) => {{
        let range = $doc.range($a.span()).unwrap();
        let start = $offset.push_to(range.start);
        let end_char_loc = start.push_to(range.end).char;

        Some(vec![Token {
            span: crate::Span {
                start: start.char,
                end: end_char_loc,
            },
            kind: $kind,
        }])
    }};
}

macro_rules! merge {
    ($($inner:expr),*) => {
        Some(
            [$($inner),*]
                .into_iter()
                .flatten()
                .flatten()
                .collect_vec(),
        )
    };
}

/// Contains values used in parsing so they don't have to be passed around so much
#[derive(Clone, Copy)]
struct ParseHelper<'a> {
    parser: PlainEnglish,
    doc: &'a Source,
}

impl<'a> ParseHelper<'a> {
    pub fn new(doc: &'a Source) -> Self {
        Self {
            parser: PlainEnglish,
            doc,
        }
    }

    fn parse_english(self, str: impl Into<String>, offset: OffsetCursor) -> Option<Vec<Token>> {
        Some(
            self.parser
                .parse_str(str.into())
                .into_iter()
                .map(|mut t| {
                    t.span.push_by(offset.char);
                    t
                })
                .collect_vec(),
        )
    }

    fn parse_pattern(self, pat: Pattern, offset: OffsetCursor) -> Option<Vec<Token>> {
        macro_rules! token {
            ($a:expr, $kind:expr) => {
                def_token!(self.doc, $a, $kind, offset)
            };
        }

        match pat {
            Pattern::Normal(expr) => self.parse_expr(expr, offset),
            Pattern::Placeholder(underscore) => token!(underscore, TokenKind::Unlintable),
            Pattern::Parenthesized(parenthesized) => merge!(
                self.parse_expr(parenthesized.expr(), offset),
                self.parse_pattern(parenthesized.pattern(), offset)
            ),
            Pattern::Destructuring(destructuring) => Some(
                destructuring
                    .items()
                    .filter_map(|item| match item {
                        DestructuringItem::Pattern(pattern) => self.parse_pattern(pattern, offset),
                        DestructuringItem::Named(named) => merge!(
                            token!(named.name(), TokenKind::Word(WordMetadata::default())),
                            self.parse_pattern(named.pattern(), offset)
                        ),
                        DestructuringItem::Spread(spread) => merge!(
                            spread
                                .sink_ident()
                                .and_then(|ident| self.parse_ident(ident, offset)),
                            spread
                                .sink_expr()
                                .and_then(|expr| self.parse_expr(expr, offset))
                        ),
                    })
                    .flatten()
                    .collect(),
            ),
        }
    }

    fn parse_ident(self, ident: Ident, offset: OffsetCursor) -> Option<Vec<Token>> {
        self.parse_expr(Expr::Ident(ident), offset)
    }

    /// Do not use for spreads contained in DestructuringItem
    fn parse_spread(self, spread: Spread, offset: OffsetCursor) -> Option<Vec<Token>> {
        merge!(
            self.parse_expr(spread.expr(), offset),
            spread
                .sink_ident()
                .and_then(|ident| self.parse_ident(ident, offset))
        )
    }

    pub fn parse_expr(self, ex: Expr, offset: OffsetCursor) -> Option<Vec<Token>> {
        let offset = offset.push_to_span(ex.span());

        macro_rules! token {
            ($a:expr, $kind:expr) => {
                def_token!(self.doc, $a, $kind, offset)
            };
        }
        macro_rules! recurse {
        ($inner:expr) => {
            self.parse_expr($inner, offset)
        };
        ($($inner:expr),*) => {
            merge!(
                $(recurse!($inner)),*
            )
        };
    }

        let iter_recurse = |exprs: &mut dyn Iterator<Item = Expr>| {
            Some(exprs.filter_map(|e| recurse!(e)).flatten().collect_vec())
        };
        let parse_dict = |dict: &mut dyn Iterator<Item = DictItem>| {
            Some(
                dict.filter_map(|di| match di {
                    DictItem::Named(named) => {
                        merge!(
                            self.parse_ident(named.name(), offset),
                            recurse!(named.expr())
                        )
                    }
                    DictItem::Keyed(keyed) => recurse!(keyed.key(), keyed.expr()),
                    DictItem::Spread(spread) => self.parse_spread(spread, offset),
                })
                .flatten()
                .collect_vec(),
            )
        };
        let parse_params = |params: &mut dyn Iterator<Item = Param>| {
            Some(
                params
                    .filter_map(|p| match p {
                        Param::Pos(pattern) => self.parse_pattern(pattern, offset),
                        Param::Named(named) => merge!(
                            self.parse_ident(named.name(), offset),
                            recurse!(named.expr())
                        ),
                        Param::Spread(spread) => self.parse_spread(spread, offset),
                    })
                    .flatten()
                    .collect_vec(),
            )
        };
        let parse_args = |params: &mut dyn Iterator<Item = Arg>| {
            Some(
                params
                    .filter_map(|a| match a {
                        Arg::Pos(expr) => recurse!(expr),
                        Arg::Named(named) => merge!(
                            self.parse_ident(named.name(), offset),
                            recurse!(named.expr())
                        ),
                        Arg::Spread(spread) => self.parse_spread(spread, offset),
                    })
                    .flatten()
                    .collect_vec(),
            )
        };

        match ex {
            Expr::Text(text) => self.parse_english(text.get(), offset.push_to_span(text.span())),
            Expr::Space(a) => {
                let mut chars = self
                    .doc
                    .get(self.doc.range(a.span()).unwrap())
                    .unwrap()
                    .chars();
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
                        TokenKind::Punctuation(Punctuation::Quote(crate::Quote { twin_loc: None }))
                    )
                } else {
                    token!(quote, TokenKind::Punctuation(Punctuation::Apostrophe))
                }
            }
            Expr::Strong(strong) => iter_recurse(&mut strong.body().exprs()),
            Expr::Emph(emph) => iter_recurse(&mut emph.body().exprs()),
            Expr::Link(a) => token!(a, TokenKind::Url),
            Expr::Ref(a) => {
                token!(a, TokenKind::Word(WordMetadata::default()))
            }
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
                let offset = offset.push_to_span(text.span()).char + 1;
                let string = text.to_untyped().text();

                Some(
                    self.parser
                        .parse_str(&string[1..string.len() - 1])
                        .into_iter()
                        .map(|mut t| {
                            t.span.push_by(offset);
                            t
                        })
                        .collect_vec(),
                )
            }
            Expr::Content(content_block) => iter_recurse(&mut content_block.body().exprs()),
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
            Expr::Dict(a) => parse_dict(&mut a.items()),
            Expr::FieldAccess(field_access) => merge!(
                recurse!(field_access.target()),
                token!(
                    field_access.field(),
                    TokenKind::Word(WordMetadata::default())
                )
            ),
            Expr::Let(let_binding) => merge!(
                match let_binding.kind() {
                    LetBindingKind::Normal(pattern) => self.parse_pattern(pattern, offset),
                    LetBindingKind::Closure(ident) => self.parse_ident(ident, offset),
                },
                let_binding.init().and_then(|e| recurse!(e))
            ),
            Expr::DestructAssign(destruct_assignment) => {
                recurse!(destruct_assignment.value())
            }
            Expr::Set(set_rule) => merge!(
                recurse!(set_rule.target()),
                set_rule.condition().and_then(|expr| recurse!(expr)),
                parse_args(&mut set_rule.args().items())
            ),
            Expr::Show(show_rule) => merge!(
                recurse!(show_rule.transform()),
                show_rule.selector().and_then(|expr| recurse!(expr))
            ),
            Expr::Contextual(contextual) => recurse!(contextual.body()),
            Expr::Conditional(conditional) => merge!(
                recurse!(conditional.condition(), conditional.if_body()),
                conditional.else_body().and_then(|expr| recurse!(expr))
            ),
            Expr::While(while_loop) => recurse!(while_loop.condition(), while_loop.body()),
            Expr::For(for_loop) => recurse!(for_loop.iterable(), for_loop.body()),
            Expr::Code(code) => iter_recurse(&mut code.body().exprs()),
            Expr::Closure(closure) => merge!(
                closure
                    .name()
                    .and_then(|ident| self.parse_ident(ident, offset)),
                parse_params(&mut closure.params().children()),
                recurse!(closure.body())
            ),
            Expr::FuncCall(func) => merge!(
                token!(func.callee(), TokenKind::Unlintable),
                parse_args(&mut func.args().items())
            ),
            a => token!(a, TokenKind::Unlintable),
        }
    }
}

thread_local! {
    static WORD_APOSTROPHE_WORD: Lrc<SequencePattern> = Lrc::new(SequencePattern::default()
                .then_any_word()
                .then_apostrophe()
                .then_any_word());
}

impl Parser for Typst {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let source_str: String = source.iter().collect();
        let typst_document = Source::detached(source_str);
        let typst_tree = Markup::from_untyped(typst_document.root())
            .expect("Unable to create typst document from parsed tree!");
        let parse_helper = ParseHelper::new(&typst_document);

        let mut tokens = typst_tree
            .exprs()
            .filter_map(|ex| parse_helper.parse_expr(ex, OffsetCursor::new(&typst_document)))
            .flatten()
            .collect_vec();

        // Consolidate conjunctions
        let mut to_remove = std::collections::VecDeque::default();
        for tok_span in WORD_APOSTROPHE_WORD
            .with(|v| v.clone())
            .find_all_matches(&tokens, source)
        {
            let start_tok = &tokens[tok_span.start];
            let end_tok = &tokens[tok_span.end - 1];
            let char_span = crate::Span::new(start_tok.span.start, end_tok.span.end);

            if let TokenKind::Word(metadata) = start_tok.kind {
                tokens[tok_span.start].kind =
                    TokenKind::Word(if end_tok.span.get_content(source) == ['s'] {
                        WordMetadata {
                            noun: Some(NounData {
                                is_possessive: Some(true),
                                ..metadata.noun.unwrap_or_default()
                            }),
                            conjunction: None,
                            ..metadata
                        }
                    } else {
                        WordMetadata {
                            noun: metadata.noun.map(|noun| NounData {
                                is_possessive: Some(false),
                                ..noun
                            }),
                            conjunction: Some(ConjunctionData {}),
                            ..metadata
                        }
                    });

                tokens[tok_span.start].span = char_span;
                to_remove.extend(tok_span.start + 1..tok_span.end);
            } else {
                panic!("Apostrophe consolidation does not start with Word Token!")
            }
        }
        tokens.remove_indices(to_remove.into_iter().sorted().unique().collect());

        tokens
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use ordered_float::OrderedFloat;

    use super::Typst;
    use crate::{parsers::StrParser, NounData, Punctuation, TokenKind, WordMetadata};

    #[test]
    fn conjunction() {
        let source = "doesn't";

        let tokens = Typst.parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert_eq!(token_kinds.len(), 1);
        assert!(token_kinds.into_iter().all(|t| t.is_conjunction()))
    }

    #[test]
    fn possessive() {
        let source = "person's";

        let tokens = Typst.parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert_eq!(token_kinds.len(), 1);
        assert!(token_kinds.into_iter().all(|t| {
            matches!(
                t,
                TokenKind::Word(WordMetadata {
                    noun: Some(NounData {
                        is_possessive: Some(true),
                        ..
                    }),
                    ..
                })
            )
        }))
    }

    #[test]
    fn number() {
        let source = "12 is larger than 11, but much less than 11!";

        let tokens = Typst.parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind).collect_vec();
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

        let tokens = Typst.parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind).collect_vec();
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

        let tokens = Typst.parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        let charslice = source.chars().collect_vec();
        assert_eq!(tokens[2].span.get_content_string(&charslice), "Typst");

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Unlintable, // Ident
                TokenKind::Unlintable, // Key 1
                TokenKind::Word(_),    // Value 1
                TokenKind::Unlintable, // Key 2
                TokenKind::Unlintable, // Value 2
            ]
        ))
    }

    #[test]
    fn str_parsing() {
        let source = r#"#let ident = "This is a string""#;

        let token_kinds = Typst.parse_str(source).iter().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert!(matches!(
            &token_kinds.as_slice(),
            &[
                TokenKind::Unlintable,
                TokenKind::Word(_), // This
                TokenKind::Space(1),
                TokenKind::Word(_), // Is
                TokenKind::Space(1),
                TokenKind::Word(_), // A
                TokenKind::Space(1),
                TokenKind::Word(_), // String
            ]
        ))
    }

    #[test]
    fn non_adjacent_spaces_not_condensed() {
        let source = r#"#authors_slice.join(", ", last: ", and ")  bob"#;

        let token_kinds = Typst.parse_str(source).iter().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert!(matches!(
            &token_kinds.as_slice(),
            &[
                TokenKind::Unlintable, // authors_slice.join
                TokenKind::Punctuation(Punctuation::Comma),
                TokenKind::Space(1),
                TokenKind::Unlintable, // Ident
                TokenKind::Punctuation(Punctuation::Comma),
                TokenKind::Space(1),
                TokenKind::Word(_), // and
                TokenKind::Space(1),
                TokenKind::Space(2),
                TokenKind::Word(_),
            ]
        ))
    }

    #[test]
    fn header_parsing() {
        let source = r"= Header
                       Paragraph";

        let tokens = Typst.parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        let charslice = source.chars().collect_vec();
        assert_eq!(tokens[0].span.get_content_string(&charslice), "Header");
        assert_eq!(tokens[2].span.get_content_string(&charslice), "Paragraph");

        assert!(matches!(
            &token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::Newline(1),
                TokenKind::Word(_)
            ]
        ))
    }

    #[test]
    fn parbreak() {
        let source = r"Paragraph

                       Paragraph";

        let token_kinds = Typst.parse_str(source).iter().map(|t| t.kind).collect_vec();
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
    fn label_unlintable() {
        let source = r"= Header
                       <label>
                       Paragraph";

        let tokens = Typst.parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        assert!(matches!(
            &token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::Newline(1),
                TokenKind::Unlintable,
                TokenKind::Newline(1),
                TokenKind::Word(_),
            ]
        ))
    }

    #[test]
    fn sentence() {
        let source = "This is a sentence, it is not interesting.";

        let tokens = Typst.parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind).collect_vec();
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
        let source = r#"group’s
writing"#;

        let tokens = Typst.parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind).collect_vec();
        dbg!(&token_kinds);

        let charslice = source.chars().collect_vec();
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
}
