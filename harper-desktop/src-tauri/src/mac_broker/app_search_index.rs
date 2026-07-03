use std::collections::BTreeSet;

use crate::os_broker::AppSearchResult;

use super::app_catalog::{
    self, app_search_result_from_bundle_id, installed_application_bundle_ids,
};

pub struct AppSearchIndex {
    index: Vec<AppSearchResult>,
}

impl AppSearchIndex {
    pub fn new() -> Self {
        Self { index: Vec::new() }
    }

    pub fn populate(&mut self) -> Result<(), String> {
        let installed_bundles = installed_application_bundle_ids()?;

        let mut seen_bundle_ids = BTreeSet::new();

        for bundle_id in installed_bundles {
            let line = bundle_id.trim();

            if !line.ends_with(".app") {
                continue;
            }

            let result = app_search_result_from_bundle_id(&bundle_id);

            if seen_bundle_ids.insert(result.bundle_id.clone()) {
                self.index.push(result);
            }
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn search(&self, query: &str) -> Vec<AppSearchResult> {
        let query = query.trim();

        if query.is_empty() {
            return self.index.to_vec();
        }

        if let Some(result) = self
            .index
            .iter()
            .find(|result| result.bundle_id == query)
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
