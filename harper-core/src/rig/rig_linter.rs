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
    use crate::{
        Document,
        linting::{LintKind, Linter},
        rig::{Atom, CaptureGroup, Concat},
    };

    struct TestRigLinter {
        rig: Box<dyn RegexNode>,
    }

    impl RigLinter for TestRigLinter {
        fn pattern(&self) -> &dyn RegexNode {
            self.rig.as_ref()
        }

        fn match_to_lint(&self, rig_match: &RigMatch) -> Option<Lint> {
            Some(Lint {
                span: rig_match.span.to_char_span(rig_match.tokens),
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
        let mut linter = TestRigLinter { rig: pattern };

        let doc = Document::new_plain_english_curated("hello world");
        let lints = linter.lint(&doc);

        assert_eq!(lints.len(), 1);
    }

    #[test]
    fn test_rig_linter_with_capture() {
        let pattern = Box::new(CaptureGroup::new(0, Box::new(Atom::word("hello"))));
        let mut linter = TestRigLinter { rig: pattern };

        let doc = Document::new_plain_english_curated("hello world");
        let lints = linter.lint(&doc);

        assert_eq!(lints.len(), 1);
    }

    #[test]
    fn test_rig_linter_concat() {
        let pattern = Box::new(Concat::new([
            Box::new(Atom::any()) as Box<dyn RegexNode>,
            Box::new(Atom::any()),
        ]));
        let mut linter = TestRigLinter { rig: pattern };

        let doc = Document::new_plain_english_curated("hello world");
        let lints = linter.lint(&doc);

        assert_eq!(lints.len(), 1);
    }
}
