use std::{
    cell::RefCell,
    collections::{self, BTreeMap},
    rc::Rc,
};

use crate::windows_broker::automation_service::AutomationService;
use crate::{os_broker::AccessibilityPermissionStatus, os_broker::OsBroker, rect::ActionableLint};
use egui::Pos2;
use harper_core::linting::Lint;
use uiautomation::Result;
use uiautomation::{UIAutomation, UIElement, UITreeWalker};
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONEAREST, MonitorFromWindow};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetPhysicalCursorPos,
};
mod automation_service;

pub struct WindowsBroker {
    service: Rc<RefCell<AutomationService>>,
}

impl WindowsBroker {
    pub fn new() -> Self {
        Self {
            service: Rc::new(RefCell::new(AutomationService::create_and_start())),
        }
    }
}

impl OsBroker for WindowsBroker {
    fn get_boxes(
        &mut self,
        lint_text: &mut dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>>,
    ) -> Vec<ActionableLint> {
        let text = self.service.borrow_mut().get_text();
        if let Some(text) = text {
            if text.len() > 16_000 {
                return Vec::new();
            }

            let lints = lint_text(text.as_str());

            let all_lint_iter = lints.values().map(|r| r.iter()).flatten();
            let Some(rects) = self
                .service
                .borrow_mut()
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
                    let service = Rc::clone(&self.service);
                    rects.into_iter().map(move |r| {
                        let service = Rc::clone(&service);
                        let suggestion_text = text.clone();
                        let suggestion_span = lint.span;
                        ActionableLint::new(
                            r,
                            lint_id.clone(),
                            lint.clone(),
                            text.clone(),
                            move |suggestion| {
                                service.borrow_mut().apply_suggestion(
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
        dbg!(pos);

        Some(pos)
    }

    fn accessibility_permission_status(&self) -> AccessibilityPermissionStatus {
        AccessibilityPermissionStatus::Granted
    }
}

fn get_focused_monitor_scale() -> f64 {
    unsafe {
        let window = GetForegroundWindow();
        let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST);

        let mut x = 0;
        let mut y = 0;

        GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y);

        let effective_scale = x as f64 / 96.;
        effective_scale
    }
}
