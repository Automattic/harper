use harper_core::linting::LintGroup;
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};

#[test]
fn leading_zero_year() {
    let mut group = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);
    for (input, expected) in [
        ("It happened in 01979.", Some("It happened in 1979.")),
        ("It happened in 02012.", Some("It happened in 2012.")),
        (
            "Since 002012, things have changed.",
            Some("Since 2012, things have changed."),
        ),
        ("01700", Some("1700")),
        ("02100", Some("2100")),
        ("January 1, 02012", Some("January 1, 2012")),
        ("(01979)", Some("(1979)")),
        ("It happened in 1979.", None),
        ("It happened in 2012.", None),
        ("01699", None),
        ("02101", None),
        ("0979", None),
        ("02", None),
        ("01979.0", None),
        ("01979e0", None),
        ("0x07bb", None),
        ("02012th", None),
        ("AB01979", None),
        ("01979B", None),
        ("01979_widget", None),
    ] {
        let doc = Document::new_markdown_default_curated(input);
        let mut by_rule = group.organized_lints(&doc);
        let lints = by_rule.remove("LeadingZeroYear").unwrap_or_default();
        assert_eq!(lints.len(), usize::from(expected.is_some()), "{input}");
        if let Some(expected) = expected {
            let mut output = doc.get_source().to_vec();
            lints[0].suggestions[0].apply(lints[0].span, &mut output);
            assert_eq!(output.iter().collect::<String>(), expected, "{input}");
        }
    }
}
