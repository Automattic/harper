use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

use harper_core::linting::LintKind;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct LintRecord {
    pub kind: LintKind,
    /// Recorded as seconds from the Unix Epoch
    pub when: u64,
    pub uuid: Uuid,
}

impl LintRecord {
    /// Record a new instance at the current system time.
    pub fn record_now(kind: LintKind) -> Result<Self, SystemTimeError> {
        Ok(Self {
            kind,
            when: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            uuid: Uuid::new_v4(),
        })
    }
}
