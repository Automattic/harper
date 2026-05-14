use harper_core::linting::Lint;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

#[derive(Default)]
pub(crate) struct DebounceState {
    last_observed_text: Option<String>,
    last_text_change: Option<Instant>,
    last_linted_text: Option<String>,
    last_lints: BTreeMap<String, Vec<Lint>>,
}

impl DebounceState {
    pub(crate) fn status(&mut self, text: &str, debounce_ms: u64) -> DebounceStatus {
        if debounce_ms == 0 {
            return DebounceStatus::Ready;
        }

        let now = Instant::now();

        if self.last_observed_text.as_deref() != Some(text) {
            self.last_observed_text = Some(text.to_string());
            self.last_text_change = Some(now);
            self.last_linted_text = None;
            self.last_lints.clear();

            return DebounceStatus::Waiting;
        }

        if self.last_linted_text.as_deref() == Some(text) {
            return DebounceStatus::Cached(self.last_lints.clone());
        }

        let Some(last_text_change) = self.last_text_change else {
            self.last_text_change = Some(now);
            return DebounceStatus::Waiting;
        };

        if now.duration_since(last_text_change) < Duration::from_millis(debounce_ms) {
            return DebounceStatus::Waiting;
        }

        DebounceStatus::Ready
    }

    pub(crate) fn store_lints(
        &mut self,
        text: &str,
        debounce_ms: u64,
        lints: &BTreeMap<String, Vec<Lint>>,
    ) {
        if debounce_ms == 0 {
            return;
        }

        self.last_linted_text = Some(text.to_string());
        self.last_lints = lints.clone();
    }
}

pub(crate) enum DebounceStatus {
    Ready,
    Waiting,
    Cached(BTreeMap<String, Vec<Lint>>),
}
