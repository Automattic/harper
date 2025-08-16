//! This test creates snapshots of the reports of all linters.
//!
//! # Usage
//!
//! To add a new snapshot, simply add the document to `tests/text` and run this
//! test. It will automatically create a new snapshot in `tests/text/linters`.
//! To update an existing snapshot, also just run this test.
//!
//! Note: This test will fail if the snapshot files are not up to date. This
//! ensures that CI will fail if linters change their behavior.

use harper_core::spell::FstDictionary;
use harper_core::{
    Dialect, Document,
    linting::{LintGroup, Linter},
};

mod snapshot;

struct LinePos {
    /// 0-based index of the line
    pub line: usize,
    /// 0-based index of the column
    pub col: usize,
}

struct Lines<'a> {
    lines: Vec<&'a str>,
    offsets: Vec<usize>,
}
impl Lines<'_> {
    fn new(source: &str) -> Lines<'_> {
        let lines: Vec<&str> = source.split('\n').collect();
        let offsets: Vec<usize> = lines
            .iter()
            .scan(0, |offset, line| {
                let old_offset = *offset;
                *offset += line.chars().count() + 1;
                Some(old_offset)
            })
            .collect();

        Lines { lines, offsets }
    }

    fn len(&self) -> usize {
        self.lines.len()
    }

    fn get_pos(&self, offset: usize) -> LinePos {
        let line_index = self
            .offsets
            .binary_search(&offset)
            .unwrap_or_else(|x| x - 1);

        LinePos {
            line: line_index,
            col: offset - self.offsets[line_index],
        }
    }
}
impl<'a> std::ops::Index<usize> for Lines<'a> {
    type Output = &'a str;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lines[index]
    }
}

fn print_error(lines: &Lines, start: usize, end: usize, message: &str) -> String {
    let mut out = String::new();

    fn print_line(out: &mut String, line: &str, number: usize) {
        out.push_str(&format!("{number:>6} | {line}\n"));
    }

    fn is_sentence_boundary(c: char) -> bool {
        matches!(c, '.' | '?' | '!' | ':' | ';')
    }
    fn print_pre_line_context(
        out: &mut String,
        context_line: &str,
        number: usize,
        line: &str,
        start_col: usize,
    ) {
        if context_line.is_empty() {
            return;
        }
        if start_col > 40 {
            // that's enough context
            return;
        }

        let last_char = context_line.chars().last().unwrap();
        let mut chars_before = line.chars().take(start_col);
        if !is_sentence_boundary(last_char) && !chars_before.any(is_sentence_boundary) {
            print_line(out, context_line, number);
        }
    }
    fn print_post_line_context(
        out: &mut String,
        context_line: &str,
        number: usize,
        line: &str,
        end_col: usize,
    ) {
        if context_line.is_empty() {
            return;
        }
        if end_col < 40 {
            // that's enough context
            return;
        }

        let mut chars_after = line.chars().skip(end_col);
        if !chars_after.any(is_sentence_boundary) {
            print_line(out, context_line, number);
        }
    }

    fn print_underline(
        out: &mut String,
        start_col: usize,
        end_col: usize,
        continuation: bool,
        message: &str,
    ) {
        out.push_str("       | ");
        for _ in 0..start_col {
            out.push(' ');
        }
        out.push(if continuation { '~' } else { '^' });
        for _ in 0..end_col.saturating_sub(start_col) {
            out.push('~');
        }

        if !message.is_empty() {
            out.push(' ');
            out.push_str(message);
        }
        out.push('\n');
    }

    let start = lines.get_pos(start);
    let end = lines.get_pos(end - 1);

    if start.line > 0 {
        print_pre_line_context(
            &mut out,
            lines[start.line - 1],
            start.line,
            lines[start.line],
            start.col,
        );
    }

    if start.line == end.line {
        print_line(&mut out, lines[start.line], start.line + 1);
        print_underline(&mut out, start.col, end.col, false, message);
    } else {
        for i in start.line..end.line {
            let line = lines[i];
            print_line(&mut out, line, i + 1);
            print_underline(
                &mut out,
                if i == start.line { start.col } else { 0 },
                line.chars().count(),
                i != start.line,
                "",
            );
        }

        print_line(&mut out, lines[end.line], end.line + 1);
        print_underline(&mut out, 0, end.col, true, message);
    }

    if end.line + 1 < lines.len() {
        print_post_line_context(
            &mut out,
            lines[end.line + 1],
            end.line + 2,
            lines[end.line],
            end.col,
        );
    }

    out
}

#[test]
fn test_most_lints() {
    snapshot::snapshot_all_text_files("linters", ".snap.yml", |source, dialect_override| {
        let dict = FstDictionary::curated();
        let document = Document::new_markdown_default(source, &dict);

        let mut linter = LintGroup::new_curated(
            dict,
            dialect_override.unwrap_or_else(|| {
                Dialect::try_guess_from_document(&document).unwrap_or(Dialect::American)
            }),
        );

        let mut lints = linter.lint(&document);
        lints.sort_by(|a, b| {
            a.span
                .start
                .cmp(&b.span.start)
                .then(a.span.end.cmp(&b.span.end))
        });

        // split the input document into lines
        let lines = Lines::new(source);

        let mut out = String::new();

        for lint in lints {
            out.push_str(&format!(
                "Lint:    {:?} ({} priority)\n",
                lint.lint_kind, lint.priority
            ));

            let message = print_error(&lines, lint.span.start, lint.span.end, &lint.message);
            out.push_str("Message: |\n");
            for l in message.lines() {
                out.push_str("  ");
                out.push_str(l);
                out.push('\n');
            }

            if !lint.suggestions.is_empty() {
                out.push_str("Suggest:\n");
                for suggestion in &lint.suggestions {
                    out.push_str(&format!("  - {suggestion}\n"));
                }
            }

            out.push_str("\n\n\n");
        }

        out
    });
}
