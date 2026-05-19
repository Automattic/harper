use crate::{Document, LSend, Lint, TokenStringExt};

use super::{RegexNode, RigMatch};

/// A trait for linters that use Rig patterns with capture support.
///
/// This is analogous to ExprLinter, but with stateful matching that preserves
/// capture group information.
///
/// Note: Unlike ExprLinter, RigLinter does NOT provide a blanket implementation
/// of Linter to avoid conflicts. You must implement Linter manually and use the
/// provided helper functions.
pub trait RigLinter: LSend {
    /// Returns the Rig pattern to match against.
    fn pattern(&self) -> &dyn RegexNode;

    /// Transform a Rig match into a Lint.
    ///
    /// Unlike ExprLinter's match_to_lint which receives only raw token slices,
    /// this receives a RigMatch with capture information.
    fn match_to_lint(&self, rig_match: &RigMatch) -> Option<Lint>;

    /// A user-facing description of what this linter checks for.
    fn description(&self) -> &str;
}

/// Helper function to run a RigLinter over a document.
///
/// This can be used in your manual Linter implementation.
pub fn run_rig_linter<T>(linter: &mut T, document: &Document) -> Vec<Lint>
where
    T: RigLinter,
{
    let mut lints = Vec::new();
    let pattern = linter.pattern();
    let source = document.get_source();

    // Iterate over chunks (similar to ExprLinter with Chunk unit)
    for chunk in document.iter_chunks() {
        let tokens = chunk;

        let mut idx = 0;
        while idx < tokens.len() {
            if let Some(result) = pattern.exec(tokens, source, idx) {
                let span = crate::Span::new(idx, idx + result.tokens_consumed);

                let rig_match = RigMatch {
                    span,
                    captures: result.captures,
                    tokens,
                    source,
                };

                if let Some(lint) = linter.match_to_lint(&rig_match) {
                    lints.push(lint);
                }

                // Advance cursor past the match
                idx += result.tokens_consumed.max(1);
            } else {
                idx += 1;
            }
        }
    }

    lints
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::linting::LintKind;
    use crate::linting::Linter;
    use crate::rig::{Atom, CaptureGroup, Concat};

    struct TestRigLinter {
        pattern: Box<dyn RegexNode>,
    }

    impl RigLinter for TestRigLinter {
        fn pattern(&self) -> &dyn RegexNode {
            self.pattern.as_ref()
        }

        fn match_to_lint(&self, rig_match: &RigMatch) -> Option<Lint> {
            // Convert token span to character span
            let char_span = if rig_match.span.start < rig_match.tokens.len() {
                let start_char = rig_match.tokens[rig_match.span.start].span.start;
                let end_char = rig_match.tokens[rig_match.span.end.saturating_sub(1)]
                    .span
                    .end;
                crate::Span::new(start_char, end_char)
            } else {
                crate::Span::new(0, 0)
            };

            Some(Lint {
                span: char_span,
                lint_kind: LintKind::Miscellaneous,
                message: "Test match".to_string(),
                suggestions: vec![],
                ..Default::default()
            })
        }

        fn description(&self) -> &str {
            "Test RigLinter"
        }
    }

    // Manual Linter implementation using the helper function
    impl Linter for TestRigLinter {
        fn lint(&mut self, document: &Document) -> Vec<Lint> {
            run_rig_linter(self, document)
        }

        fn description(&self) -> &str {
            RigLinter::description(self)
        }
    }

    #[test]
    fn test_rig_linter_basic() {
        let pattern = Box::new(Atom::word("hello"));
        let mut linter = TestRigLinter { pattern };

        let doc = Document::new_plain_english_curated("hello world");
        let lints = linter.lint(&doc);

        assert_eq!(lints.len(), 1);
    }

    #[test]
    fn test_rig_linter_with_capture() {
        let pattern = Box::new(CaptureGroup::new(0, Box::new(Atom::word("hello"))));
        let mut linter = TestRigLinter { pattern };

        let doc = Document::new_plain_english_curated("hello world");
        let lints = linter.lint(&doc);

        assert_eq!(lints.len(), 1);
    }

    #[test]
    fn test_rig_linter_concat() {
        let pattern = Box::new(Concat::new(vec![
            Box::new(Atom::any()),
            Box::new(Atom::any()),
        ]));
        let mut linter = TestRigLinter { pattern };

        let doc = Document::new_plain_english_curated("hello world");
        let lints = linter.lint(&doc);

        assert_eq!(lints.len(), 1);
    }
}
