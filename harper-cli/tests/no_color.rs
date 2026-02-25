use std::process::Command;

fn harper_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_harper-cli"))
}

/// Input that triggers at least one lint ("an test" → AnA rule).
const BAD_INPUT: &str = "This is an test.";

fn has_ansi(bytes: &[u8]) -> bool {
    bytes.contains(&0x1b)
}

#[test]
fn default_output_contains_ansi() {
    let output = harper_cli()
        .args(["lint", BAD_INPUT])
        .env_remove("NO_COLOR")
        .output()
        .unwrap();

    let combined = [&output.stdout[..], &output.stderr[..]].concat();
    assert!(
        has_ansi(&combined),
        "default output should contain ANSI escape codes"
    );
}

#[test]
fn no_color_flag_strips_ansi() {
    let output = harper_cli()
        .args(["--no-color", "lint", BAD_INPUT])
        .env_remove("NO_COLOR")
        .output()
        .unwrap();

    let combined = [&output.stdout[..], &output.stderr[..]].concat();
    assert!(
        !has_ansi(&combined),
        "--no-color output should not contain ANSI escape codes"
    );
}

#[test]
fn no_color_env_strips_ansi() {
    let output = harper_cli()
        .env("NO_COLOR", "1")
        .args(["lint", BAD_INPUT])
        .output()
        .unwrap();

    let combined = [&output.stdout[..], &output.stderr[..]].concat();
    assert!(
        !has_ansi(&combined),
        "NO_COLOR=1 output should not contain ANSI escape codes"
    );
}
