use std::iter::once;
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError, TrySendError, sync_channel};
use std::thread::sleep;
use std::time::Duration;

use crate::rect::Rect;
use crate::windows_broker::get_focused_monitor_scale;
use harper_core::{Span, linting::Suggestion};
use is_macro::Is;
use uiautomation::types::{Handle, TextPatternRangeEndpoint, TextUnit, TreeScope, UIProperty};
use uiautomation::variants::Variant;
use uiautomation::{
    UIAutomation, UIElement,
    patterns::{UITextPattern, UIValuePattern},
};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Accessibility::IUIAutomationTextRange;
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetWindowThreadProcessId,
};

/// Information about a worker thread.
struct WorkerData {
    sender: SyncSender<(WorkerJob, Vec<JobArgument>)>,
    receiver: Receiver<JobResult>,
}

#[derive(Debug)]
struct ApplySuggestionRequest {
    window: isize,
    expected_text: String,
    span: Span<char>,
    suggestion: Suggestion,
}

#[derive(Debug, Is)]
enum JobArgument {
    Span(Span<char>),
    Window(isize),
    Text(String),
    ApplySuggestion(ApplySuggestionRequest),
}

/// The result of a job run by the worker thread.
#[derive(Debug, Is)]
enum JobResult {
    None,
    String(String),
    GroupedRects(Vec<Vec<Rect>>),
    Err,
}

/// An actual function pointer to be run by the worker thread.
type WorkerJob = fn(&UIAutomation, Vec<JobArgument>) -> JobResult;

/// Runs and communicates with a worker thread to interact with the Win32 Automation API to query the accessibility tree.
/// Necessary because the API has very specific thread setting requirements to work.
pub struct AutomationService {
    worker_data: Option<WorkerData>,
    // Needed to redirect focus to the last focused window when the focus arrives on the Harper highlighter window
    last_focused_window: Option<isize>,
}

impl AutomationService {
    pub fn create_and_start() -> Self {
        let mut output = Self {
            last_focused_window: None,
            worker_data: None,
        };

        output.start_worker_thread();

        output
    }

    /// Starts the worker thread if it is not already running.
    /// Does nothing if the worker thread is already running.
    fn start_worker_thread(&mut self) {
        let (job_sender, job_receiver) = sync_channel::<(WorkerJob, Vec<JobArgument>)>(1);
        let (result_sender, result_receiver) = sync_channel(1);

        std::thread::spawn(move || {
            let automation = UIAutomation::new().unwrap();

            loop {
                // Stop the thread if the other side of the channel has been closed (or dropped).
                let job = match job_receiver.try_recv() {
                    Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => None,
                    Ok(job) => Some(job),
                };

                if let Some((job, arguments)) = job {
                    let result = job(&automation, arguments);

                    // Stop the thread if the other side of the channel has been closed (or dropped).
                    if let Err(err) = result_sender.try_send(result) {
                        if let TrySendError::Disconnected(_) = err {
                            break;
                        }
                    }
                }

                sleep(Duration::from_millis(16));
            }
        });

        self.worker_data = Some(WorkerData {
            receiver: result_receiver,
            sender: job_sender,
        });
    }

    /// Stops the worker thread if it is running. This method does nothing if it is not running.
    fn stop_worker_thread(&mut self) {
        // This drops the inner fields, which closes the channel, which signals to the worker to stop running.
        self.worker_data = None;
    }

    /// Attempts to run a worker job on the worker thread. Returns `None` if the worker thread does not exist.
    fn run_worker_job(&self, job: WorkerJob, arguments: Vec<JobArgument>) -> Option<JobResult> {
        let worker_data = self.worker_data.as_ref()?;
        worker_data.sender.send((job, arguments)).unwrap();
        Some(worker_data.receiver.recv().unwrap())
    }

    /// Grab text from the worker.
    /// Attempts to get the most up-to-date information possible.
    /// Returns `None` if the worker is not running.
    pub fn get_text(&mut self) -> Option<String> {
        let window = self.resolve_focused_window()?;
        let result = self.run_worker_job(get_text_job, vec![JobArgument::Window(window)])?;

        match result {
            JobResult::String(text) => Some(text),
            _ => None,
        }
    }

    pub fn apply_suggestion(
        &mut self,
        expected_text: String,
        span: Span<char>,
        suggestion: Suggestion,
    ) {
        let Some(window) = self.resolve_focused_window() else {
            return;
        };

        let request = ApplySuggestionRequest {
            window,
            expected_text,
            span,
            suggestion,
        };

        let _ = self.run_worker_job(
            apply_suggestion_job,
            vec![JobArgument::ApplySuggestion(request)],
        );
    }

    /// Pass a collection of text spans to the worker and have it compute the associated bounding boxes for each span.
    /// Each span may have multiple bounding boxes.
    /// Input spans share the same index as their output bounding box.
    pub fn get_bounding_boxes(
        &mut self,
        text: &str,
        spans: impl IntoIterator<Item = Span<char>>,
    ) -> Option<Vec<Vec<Rect>>> {
        let window = self.resolve_focused_window()?;

        let result = self.run_worker_job(
            get_bounding_rect_job,
            once(JobArgument::Window(window))
                .chain(once(JobArgument::Text(text.to_string())))
                .chain(spans.into_iter().map(JobArgument::Span))
                .collect(),
        )?;

        match result {
            JobResult::GroupedRects(rects) => Some(rects),
            _ => None,
        }
    }

    /// Returns the foreground source window, retaining the last external window while Harper's
    /// overlay owns focus.
    pub fn resolve_focused_window(&mut self) -> Option<isize> {
        let (focused_window, focused_process_id) = focused_window()?;

        if focused_process_id == std::process::id() {
            return self.last_focused_window;
        }

        self.last_focused_window = Some(focused_window);
        Some(focused_window)
    }
}

impl Drop for AutomationService {
    fn drop(&mut self) {
        self.stop_worker_thread();
    }
}

fn apply_suggestion_job(automation: &UIAutomation, mut arguments: Vec<JobArgument>) -> JobResult {
    let Some(JobArgument::ApplySuggestion(request)) = arguments.pop() else {
        return JobResult::Err;
    };

    if !arguments.is_empty() {
        return JobResult::Err;
    }

    let Ok(element) =
        text_element_for_window(automation, request.window, Some(&request.expected_text))
    else {
        eprintln!(
            "Unable to apply Windows suggestion: the source text element is no longer available"
        );
        return JobResult::None;
    };

    let Ok(current_text) = get_text(&element) else {
        eprintln!("Unable to apply Windows suggestion: the source text can no longer be read");
        return JobResult::None;
    };

    let updated_text = match apply_suggestion_to_text(
        &current_text,
        &request.expected_text,
        request.span,
        &request.suggestion,
    ) {
        Ok(updated_text) => updated_text,
        Err(error) => {
            eprintln!("Unable to apply Windows suggestion: {error}");
            return JobResult::None;
        }
    };

    let Ok(value_pattern) = element.get_pattern::<UIValuePattern>() else {
        eprintln!(
            "Unable to apply Windows suggestion: the text element has no writable value pattern"
        );
        return JobResult::None;
    };

    match value_pattern.is_readonly() {
        Ok(true) => {
            eprintln!("Unable to apply Windows suggestion: the text element is read-only");
        }
        Ok(false) => {
            if let Err(error) = value_pattern.set_value(&updated_text) {
                eprintln!("Unable to apply Windows suggestion: {error}");
            }
        }
        Err(error) => {
            eprintln!("Unable to determine whether the Windows text element is writable: {error}");
        }
    }

    JobResult::None
}

fn apply_suggestion_to_text(
    current_text: &str,
    expected_text: &str,
    span: Span<char>,
    suggestion: &Suggestion,
) -> std::result::Result<String, &'static str> {
    if current_text != expected_text {
        return Err("the source text changed after linting");
    }

    let mut chars = current_text.chars().collect::<Vec<_>>();
    if span.end > chars.len() {
        return Err("the lint span is outside the source text");
    }

    suggestion.apply(span, &mut chars);
    Ok(chars.into_iter().collect())
}

fn get_text(element: &UIElement) -> uiautomation::Result<String> {
    let pattern: UITextPattern = element.get_pattern()?;
    let range = pattern.get_document_range()?;
    range.get_text(-1)
}

/// Finds a fresh text element below `window`, preferring keyboard focus, then the smallest element
/// below the cursor, and finally the first readable text element in UI Automation tree order. When
/// `expected_text` is provided, unrelated text providers are excluded.
fn text_element_for_window(
    automation: &UIAutomation,
    window: isize,
    expected_text: Option<&str>,
) -> uiautomation::Result<UIElement> {
    let root = automation.element_from_handle(Handle::from(window))?;
    let text_condition = automation.create_property_condition(
        UIProperty::IsTextPatternAvailable,
        Variant::from(true),
        None,
    )?;
    let keyboard_condition = automation.create_property_condition(
        UIProperty::HasKeyboardFocus,
        Variant::from(true),
        None,
    )?;
    let condition = automation.create_and_condition(text_condition, keyboard_condition)?;

    for (index, element) in root
        .find_all(TreeScope::Subtree, &condition)?
        .into_iter()
        .enumerate()
    {
        if let Some(expected) = expected_text {
            let text = get_text(&element);

            let Ok(text) = text else {
                continue;
            };

            if expected_text.is_some_and(|expected| expected != text) {
                continue;
            }
        }

        return Ok(element);
    }

    Err(Error::new(
        uiautomation::errors::ERR_NOTFOUND,
        "no text element found",
    ))
}

fn cursor_overlap_area(element: &UIElement, cursor: Option<POINT>) -> Option<i64> {
    let cursor = cursor?;
    let rect = element.get_bounding_rectangle().ok()?;
    if cursor.x < rect.get_left()
        || cursor.x >= rect.get_right()
        || cursor.y < rect.get_top()
        || cursor.y >= rect.get_bottom()
    {
        return None;
    }

    Some(
        (i64::from(rect.get_right()) - i64::from(rect.get_left()))
            * (i64::from(rect.get_bottom()) - i64::from(rect.get_top())),
    )
}

fn cursor_position() -> Option<POINT> {
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point) }.ok()?;
    Some(point)
}

fn get_text_job(automation: &UIAutomation, args: Vec<JobArgument>) -> JobResult {
    let Some(JobArgument::Window(window)) = args.first() else {
        return JobResult::Err;
    };
    let Ok(element) = text_element_for_window(automation, *window, None) else {
        return JobResult::Err;
    };

    get_text(&element).map_or(JobResult::Err, JobResult::String)
}

use std::{ffi::c_void, mem::size_of};

use uiautomation::{Error, Result};
use windows::Win32::System::{
    Com::SAFEARRAY,
    Ole::{
        SafeArrayDestroy, SafeArrayGetDim, SafeArrayGetElement, SafeArrayGetElemsize,
        SafeArrayGetLBound, SafeArrayGetUBound,
    },
};

struct OwnedSafeArray(*mut SAFEARRAY);

impl Drop for OwnedSafeArray {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let _ = SafeArrayDestroy(self.0);
            }
        }
    }
}

fn bounding_rectangles_for_span(
    element: &UIElement,
    start: i32,
    len: i32,
) -> Result<Vec<(f64, f64, f64, f64)>> {
    if start < 0 || len < 0 {
        return Err(Error::new(
            uiautomation::errors::ERR_INVALID_ARG,
            "start and len must be non-negative",
        ));
    }

    let pattern: UITextPattern = element.get_pattern()?;
    let range = pattern.get_document_range()?;

    range.move_endpoint_by_range(
        TextPatternRangeEndpoint::End,
        &range,
        TextPatternRangeEndpoint::Start,
    )?;

    range.move_endpoint_by_unit(TextPatternRangeEndpoint::Start, TextUnit::Character, start)?;

    range.move_endpoint_by_range(
        TextPatternRangeEndpoint::End,
        &range,
        TextPatternRangeEndpoint::Start,
    )?;

    range.move_endpoint_by_unit(TextPatternRangeEndpoint::End, TextUnit::Character, len)?;

    let raw: &IUIAutomationTextRange = range.as_ref();
    let array = OwnedSafeArray(unsafe { raw.GetBoundingRectangles()? });

    if array.0.is_null() {
        return Ok(Vec::new());
    }

    let dim = unsafe { SafeArrayGetDim(array.0) };

    if dim != 1 {
        return Err(Error::new(
            uiautomation::errors::ERR_FORMAT,
            "bounding rectangles SAFEARRAY is not one-dimensional",
        ));
    }

    let elem_size = unsafe { SafeArrayGetElemsize(array.0) };

    if elem_size as usize != size_of::<f64>() {
        return Err(Error::new(
            uiautomation::errors::ERR_FORMAT,
            "bounding rectangles SAFEARRAY does not contain f64-sized elements",
        ));
    }

    let lower = unsafe { SafeArrayGetLBound(array.0, 1)? };
    let upper = unsafe { SafeArrayGetUBound(array.0, 1)? };

    if upper < lower {
        return Ok(Vec::new());
    }

    let count = usize::try_from(i64::from(upper) - i64::from(lower) + 1)
        .map_err(|_| Error::new(uiautomation::errors::ERR_FORMAT, "SAFEARRAY is too large"))?;

    if count % 4 != 0 {
        return Err(Error::new(
            uiautomation::errors::ERR_FORMAT,
            "bounding rectangles SAFEARRAY length is not divisible by four",
        ));
    }

    let mut result = Vec::with_capacity(count / 4);

    for rect in 0..count / 4 {
        let mut values = [0.0_f64; 4];

        for (component, value) in values.iter_mut().enumerate() {
            let offset = rect
                .checked_mul(4)
                .and_then(|n| n.checked_add(component))
                .ok_or_else(|| {
                    Error::new(uiautomation::errors::ERR_FORMAT, "SAFEARRAY index overflow")
                })?;

            let index = i64::from(lower)
                .checked_add(i64::try_from(offset).map_err(|_| {
                    Error::new(uiautomation::errors::ERR_FORMAT, "SAFEARRAY index overflow")
                })?)
                .and_then(|n| i32::try_from(n).ok())
                .ok_or_else(|| {
                    Error::new(uiautomation::errors::ERR_FORMAT, "SAFEARRAY index overflow")
                })?;

            unsafe {
                SafeArrayGetElement(array.0, &index, value as *mut f64 as *mut c_void)?;
            }
        }

        result.push((values[0], values[1], values[2], values[3]));
    }

    Ok(result)
}

fn get_bounding_rect_job(automation: &UIAutomation, arguments: Vec<JobArgument>) -> JobResult {
    let Some(JobArgument::Window(window)) = arguments.first() else {
        return JobResult::Err;
    };
    let Some(JobArgument::Text(expected_text)) = arguments.get(1) else {
        return JobResult::Err;
    };
    let Ok(text_element) = text_element_for_window(automation, *window, Some(expected_text)) else {
        return JobResult::Err;
    };

    let effective_monitor_scale = get_focused_monitor_scale();

    let mut rects = Vec::with_capacity(arguments.len().saturating_sub(2));

    for span in arguments.into_iter().skip(2) {
        let span = span.expect_span();

        let Ok(found_rects) =
            bounding_rectangles_for_span(&text_element, span.start as i32, span.len() as i32)
        else {
            return JobResult::Err;
        };

        rects.push(
            found_rects
                .iter()
                .map(|(x, y, w, h)| {
                    Rect::new(
                        *x / effective_monitor_scale,
                        *y / effective_monitor_scale,
                        *w / effective_monitor_scale,
                        *h / effective_monitor_scale,
                    )
                })
                .collect(),
        );
    }

    JobResult::GroupedRects(rects)
}

fn focused_window() -> Option<(isize, u32)> {
    let hwnd: HWND = unsafe { GetForegroundWindow() };

    if hwnd.0.is_null() {
        return None;
    }

    let mut process_id = 0;

    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
    }

    Some((hwnd.0 as isize, process_id))
}
