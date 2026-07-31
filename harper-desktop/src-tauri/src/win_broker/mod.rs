mod app_catalog;
mod cursor_position;
mod focused_window;
mod uia_text;
mod window_stability;

use harper_core::linting::Lint;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::config::Integration;
use crate::os_broker::{AccessibilityPermissionStatus, AppSearchResult, OsBroker};
use crate::rect::ActionableLint;

use self::window_stability::{
    WINDOW_MOVEMENT_SETTLE_DURATION, WindowMovementState, frontmost_window_frame_for_pid,
    settled_window_state, window_frame_changed,
};

/// Windows implementation of the OS data the highlighter needs.
///
/// Uses Microsoft UI Automation (UIA) for text extraction and standard Win32 APIs for
/// cursor position and focused window detection. The UIA automation object is created
/// lazily per thread to avoid COM threading constraints.
pub struct WindowsBroker {
    last_focused: Option<(u32, Instant)>,
    integrations: Arc<Mutex<Vec<Integration>>>,
    window_movement: Option<WindowMovementState>,
}

impl WindowsBroker {
    pub fn new(integrations: Arc<Mutex<Vec<Integration>>>) -> Self {
        Self {
            last_focused: None,
            integrations,
            window_movement: None,
        }
    }

    fn target_pid(&mut self) -> Option<u32> {
        if let Some((last_pid, measured_at)) = self.last_focused {
            if Instant::now().duration_since(measured_at).as_secs() < 3 {
                return Some(last_pid);
            }
        }

        let focused_pid = match focused_window::focused_window_pid() {
            Ok(Some(pid)) => pid,
            _ => return None,
        };

        let current_pid = std::process::id();
        if focused_pid == current_pid {
            return self.last_focused.map(|(pid, _)| pid);
        }

        self.last_focused = Some((focused_pid, Instant::now()));
        Some(focused_pid)
    }

    fn window_is_moving(&mut self, pid: u32) -> bool {
        let Some(frame) = frontmost_window_frame_for_pid(pid) else {
            self.window_movement = None;
            return true;
        };

        let now = Instant::now();
        let Some(state) = &mut self.window_movement else {
            self.window_movement = Some(settled_window_state(pid, frame, now));
            return false;
        };

        if state.pid != pid {
            *state = settled_window_state(pid, frame, now);
            return false;
        }

        if window_frame_changed(state.frame, frame) {
            state.frame = frame;
            state.last_changed_at = now;
            return true;
        }

        now.duration_since(state.last_changed_at) < WINDOW_MOVEMENT_SETTLE_DURATION
    }

    fn exe_name_for_pid(&self, pid: u32) -> Option<String> {
        use windows::{
            Win32::Foundation::CloseHandle,
            Win32::System::Threading::{
                OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
                QueryFullProcessImageNameW,
            },
        };

        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()? };

        let mut buf = vec![0u16; 260];
        let mut size = buf.len() as u32;

        let ok = unsafe {
            QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_WIN32,
                windows::core::PWSTR(buf.as_mut_ptr()),
                &mut size,
            )
        }
        .is_ok();

        unsafe { CloseHandle(handle).ok() };

        if !ok {
            return None;
        }

        let path: String = String::from_utf16_lossy(&buf[..size as usize]);
        std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_lowercase())
    }
}

impl Default for WindowsBroker {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new(Integration::curated_integrations())))
    }
}

impl OsBroker for WindowsBroker {
    fn get_boxes(
        &mut self,
        lint_text: &mut dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>>,
    ) -> Vec<ActionableLint> {
        let pid = match self.target_pid() {
            Some(pid) => pid,
            None => {
                self.window_movement = None;
                return Vec::new();
            }
        };

        let app_id = match self.exe_name_for_pid(pid) {
            Some(name) => name,
            None => {
                self.window_movement = None;
                return Vec::new();
            }
        };

        let integration_enabled = match self.integrations.lock() {
            Ok(integrations) => Integration::is_integration_enabled_in(&integrations, &app_id),
            Err(e) => {
                eprintln!("Unable to read integrations: {e}");
                false
            }
        };

        if !integration_enabled {
            self.window_movement = None;
            return Vec::new();
        }

        if self.window_is_moving(pid) {
            return Vec::new();
        }

        let Some((automation, root)) = uia_text::focused_element() else {
            return Vec::new();
        };

        uia_text::collect_rects(&root, &automation, lint_text)
    }

    fn cursor_position(&self) -> Option<egui::Pos2> {
        cursor_position::cursor_position()
    }

    fn accessibility_permission_status(&self) -> AccessibilityPermissionStatus {
        AccessibilityPermissionStatus::Granted
    }

    fn system_integration_display_name(&self, app_id: &str) -> String {
        app_catalog::app_display_name(app_id)
    }

    fn search_apps(&self, query: &str) -> Result<Vec<AppSearchResult>, String> {
        app_catalog::search_apps(query)
    }
}
