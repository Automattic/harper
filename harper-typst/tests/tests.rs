use itertools::Itertools;
use ordered_float::OrderedFloat;

use harper_core::{DictWordMetadata, Document, NounData, Number, Punctuation, TokenKind};
use harper_typst::Typst;

#[test]
fn number() {
    let source = "12 is larger than 11, but much less than 11!";

    let document = Document::new_curated(source, &Typst);
    let token_kinds = document.tokens().map(|t| t.kind.clone()).collect_vec();
    dbg!(&token_kinds);

    assert!(matches!(
        token_kinds.as_slice(),
        &[
            TokenKind::Number(Number {
                value: OrderedFloat(12.0),
                suffix: None,
                ..
            }),
            TokenKind::Space(1),
            TokenKind::Word(_),
            TokenKind::Space(1),
            TokenKind::Word(_),
            TokenKind::Space(1),
            TokenKind::Word(_),
            TokenKind::Space(1),
            TokenKind::Number(Number {
                value: OrderedFloat(11.0),
                suffix: None,
                ..
            }),
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
            TokenKind::Number(Number {
                value: OrderedFloat(11.0),
                suffix: None,
                ..
            }),
            TokenKind::Punctuation(Punctuation::Bang),
        ]
    ))
}

#[test]
fn math_unlintable() {
    let source = "$12 > 11$, $12 << 11!$";

    let document = Document::new_curated(source, &Typst);
    let token_kinds = document.tokens().map(|t| t.kind.clone()).collect_vec();
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
    let token_kinds = document.tokens().map(|t| t.kind.clone()).collect_vec();
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
    let token_kinds = document.tokens().map(|t| t.kind.clone()).collect_vec();
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
    let token_kinds = document.tokens().map(|t| t.kind.clone()).collect_vec();
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
    let token_kinds = document.tokens().map(|t| t.kind.clone()).collect_vec();
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
    let token_kinds = document.tokens().map(|t| t.kind.clone()).collect_vec();
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
    let token_kinds = document.tokens().map(|t| t.kind.clone()).collect_vec();
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
    let token_kinds = document.tokens().map(|t| t.kind.clone()).collect_vec();
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
    let token_kinds = document.tokens().map(|t| t.kind.clone()).collect_vec();
    dbg!(&token_kinds);

    let charslice = source.chars().collect_vec();
    let tokens = document.tokens().collect_vec();
    assert_eq!(tokens[2].span.get_content_string(&charslice), "writing");

    assert!(matches!(
        token_kinds.as_slice(),
        &[
            TokenKind::Word(Some(DictWordMetadata {
                noun: Some(NounData {
                    is_possessive: Some(true),
                    ..
                }),
                ..
            })),
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
    let token_kinds = document.tokens().map(|t| t.kind.clone()).collect_vec();
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
    let token_kinds = document.tokens().map(|t| t.kind.clone()).collect_vec();
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
