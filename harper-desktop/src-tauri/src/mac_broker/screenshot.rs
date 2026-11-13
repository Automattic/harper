use std::time::{Duration, Instant};

use objc2_app_kit::NSRunningApplication;
use objc2_foundation::ns_string;

const SCREENSHOT_POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Cache Screenshot's process state between polls, including while overlays are hidden.
pub struct ScreenshotPoll {
    running: bool,
    next_check: Instant,
}

impl ScreenshotPoll {
    pub fn new(now: Instant) -> Self {
        Self {
            running: false,
            next_check: now,
        }
    }

    pub fn is_running(&mut self) -> bool {
        self.poll(Instant::now(), || {
            NSRunningApplication::runningApplicationsWithBundleIdentifier(ns_string!(
                "com.apple.screencaptureui"
            ))
            .iter()
            .any(|app| !app.isTerminated())
        })
    }

    fn poll(&mut self, now: Instant, is_running: impl FnOnce() -> bool) -> bool {
        if now >= self.next_check {
            self.next_check = now + SCREENSHOT_POLL_INTERVAL;
            self.running = is_running();
        }
        self.running
    }
}

#[cfg(test)]
mod tests {
    use super::{SCREENSHOT_POLL_INTERVAL, ScreenshotPoll};
    use std::time::{Duration, Instant};

    #[test]
    fn screenshot_running_at_startup_hides_before_the_first_interval() {
        let now = Instant::now();
        let mut poll = ScreenshotPoll::new(now);

        assert!(poll.poll(now, || true));
        assert_eq!(poll.next_check, now + SCREENSHOT_POLL_INTERVAL);
    }

    #[test]
    fn frequent_events_use_cached_state_without_postponing_the_next_check() {
        let now = Instant::now();
        let mut poll = ScreenshotPoll::new(now);
        assert!(!poll.poll(now, || false));

        for millis in [1, 16, 250, 499] {
            assert!(!poll.poll(now + Duration::from_millis(millis), || {
                panic!("process query ran before its deadline")
            }));
            assert_eq!(poll.next_check, now + SCREENSHOT_POLL_INTERVAL);
        }

        assert!(poll.poll(now + SCREENSHOT_POLL_INTERVAL, || true));
        assert!(poll.poll(now + Duration::from_millis(501), || {
            panic!("cached running state should be retained")
        }));
    }

    #[test]
    fn hidden_overlays_keep_polling_until_exit_and_handle_another_capture() {
        let now = Instant::now();
        let mut poll = ScreenshotPoll::new(now);
        assert!(poll.poll(now, || true));

        // An idle Screenshot process must keep overlays hidden until it exits.
        for tick in 1..=30 {
            assert!(poll.poll(now + SCREENSHOT_POLL_INTERVAL * tick, || true));
        }
        assert!(!poll.poll(now + SCREENSHOT_POLL_INTERVAL * 31, || false));
        assert!(!poll.poll(now + SCREENSHOT_POLL_INTERVAL * 32, || false));
        assert!(poll.poll(now + SCREENSHOT_POLL_INTERVAL * 33, || true));
    }
}
