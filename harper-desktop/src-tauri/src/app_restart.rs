//! Fallible, user-initiated restart coordination.
use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
};

/// Keep the gate closed after launch succeeds, until the old parent exits.
/// A failed launch leaves the current parent available for another attempt.
pub fn restart_once(
    restarting: &AtomicBool,
    stop: impl FnOnce(),
    launch: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    restarting
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .map_err(|_| io::Error::other("Harper is already restarting."))?;
    stop();
    let result = launch();
    if result.is_err() {
        restarting.store(false, Ordering::SeqCst);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn stops_before_launch_and_launches_only_once() {
        let gate = AtomicBool::new(false);
        let stopped = Cell::new(false);
        let launches = Cell::new(0);
        restart_once(
            &gate,
            || stopped.set(true),
            || {
                assert!(stopped.get());
                launches.set(launches.get() + 1);
                Ok(())
            },
        )
        .unwrap();
        assert!(
            restart_once(
                &gate,
                || panic!("duplicate stop"),
                || panic!("duplicate launch")
            )
            .is_err()
        );
        assert_eq!(launches.get(), 1);
    }

    #[test]
    fn launch_failure_is_reported_and_can_be_retried() {
        let gate = AtomicBool::new(false);
        let error =
            restart_once(&gate, || {}, || Err(io::Error::other("launch denied"))).unwrap_err();
        assert_eq!(error.to_string(), "launch denied");
        assert!(!gate.load(Ordering::SeqCst));
        restart_once(&gate, || {}, || Ok(())).unwrap();
    }
}
