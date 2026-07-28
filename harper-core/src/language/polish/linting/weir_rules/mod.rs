use crate::linting::LintGroup;

/// Get the Weir rules lint group for Polish.
pub fn lint_group() -> LintGroup {
    // For now, return an empty lint group - this will be populated with actual Weir rules
    LintGroup::empty()
}
