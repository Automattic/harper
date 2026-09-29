//! Desktop updates belong to the main process, not to any particular WebView.

use crate::config::Config;
use serde::Serialize;
use std::{
    future::Future,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_updater::UpdaterExt;
use tokio::sync::Mutex;

const LATEST_VERSION_URL: &str = "https://writewithharper.com/latestversion";
const DAY_MS: u64 = 24 * 60 * 60 * 1000;
const POLL_INTERVAL: Duration = Duration::from_secs(60);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Outcome of an attempted update, using the settings UI's existing status names.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateStatus {
    UpToDate,
    Updated,
    Error,
}

/// IPC-safe update outcome. An installed update may still require an app restart.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResult {
    pub status: UpdateStatus,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub message: String,
    pub error: Option<String>,
}

impl UpdateResult {
    fn error(error: String) -> Self {
        Self {
            status: UpdateStatus::Error,
            current_version: None,
            latest_version: None,
            message: format!("Unable to check for updates: {error}"),
            error: Some(error),
        }
    }
}

/// Serializes manual and automatic updates and remembers an installation until restart.
/// The mutex covers the entire attempt, but the separate config lock never covers network I/O.
#[derive(Default)]
pub struct DesktopUpdater {
    installed_update: Mutex<Option<UpdateResult>>,
}

impl DesktopUpdater {
    /// Read the running version from packaged metadata, even after installing an update.
    pub fn current_version<R: Runtime>(app: &AppHandle<R>) -> String {
        normalize_version(&app.package_info().version.to_string())
    }

    /// Fetch display-only release information without invoking the signed updater flow.
    pub async fn latest_version() -> Result<String, String> {
        let response = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| error.to_string())?
            .get(LATEST_VERSION_URL)
            .header(reqwest::header::ACCEPT, "text/plain")
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| format!("Unable to get latest version: {error}"))?;
        let text = response
            .text()
            .await
            .map_err(|error| format!("Unable to get latest version: {error}"))?;
        Ok(normalize_version(&text))
    }

    /// Check and install an update. Only automatic requests can be skipped (returning `None`).
    /// Record attempts before network I/O so failed checks also respect the daily interval.
    /// Manual requests bypass both the preference and interval, but share installation state.
    pub async fn update_to_latest<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        automatic: bool,
    ) -> Option<UpdateResult> {
        self.run_update(|| async {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_millis() as u64;

            {
                let config = app.state::<Arc<Mutex<Config>>>();
                let mut config = config.lock().await;
                if automatic
                    && !should_check_for_update(config.auto_update, config.last_update_check, now)
                {
                    return Ok(None);
                }

                let previous = config.last_update_check;
                config.last_update_check = Some(now);
                if let Err(error) = config.save_to_system().await {
                    config.last_update_check = previous;
                    return Err(format!("Unable to save update-check timestamp: {error}"));
                }
            }

            check_and_install(app).await.map(Some)
        })
        .await
    }

    /// Hold one installation lock and cache only successful installations.
    async fn run_update<F, Fut>(&self, operation: F) -> Option<UpdateResult>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Option<UpdateResult>, String>>,
    {
        let mut installed = self.installed_update.lock().await;
        if let Some(result) = installed.as_ref() {
            return Some(result.clone());
        }

        let result = match operation().await {
            Ok(result) => result?,
            Err(error) => UpdateResult::error(error),
        };
        if result.status == UpdateStatus::Updated {
            *installed = Some(result.clone());
        }
        Some(result)
    }
}

/// Start once after plugin initialization. The first tick is immediate; subsequent ticks only
/// check eligibility, not the network. Skip missed ticks after sleep rather than catching up.
/// This task lives with the main process, including when all windows are closed.
pub fn start_auto_updates(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(POLL_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            if let Some(result) = app
                .state::<DesktopUpdater>()
                .update_to_latest(&app, true)
                .await
                && let Some(error) = result.error
            {
                tracing::warn!("Unable to automatically update Harper Desktop: {error}");
            }
        }
    });
}

/// Perform the signed plugin workflow, without changing the plugin's platform restart behavior.
async fn check_and_install<R: Runtime>(app: &AppHandle<R>) -> Result<UpdateResult, String> {
    let current_version = DesktopUpdater::current_version(app);
    let update = app
        .updater_builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| error.to_string())?
        .check()
        .await
        .map_err(|error| error.to_string())?;

    let Some(mut update) = update else {
        return Ok(UpdateResult {
            status: UpdateStatus::UpToDate,
            current_version: Some(current_version),
            latest_version: None,
            message: "Harper is up to date.".into(),
            error: None,
        });
    };
    update.timeout = Some(Duration::from_secs(10 * 60));
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|error| error.to_string())?;

    Ok(UpdateResult {
        status: UpdateStatus::Updated,
        current_version: Some(current_version),
        latest_version: Some(normalize_version(&update.version)),
        message: "Update installed. Restart Harper to finish.".into(),
        error: None,
    })
}

fn normalize_version(version: &str) -> String {
    let version = version.trim();
    version
        .strip_prefix(['v', 'V'])
        .unwrap_or(version)
        .to_owned()
}

/// A future timestamp is treated as due so a backwards clock adjustment cannot stall updates.
fn should_check_for_update(auto_update: bool, last_check: Option<u64>, now: u64) -> bool {
    auto_update && last_check.is_none_or(|last| now < last || now.saturating_sub(last) >= DAY_MS)
}
