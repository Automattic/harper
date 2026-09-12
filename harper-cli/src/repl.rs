use std::io::{self, BufRead, IsTerminal, Write};
use std::sync::Arc;

use ariadne::{Color, Label, Report, ReportKind, Source};
use harper_core::{
    Dialect, Document, linting::LintGroup, remove_overlaps_map, spell::FstDictionary,
};

use crate::lint::configure_lint_group;

pub fn repl(
    dictionary: Arc<FstDictionary>,
    dialect: Dialect,
    mut only: Option<Vec<String>>,
    color: bool,
) -> anyhow::Result<()> {
    let mut lint_group = LintGroup::new_curated(dictionary.clone(), dialect);
    if let Some(rules) = &mut only {
        rules.retain(|rule| {
            if lint_group.config.has_rule(rule) {
                true
            } else {
                eprintln!("Warning: Cannot enable unknown rule '{}'.", rule);
                false
            }
        });
    }
    configure_lint_group(&mut lint_group, &only, &None);

    let stdin = io::stdin();
    let interactive = stdin.is_terminal();
    Session {
        dictionary,
        lint_group,
        color,
    }
    .run(
        stdin.lock(),
        io::stdout().lock(),
        io::stderr().lock(),
        interactive,
    )
}

struct Session {
    dictionary: Arc<FstDictionary>,
    lint_group: LintGroup,
    color: bool,
}

impl Session {
    fn run(
        &mut self,
        mut input: impl BufRead,
        mut output: impl Write,
        mut prompts: impl Write,
        interactive: bool,
    ) -> anyhow::Result<()> {
        if interactive {
            writeln!(
                prompts,
                "Enter text to lint. Press Ctrl-C or send EOF to exit."
            )?;
        }

        let mut line = String::new();
        loop {
            if interactive {
                write!(prompts, "> ")?;
                prompts.flush()?;
            }
            line.clear();
            if input.read_line(&mut line)? == 0 {
                return Ok(());
            }
            // Remove only the line terminator: other whitespace is part of the text to lint.
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }
            if line.trim().is_empty() {
                continue;
            }

            let doc = Document::new_plain_english(&line, &self.dictionary);
            let mut named_lints = self.lint_group.organized_lints(&doc);
            remove_overlaps_map(&mut named_lints);
            let mut lints: Vec<_> = named_lints
                .iter()
                .flat_map(|(rule, lints)| lints.iter().map(move |lint| (rule, lint)))
                .collect();
            lints.sort_by_key(|(rule, lint)| (lint.span.start, lint.span.end, *rule));

            if lints.is_empty() {
                writeln!(output, "No lints found.")?;
            }
            let source = Source::from(line.as_str());
            for (rule, lint) in lints {
                let mut report =
                    Report::build(ReportKind::Advice, ("repl", lint.span.start..lint.span.end))
                        .with_config(ariadne::Config::default().with_color(self.color))
                        .with_message(format!("[{}::{}]", lint.lint_kind, rule))
                        .with_label(
                            Label::new(("repl", lint.span.into()))
                                .with_message(&lint.message)
                                .with_color(Color::Magenta),
                        );
                for suggestion in &lint.suggestions {
                    report = report.with_help(suggestion);
                }
                report
                    .finish()
                    .write_for_stdout(("repl", &source), &mut output)?;
            }
            // A caller piping a long-lived input must see results before sending another line.
            output.flush()?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Session {
        let dictionary = FstDictionary::curated();
        let mut lint_group = LintGroup::new_curated(dictionary.clone(), Dialect::American);
        configure_lint_group(&mut lint_group, &Some(vec!["AnA".into()]), &None);
        Session {
            dictionary,
            lint_group,
            color: false,
        }
    }

    #[test]
    fn interactive_prompts_are_separate_from_results() {
        let mut output = Vec::new();
        let mut prompts = Vec::new();
        session()
            .run(
                "\nThis is a test.\n".as_bytes(),
                &mut output,
                &mut prompts,
                true,
            )
            .unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "No lints found.\n");
        let prompts = String::from_utf8(prompts).unwrap();
        assert!(prompts.contains("Ctrl-C"));
        assert!(prompts.ends_with("> > > "));
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("write failed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("flush failed"))
        }
    }

    #[test]
    fn output_errors_are_not_swallowed() {
        for text in ["This is a test.\n", "This is an test.\n"] {
            assert!(
                session()
                    .run(text.as_bytes(), FailingWriter, io::sink(), false)
                    .is_err()
            );
        }
    }

    #[test]
    fn prompt_errors_are_propagated() {
        assert!(
            session()
                .run(io::empty(), io::sink(), FailingWriter, true)
                .is_err()
        );
    }

    #[test]
    fn invalid_utf8_is_reported() {
        assert!(
            session()
                .run(&b"\xff\n"[..], io::sink(), io::sink(), false)
                .is_err()
        );
    }
}
