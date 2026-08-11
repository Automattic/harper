use crate::os_broker::AppSearchResult;

use super::app_catalog::installed_applications;

pub struct AppSearchIndex {
    index: Vec<AppSearchResult>,
    initialized: bool,
}

impl AppSearchIndex {
    pub fn new() -> Self {
        Self {
            index: Vec::new(),
            initialized: false,
        }
    }

    pub fn populate(&mut self) -> Result<(), String> {
        self.populate_with(installed_applications)
    }

    fn populate_with(
        &mut self,
        load_applications: impl FnOnce() -> Result<Vec<AppSearchResult>, String>,
    ) -> Result<(), String> {
        if self.initialized {
            return Ok(());
        }

        self.index = load_applications()?;
        self.initialized = true;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Vec<AppSearchResult> {
        let query = query.trim();

        if query.is_empty() {
            return self.index.to_vec();
        }

        if let Some(result) = self
            .index
            .iter()
            .find(|result| result.bundle_id.eq_ignore_ascii_case(query))
            .cloned()
        {
            return vec![result];
        }

        let lower_query = query.to_lowercase();
        self.index
            .iter()
            .filter(|result| {
                result.name.to_lowercase().contains(&lower_query)
                    || result.bundle_id.to_lowercase().contains(&lower_query)
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    fn applications() -> Vec<AppSearchResult> {
        vec![
            AppSearchResult {
                name: "Google Chrome".to_string(),
                bundle_id: "com.google.Chrome".to_string(),
            },
            AppSearchResult {
                name: "Firefox".to_string(),
                bundle_id: "org.mozilla.firefox".to_string(),
            },
            AppSearchResult {
                name: "Safari".to_string(),
                bundle_id: "com.apple.Safari".to_string(),
            },
            AppSearchResult {
                name: "TextEdit".to_string(),
                bundle_id: "com.apple.TextEdit".to_string(),
            },
        ]
    }

    #[test]
    fn searches_names_and_bundle_ids_case_insensitively() {
        let mut index = AppSearchIndex::new();
        index.populate_with(|| Ok(applications())).unwrap();

        assert_eq!(index.search("textedit"), vec![applications()[3].clone()]);
        assert_eq!(
            index.search("COM.GOOGLE.CHROME"),
            vec![applications()[0].clone()]
        );
        assert_eq!(index.search("FIREFOX"), vec![applications()[1].clone()]);
        assert_eq!(
            index.search("apple.safari"),
            vec![applications()[2].clone()]
        );
    }

    #[test]
    fn successful_empty_population_is_not_repeated() {
        let calls = Cell::new(0);
        let mut index = AppSearchIndex::new();

        index
            .populate_with(|| {
                calls.set(calls.get() + 1);
                Ok(Vec::new())
            })
            .unwrap();
        index
            .populate_with(|| {
                calls.set(calls.get() + 1);
                Ok(applications())
            })
            .unwrap();

        assert_eq!(calls.get(), 1);
        assert!(index.search("").is_empty());
    }

    #[test]
    fn failed_population_can_retry() {
        let calls = Cell::new(0);
        let mut index = AppSearchIndex::new();

        assert!(
            index
                .populate_with(|| {
                    calls.set(calls.get() + 1);
                    Err("Spotlight failed".to_string())
                })
                .is_err()
        );
        index
            .populate_with(|| {
                calls.set(calls.get() + 1);
                Ok(applications())
            })
            .unwrap();

        assert_eq!(calls.get(), 2);
        assert_eq!(index.search("Safari"), vec![applications()[2].clone()]);
    }
}
