mod dedent;
mod heuristics;

use dedent::DedentLines;
use harper_core::parsers::{self, Parser, PlainEnglish};
use harper_core::{Mask, Masker, Span, Token};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

fn is_ignored_comment(text: &str) -> bool {
    text.contains("spellchecker:ignore")
        || text.contains("spellchecker: ignore")
        || text.contains("spell-checker:ignore")
        || text.contains("spell-checker: ignore")
        || text.contains("spellcheck:ignore")
        || text.contains("spellcheck: ignore")
        || text.contains("harper:ignore")
        || text.contains("harper: ignore")
}

/// Isolates YAML `#` comments and prose-like scalar values (plain,
/// quoted, or block scalars) from the rest of a YAML document's
/// structure (keys, punctuation, enum-like values).
struct YamlMasker {
    language: tree_sitter::Language,
}

impl YamlMasker {
    fn new(language: tree_sitter::Language) -> Self {
        Self { language }
    }

    fn is_comment_node(n: &Node) -> bool {
        n.kind() == "comment"
    }

    fn is_scalar_node(n: &Node) -> bool {
        let is_candidate_kind = matches!(
            n.kind(),
            "plain_scalar" | "single_quote_scalar" | "double_quote_scalar" | "block_scalar"
        );

        is_candidate_kind && !Self::is_mapping_key(n)
    }

    /// True if `n` lies within the "key" field of its nearest enclosing
    /// `block_mapping_pair`/`flow_pair` ancestor. This uses tree-sitter's
    /// named `key` field rather than a byte-position heuristic, so it
    /// correctly handles explicit-key syntax (`? key` / `: value`), where
    /// the key node does not start at the same byte offset as the pair
    /// itself.
    fn is_mapping_key(n: &Node) -> bool {
        let n_start = n.start_byte();
        let n_end = n.end_byte();
        let mut node = *n;

        while let Some(parent) = node.parent() {
            if matches!(parent.kind(), "block_mapping_pair" | "flow_pair") {
                return match parent.child_by_field_name("key") {
                    Some(key) => key.start_byte() <= n_start && n_end <= key.end_byte(),
                    None => false,
                };
            }
            node = parent;
        }

        false
    }
}

impl Masker for YamlMasker {
    fn create_mask(&self, source: &[char]) -> Mask {
        let comment_mask =
            TreeSitterMasker::new(self.language.clone(), Self::is_comment_node).create_mask(source);
        let scalar_mask =
            TreeSitterMasker::new(self.language.clone(), Self::is_scalar_node).create_mask(source);

        let mut spans: Vec<Span<char>> = Vec::new();

        for (span, chars) in comment_mask.iter_allowed(source) {
            let text: String = chars.iter().collect();

            if !is_ignored_comment(&text) {
                spans.push(span);
            }
        }

        for (span, chars) in scalar_mask.iter_allowed(source) {
            let text: String = chars.iter().collect();

            if heuristics::is_prose_scalar(&text) {
                spans.push(span);
            }
        }

        spans.sort_by_key(|s| s.start);

        // Defensive guard: `Mask`'s `FromIterator` panics on overlapping
        // spans. The comment pass and the scalar pass are independent
        // tree-sitter queries, and under adversarial input / parser error
        // recovery they could theoretically produce overlapping spans. To
        // avoid turning that into a panic (a DoS in harper-ls), coalesce any
        // overlapping (or touching) spans by merging them, keeping all
        // prose rather than silently dropping any of it.
        let mut coalesced: Vec<Span<char>> = Vec::with_capacity(spans.len());
        for span in spans {
            if let Some(last) = coalesced.last_mut()
                && span.start <= last.end
            {
                last.end = last.end.max(span.end);
                continue;
            }
            coalesced.push(span);
        }

        coalesced.into_iter().collect()
    }
}

/// A standalone parser for YAML documents. Lints `#` comments and
/// prose-like scalar values (plain, quoted, or block scalars), while
/// leaving structural YAML (keys, identifiers, enum-like values) alone.
pub struct YamlParser {
    inner: parsers::Mask<YamlMasker, DedentLines<PlainEnglish>>,
}

impl Default for YamlParser {
    fn default() -> Self {
        Self {
            inner: parsers::Mask::new(
                YamlMasker::new(tree_sitter_yaml::LANGUAGE.into()),
                DedentLines::new(PlainEnglish),
            ),
        }
    }
}

impl Parser for YamlParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}

#[cfg(test)]
mod yaml_masker_tests {
    use harper_core::Masker;

    use super::YamlMasker;

    fn allowed_text(source: &str) -> Vec<String> {
        let chars: Vec<char> = source.chars().collect();
        let masker = YamlMasker::new(tree_sitter_yaml::LANGUAGE.into());
        let mask = masker.create_mask(&chars);

        mask.iter_allowed(&chars)
            .map(|(_, content)| content.iter().collect())
            .collect()
    }

    #[test]
    fn keeps_comment() {
        let texts = allowed_text("# a short comment\nname: value\n");
        assert_eq!(texts, vec!["# a short comment".to_string()]);
    }

    #[test]
    fn drops_comment_tagged_spellchecker_ignore_no_space() {
        let texts = allowed_text("# spellchecker:ignore an intentional splling error\n");
        assert!(texts.is_empty());
    }

    #[test]
    fn drops_comment_tagged_spellchecker_ignore_with_space() {
        let texts = allowed_text("# spellchecker: ignore an intentional splling error\n");
        assert!(texts.is_empty());
    }

    #[test]
    fn drops_comment_tagged_spell_checker_ignore() {
        let texts = allowed_text("# spell-checker:ignore an intentional splling error\n");
        assert!(texts.is_empty());
    }

    #[test]
    fn drops_comment_tagged_spellcheck_ignore() {
        let texts = allowed_text("# spellcheck:ignore an intentional splling error\n");
        assert!(texts.is_empty());
    }

    #[test]
    fn drops_comment_tagged_harper_ignore() {
        let texts = allowed_text("# harper:ignore an intentional splling error\n");
        assert!(texts.is_empty());
    }

    #[test]
    fn keeps_other_comments_on_the_same_document_as_an_ignored_one() {
        // Only the tagged comment should be suppressed - an unrelated
        // comment elsewhere in the same document must still be linted.
        let texts = allowed_text(
            "# spellchecker:ignore an intentional splling error\nname: value\n# a normal comment worth checking\n",
        );

        assert_eq!(texts, vec!["# a normal comment worth checking".to_string()]);
    }

    #[test]
    fn keeps_prose_like_plain_scalar() {
        let texts = allowed_text("description: This service handles user login and session\n");
        assert!(
            texts
                .iter()
                .any(|t| t.contains("This service handles user login"))
        );
    }

    #[test]
    fn drops_short_identifier_value() {
        let texts = allowed_text("status: enabled\n");
        assert!(texts.is_empty());
    }

    #[test]
    fn keeps_prose_like_double_quoted_scalar() {
        let texts = allowed_text("summary: \"This is a longer human readable description\"\n");
        assert!(
            texts
                .iter()
                .any(|t| t.contains("This is a longer human readable description"))
        );
    }

    #[test]
    fn keeps_block_scalar_prose() {
        let texts =
            allowed_text("notes: |\n  This is a long form block of\n  descriptive prose text.\n");
        assert!(texts.iter().any(|t| t.contains("descriptive prose text")));
    }

    #[test]
    fn drops_kebab_case_value() {
        let texts = allowed_text("mode: production-ready-config\n");
        assert!(texts.is_empty());
    }

    #[test]
    fn keeps_scalar_and_trailing_comment_on_same_line() {
        let texts = allowed_text("description: This is a fairly long value # a trailing note\n");

        // The scalar prose and the trailing comment must come back as two
        // distinct spans, not merged into a single whole-line passthrough.
        assert_eq!(texts.len(), 2);

        // The YAML key itself must never leak into either allowed span.
        assert!(texts.iter().all(|t| !t.contains("description")));

        let comment_spans: Vec<&String> = texts.iter().filter(|t| t.contains('#')).collect();
        let scalar_spans: Vec<&String> = texts.iter().filter(|t| !t.contains('#')).collect();

        assert_eq!(comment_spans.len(), 1);
        assert_eq!(scalar_spans.len(), 1);

        assert!(comment_spans[0].trim_start().starts_with('#'));
        assert!(comment_spans[0].contains("a trailing note"));
        assert!(!comment_spans[0].contains("This is a fairly long value"));

        assert!(scalar_spans[0].contains("This is a fairly long value"));
        assert!(!scalar_spans[0].contains("a trailing note"));

        let joined = texts.join(" ");
        assert!(joined.contains("This is a fairly long value"));
        assert!(joined.contains("a trailing note"));
    }

    #[test]
    fn does_not_bleed_prose_value_into_following_key() {
        // Regression test: a prose value on one line and a plain-scalar key
        // on the very next line are separated only by a newline. Before the
        // fix, YamlMasker treated the following key as a scalar candidate
        // too, so TreeSitterMasker::create_mask's whitespace-only-gap merge
        // welded the two adjacent allowed spans into one, e.g.
        // "...here\ntags". Excluding mapping keys from the scalar candidate
        // set means the gap now contains "tags:" (non-whitespace), so no
        // merge happens.
        let texts = allowed_text(
            "name: foo\nmode: bar\ndescription: This is a genuinely long prose sentence here\ntags: something\n",
        );

        // The prose value must survive as its own clean span, not welded to
        // the following key.
        assert!(
            texts
                .iter()
                .any(|t| t.contains("This is a genuinely long prose sentence here"))
        );

        // No allowed span may contain both the tail of the prose value and
        // the following key's name -- that's exactly what the bug produced.
        assert!(
            !texts
                .iter()
                .any(|t| t.contains("here") && t.contains("tags"))
        );

        // None of the short key names should ever appear bled into a kept
        // span alongside other content.
        for key in ["name", "mode", "description", "tags"] {
            assert!(
                !texts.iter().any(|t| t.contains(key)),
                "key `{key}` leaked into an allowed span: {texts:?}"
            );
        }
    }

    #[test]
    fn keeps_multi_word_quoted_key_out_of_lint_pass() {
        // Even a key that would otherwise pass the word-count prose
        // heuristic (a multi-word quoted string used as a mapping key) must
        // never be linted -- keys are not prose.
        let texts = allowed_text("\"a fairly long key name\": short value\n");
        assert!(texts.is_empty());
    }

    #[test]
    fn excludes_explicit_key_from_lint_pass() {
        // Explicit-key YAML syntax (`? key` / `: value`) puts the key node
        // in a different byte position than the pair itself, which used to
        // trip up the old byte-position heuristic and let the key bleed
        // into the lint pass. It must still be excluded, just like an
        // implicit key would be.
        let texts = allowed_text("? a genuinely long explicit key here\n: short\n");

        assert!(
            !texts
                .iter()
                .any(|t| t.contains("a genuinely long explicit key here")),
            "explicit key leaked into an allowed span: {texts:?}"
        );
    }

    #[test]
    fn block_scalar_hash_is_literal_content_not_a_comment() {
        // A `#` inside a block scalar's indented content is literal text,
        // not a real YAML comment -- the two TreeSitterMasker passes must
        // never produce overlapping spans for this, since Mask::from_iter
        // panics on overlap.
        let texts =
            allowed_text("notes: |\n  This is prose text with a # character in it for testing.\n");

        assert_eq!(texts.len(), 1);
        assert!(texts[0].contains("a # character in it for testing"));
    }
}
