use crate::windows_broker::automation_service::AutomationService;
use crate::{
    os_broker::{AccessibilityPermissionStatus, AppSearchResult, OsBroker},
    rect::ActionableLint,
};
use cached::cached;
use egui::Pos2;
use harper_core::linting::Lint;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
};
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONEAREST, MonitorFromWindow};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetForegroundWindow};
use wintheon::file::{IconSize, Priority};
use wintheon::gather::Gatherer;
mod automation_service;

pub struct WindowsBroker {
    service: Arc<Mutex<AutomationService>>,
}

impl WindowsBroker {
    pub fn new() -> Self {
        Self {
            service: Arc::new(Mutex::new(AutomationService::create_and_start())),
        }
    }
}

impl Default for WindowsBroker {
    fn default() -> Self {
        Self::new()
    }
}

impl OsBroker for WindowsBroker {
    fn get_boxes(
        &mut self,
        lint_text: &mut dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>>,
    ) -> Vec<ActionableLint> {
        let text = self.service.lock().unwrap().get_text();
        if let Some(text) = text {
            if text.len() > 16_000 {
                return Vec::new();
            }

            let lints = lint_text(text.as_str());

            let all_lint_iter = lints.values().map(|r| r.iter()).flatten();
            let Some(rects) = self
                .service
                .lock()
                .unwrap()
                .get_bounding_boxes(all_lint_iter.map(|l| l.span))
            else {
                return Vec::new();
            };

            lints
                .into_iter()
                .map(|(lint_id, lints)| lints.into_iter().map(move |l| (lint_id.clone(), l)))
                .flatten()
                .zip(rects)
                .map(|((lint_id, lint), rects)| {
                    let text = text.clone();
                    let service = self.service.clone();
                    rects.into_iter().map(move |r| {
                        let service = service.clone();
                        let suggestion_text = text.clone();
                        let suggestion_span = lint.span;
                        ActionableLint::new(
                            r,
                            lint_id.clone(),
                            lint.clone(),
                            text.clone(),
                            move |suggestion| {
                                service.lock().unwrap().apply_suggestion(
                                    suggestion_text,
                                    suggestion_span,
                                    suggestion,
                                );
                            },
                        )
                    })
                })
                .flatten()
                .collect()
        } else {
            Vec::new()
        }
    }

    fn cursor_position(&self) -> Option<Pos2> {
        let mut point = POINT::default();

        unsafe {
            GetCursorPos(&mut point).unwrap();
        }

        let monitor_scale = get_focused_monitor_scale();

        let pos = Pos2::new(
            point.x as f32 / monitor_scale as f32,
            point.y as f32 / monitor_scale as f32,
        );

        Some(pos)
    }

    fn accessibility_permission_status(&self) -> AccessibilityPermissionStatus {
        AccessibilityPermissionStatus::Granted
    }

    fn request_accessibility_permission(&self) -> AccessibilityPermissionStatus {
        AccessibilityPermissionStatus::Granted
    }

    fn integration_display_name(&self, bundle_id: &str) -> String {
        if let Some(entry) = look_up_application(bundle_id) {
            return entry.display_name;
        }

        bundle_id.to_string()
    }

    fn installed_application_bundle_ids(&self) -> Result<Vec<String>, String> {
        let list = installed_applications_list();
        Ok(list
            .iter()
            .map(|i| i.path.to_string_lossy().into_owned())
            .collect())
    }

    fn application_icon_png(&self, bundle_id: &str) -> Result<Vec<u8>, String> {
        if let Some(entry) = look_up_application(bundle_id) {
            if let Some(png) = entry.icon_png {
                return Ok(png);
            } else {
                return Err("Found application but it was missing an icon.".to_string());
            }
        } else {
            return Err("Unable to locate application.".to_string());
        }
    }

    fn launch_app_bundle(&self, _bundle_id: &str) -> Result<(), String> {
        Ok(())
    }

    /// Search for an application in the OS' global list of installed apps.
    fn search_apps(&self, _query: &str) -> Result<Vec<AppSearchResult>, String> {
        Ok(Vec::new())
    }
}

fn get_focused_monitor_scale() -> f64 {
    unsafe {
        let window = GetForegroundWindow();
        let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST);

        let mut x = 0;
        let mut y = 0;

        let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y);

        let effective_scale = x as f64 / 96.;
        effective_scale
    }
}

#[derive(Debug, Clone)]
struct ApplicationListEntry {
    path: PathBuf,
    /// The PNG bytes of an icon. 256 on a side, square.
    icon_png: Option<Vec<u8>>,
    display_name: String,
}

fn look_up_application(bundle_id: &str) -> Option<ApplicationListEntry> {
    // In Windows, the application path is the bundle ID.
    let list = installed_applications_list();
    if let Some(entry) = list
        .iter()
        .find(|entry| entry.path.to_string_lossy() == bundle_id)
    {
        Some(entry.clone())
    } else {
        None
    }
}

#[cached]
fn installed_applications_list() -> Arc<Vec<ApplicationListEntry>> {
    let mut list = Vec::new();

    for res in gatherer().scan() {
        if let Ok(app) = res {
            let icon = if let Ok(icon) = app.entry.icon() {
                icon.extract_icon_as_png_at(IconSize::Jumbo)
            } else {
                None
            };

            list.push(ApplicationListEntry {
                path: app.entry.path().to_owned(),
                icon_png: icon,
                display_name: app.entry.display_name(),
            })
        }
    }

    Arc::new(list)
}

fn gatherer() -> Gatherer {
    Gatherer::new()
        .with_desktop(Priority(1.0))
        .with_start_menu(Priority(1.5))
        .with_windows_apps(Priority(2.0))
}
