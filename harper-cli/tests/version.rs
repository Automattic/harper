use std::process::Command;

fn harper_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_harper-cli"))
}

/// Regression test for the prebuilt executable always reporting `0.1.0`.
///
/// `harper-cli` is `publish = false`, so its own crate version was never bumped
/// and stayed at `0.1.0`. `--version` must report the real Harper version, which
/// is tracked by `harper-core`.
#[test]
fn version_reports_harper_core_version() {
    let output = harper_cli().arg("--version").output().unwrap();

    assert!(
        output.status.success(),
        "--version should exit successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let reported = stdout
        .trim()
        .rsplit(' ')
        .next()
        .expect("--version output should contain a version");

    assert_eq!(
        reported,
        harper_core::core_version(),
        "--version must report the harper-core version, got {stdout:?}"
    );
    assert_ne!(
        reported, "0.1.0",
        "--version must not report the stale harper-cli crate version"
    );
}
