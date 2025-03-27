mod record;
mod summary;

use std::io::{self, Read, Write};
use std::io::{BufRead, BufReader};

pub use record::Record;
pub use record::RecordKind;
use serde::Serialize;
use serde_json::Serializer;
pub use summary::Summary;

/// A collection of logged statistics for the various Harper frontends.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Stats {
    pub records: Vec<Record>,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Count the number of each kind of lint applied.
    pub fn summarize(&self) -> Summary {
        let mut summary = Summary::new();

        for record in &self.records {
            match &record.kind {
                RecordKind::Lint(lint_kind) => summary.inc_lint_count(*lint_kind),
                RecordKind::LintConfigUpdate(lint_group_config) => {
                    summary.final_config = lint_group_config.clone();
                }
            }
        }

        summary
    }

    /// Write the records from `self`.
    /// Expects the target buffer to either be empty or already be terminated by a newline.
    pub fn write(&self, w: &mut impl Write) -> io::Result<()> {
        for record in &self.records {
            let mut serializer = Serializer::new(&mut *w);
            record.serialize(&mut serializer)?;
            writeln!(w)?;
        }

        Ok(())
    }

    /// Read records from a buffer into `self`.
    /// Assumes the buffer is properly formatted and terminated with a newline.
    /// An empty buffer will result in no mutation to `self`.
    pub fn read(r: &mut impl Read) -> io::Result<Self> {
        let br = BufReader::new(r);
        let mut records = Vec::new();

        for line_res in br.lines() {
            let line = line_res?;

            let record: Record = serde_json::from_str(&line)?;
            records.push(record);
        }

        Ok(Self { records })
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;

    use crate::{Record, Stats};

    impl Arbitrary for Stats {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut stats = Stats::new();

            for _ in 0..g.size() {
                stats.records.push(Record::arbitrary(g));
            }

            stats
        }
    }

    #[quickcheck]
    fn io_is_reversible(ex: Stats) -> bool {
        let mut written = Vec::new();

        ex.write(&mut written).unwrap();

        let mut readable = Cursor::new(written);

        let read = Stats::read(&mut readable).unwrap();

        ex == read
    }
}
