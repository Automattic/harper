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
    /// Report a fresh check without forgetting the update already installed on disk.
    fn with_latest_version(mut self, latest_version: Option<String>) -> Self {
        if let Some(latest_version) = latest_version {
            if self.latest_version.as_ref() != Some(&latest_version) {
                self.message = format!(
                    "Update installed. Restart Harper to finish before installing version {latest_version}."
                );
            }
            self.latest_version = Some(latest_version);
        }
        self
    }

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
    /// Checks continue after an installation; further installations wait until restart.
    pub async fn update_to_latest<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        automatic: bool,
    ) -> Option<UpdateResult> {
        self.run_update(|installed_update| async move {
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

            check_and_install(app, installed_update).await.map(Some)
        })
        .await
    }

    /// If an update is already prepared and downloaded, it will be provided to the operation.
    async fn run_update<F, Fut>(&self, operation: F) -> Option<UpdateResult>
    where
        F: FnOnce(Option<UpdateResult>) -> Fut,
        Fut: Future<Output = Result<Option<UpdateResult>, String>>,
    {
        let mut installed = self.installed_update.lock().await;
        let result = match operation(installed.clone()).await {
            Ok(result) => result?,
            Err(error) => UpdateResult::error(error),
        };
        if installed.is_none() && result.status == UpdateStatus::Updated {
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
async fn check_and_install<R: Runtime>(
    app: &AppHandle<R>,
    installed_update: Option<UpdateResult>,
) -> Result<UpdateResult, String> {
    let current_version = DesktopUpdater::current_version(app);
    let update = app
        .updater_builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| error.to_string())?
        .check()
        .await
        .map_err(|error| error.to_string())?;

    // Still check the signed-release endpoint, but never replace a pending installation.
    if let Some(installed) = installed_update {
        return Ok(
            installed.with_latest_version(update.map(|update| normalize_version(&update.version)))
        );
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn installed_result() -> UpdateResult {
        UpdateResult {
            status: UpdateStatus::Updated,
            current_version: Some("2.11.0".into()),
            latest_version: Some("2.12.0".into()),
            message: "Update installed. Restart Harper to finish.".into(),
            error: None,
        }
    }

    async fn install(updater: &DesktopUpdater) {
        let result = updater
            .run_update(|installed| async move {
                assert!(installed.is_none());
                Ok(Some(installed_result()))
            })
            .await;
        assert_eq!(result, Some(installed_result()));
    }

    #[test]
    fn automatic_checks_respect_preference_and_daily_interval() {
        let now = DAY_MS * 2;
        for (enabled, last_check, expected) in [
            (false, None, false),
            (false, Some(0), false),
            (false, Some(now + 1), false),
            (true, None, true),
            (true, Some(now), false),
            (true, Some(now - DAY_MS + 1), false),
            (true, Some(now - DAY_MS), true),
            (true, Some(0), true),
            (true, Some(now + 1), true),
        ] {
            assert_eq!(should_check_for_update(enabled, last_check, now), expected);
        }
    }

    #[test]
    fn versions_are_normalized() {
        for version in ["2.12.0", "v2.12.0", " V2.12.0\n"] {
            assert_eq!(normalize_version(version), "2.12.0");
        }
    }

    #[test]
    fn same_release_keeps_restart_reminder() {
        assert_eq!(
            installed_result().with_latest_version(Some("2.12.0".into())),
            installed_result()
        );
    }

    #[test]
    fn no_release_keeps_restart_reminder() {
        assert_eq!(
            installed_result().with_latest_version(None),
            installed_result()
        );
    }

    #[test]
    fn newer_release_is_reported_without_claiming_it_was_installed() {
        let result = installed_result().with_latest_version(Some("2.13.0".into()));
        assert_eq!(result.status, UpdateStatus::Updated);
        assert_eq!(result.current_version.as_deref(), Some("2.11.0"));
        assert_eq!(result.latest_version.as_deref(), Some("2.13.0"));
        assert!(result.message.contains("before installing version 2.13.0"));
    }

    #[tokio::test]
    async fn checks_continue_after_installation_and_keep_original_installation() {
        let updater = DesktopUpdater::default();
        install(&updater).await;

        let result = updater
            .run_update(|installed| async move {
                assert_eq!(installed, Some(installed_result()));
                Ok(Some(
                    installed
                        .unwrap()
                        .with_latest_version(Some("2.13.0".into())),
                ))
            })
            .await;
        assert_eq!(result.unwrap().latest_version.as_deref(), Some("2.13.0"));

        // The latest check's version must not become the remembered installed version.
        let result = updater
            .run_update(|installed| async move {
                assert_eq!(installed, Some(installed_result()));
                Ok(installed)
            })
            .await;
        assert_eq!(result, Some(installed_result()));
    }

    #[tokio::test]
    async fn ineligible_checks_are_still_skipped_after_installation() {
        let updater = DesktopUpdater::default();
        install(&updater).await;

        let result = updater
            .run_update(|installed| async move {
                assert_eq!(installed, Some(installed_result()));
                Ok(None)
            })
            .await;
        assert_eq!(result, None);
        assert_eq!(
            *updater.installed_update.lock().await,
            Some(installed_result())
        );
    }

    #[tokio::test]
    async fn failed_checks_do_not_prevent_installation() {
        let updater = DesktopUpdater::default();
        let result = updater
            .run_update(|installed| async move {
                assert!(installed.is_none());
                Err("Network unavailable".into())
            })
            .await;
        assert_eq!(result.unwrap().status, UpdateStatus::Error);
        install(&updater).await;
    }

    #[tokio::test]
    async fn failed_checks_preserve_pending_installation() {
        let updater = DesktopUpdater::default();
        install(&updater).await;
        let result = updater
            .run_update(|installed| async move {
                assert_eq!(installed, Some(installed_result()));
                Err("Network unavailable".into())
            })
            .await;
        assert_eq!(result.unwrap().status, UpdateStatus::Error);
        assert_eq!(
            *updater.installed_update.lock().await,
            Some(installed_result())
        );
    }

    #[tokio::test]
    async fn up_to_date_checks_do_not_prevent_later_installation() {
        let updater = DesktopUpdater::default();
        let result = updater
            .run_update(|installed| async move {
                assert!(installed.is_none());
                Ok(Some(UpdateResult {
                    status: UpdateStatus::UpToDate,
                    current_version: Some("2.11.0".into()),
                    latest_version: None,
                    message: "Harper is up to date.".into(),
                    error: None,
                }))
            })
            .await;
        assert_eq!(result.unwrap().status, UpdateStatus::UpToDate);
        install(&updater).await;
    }

    #[tokio::test]
    async fn concurrent_checks_observe_the_first_installation() {
        let updater = DesktopUpdater::default();
        let first = updater.run_update(|installed| async move {
            assert!(installed.is_none());
            tokio::task::yield_now().await;
            Ok(Some(installed_result()))
        });
        let second = updater.run_update(|installed| async move {
            assert_eq!(installed, Some(installed_result()));
            Ok(installed)
        });
        let (first, second) = tokio::join!(first, second);
        assert_eq!(first, Some(installed_result()));
        assert_eq!(second, Some(installed_result()));
    }
}
