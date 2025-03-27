use std::{collections::HashMap, fmt::Display};

use harper_core::linting::{LintGroupConfig, LintKind};

pub struct Summary {
    pub lint_counts: HashMap<LintKind, u32>,
    pub final_config: LintGroupConfig,
}

impl Summary {
    pub fn new() -> Self {
        Self {
            lint_counts: HashMap::new(),
            final_config: LintGroupConfig::default(),
        }
    }

    /// Increment the count for a particular lint kind.
    pub fn inc_lint_count(&mut self, kind: LintKind) {
        self.lint_counts
            .entry(kind)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
    }

    /// Get the count for a particular lint kind.
    pub fn get_count(&self, kind: LintKind) -> u32 {
        self.lint_counts.get(&kind).copied().unwrap_or(0)
    }
}

impl Default for Summary {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (kind, count) in &self.lint_counts {
            writeln!(f, "{kind}\t{count}")?;
        }

        Ok(())
    }
}
