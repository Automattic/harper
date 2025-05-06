use std::{collections::HashMap, fmt::Display};

use harper_core::linting::{LintGroupConfig, LintKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Summary {
    pub lint_counts: HashMap<LintKind, u32>,
    pub total_applied: u32,
    pub final_config: LintGroupConfig,
    // The most common misspelled words.
    pub misspelled: HashMap<String, u32>,
}

impl Summary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the count for a particular lint kind.
    pub fn inc_lint_count(&mut self, kind: LintKind) {
        self.lint_counts
            .entry(kind)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
        self.total_applied += 1;
    }

    /// Increment the count for a particular misspelled word.
    pub fn inc_misspelled_count(&mut self, word: impl AsRef<str>) {
        if let Some(counter) = self.misspelled.get_mut(word.as_ref()) {
            *counter += 1
        } else {
            self.misspelled.insert(word.as_ref().to_owned(), 1);
        }
    }

    /// Get the count for a particular lint kind.
    pub fn get_count(&self, kind: LintKind) -> u32 {
        self.lint_counts.get(&kind).copied().unwrap_or(0)
    }
}

impl Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "`LintKind` counts")?;
        writeln!(f, "=================")?;

        for (kind, count) in &self.lint_counts {
            writeln!(f, "{kind}\t{count}")?;
        }

        writeln!(f, "Misspelling counts")?;
        writeln!(f, "=================")?;

        let mut misspelled: Vec<_> = self
            .misspelled
            .iter()
            .map(|(a, b)| (a.clone(), *b))
            .collect();

        misspelled.sort_by_key(|(_a, b)| u32::MAX - b);

        for (kind, count) in &misspelled {
            writeln!(f, "{kind}\t{count}")?;
        }

        Ok(())
    }
}
