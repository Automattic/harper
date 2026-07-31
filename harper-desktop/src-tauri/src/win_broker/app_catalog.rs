use crate::os_broker::AppSearchResult;

#[allow(dead_code)]
pub fn installed_application_ids() -> Result<Vec<String>, String> {
    Err("App catalog not yet implemented on Windows.".to_string())
}

pub fn app_display_name(app_id: &str) -> String {
    app_id.to_owned()
}

pub fn search_apps(_query: &str) -> Result<Vec<AppSearchResult>, String> {
    Ok(Vec::new())
}
