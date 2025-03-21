mod lint_record;
mod lint_summary;

use std::io::{self, Read, Write};

pub use lint_record::LintRecord;
pub use lint_summary::LintSummary;

/// A collection of logged statistics for the various Harper frontends.
pub struct Stats {
    /// A record of the lints the user has applied.
    lints_applied: Vec<LintRecord>,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            lints_applied: Vec::new(),
        }
    }

    pub fn lint_applied(&mut self, record: LintRecord) {
        self.lints_applied.push(record);
    }

    /// Count the number of each kind of lint applied.
    pub fn summarize_lints_applied(&self) -> LintSummary {
        let mut summary = LintSummary::new();

        for lint in &self.lints_applied {
            summary.inc(lint.kind);
        }

        summary
    }

    pub fn write_csv(&self, w: &mut impl Write) -> io::Result<()> {
        let mut writer = csv::WriterBuilder::new().has_headers(false).from_writer(w);

        for record in &self.lints_applied {
            writer.serialize(record)?;
        }

        Ok(())
    }

    pub fn read_csv(r: &mut impl Read) -> io::Result<Self> {
        let mut reader = csv::ReaderBuilder::new().has_headers(false).from_reader(r);
        let mut records = Vec::new();

        for result in reader.deserialize() {
            let record: LintRecord = result?;

            records.push(record);
        }

        Ok(Self {
            lints_applied: records,
        })
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}
