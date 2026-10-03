use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::Duration;

fn repl(input: &str, args: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_harper-cli"))
        .args(["--no-color", "repl"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn checks_multiple_lines_and_an_unterminated_final_line() {
    let output = repl("This is an test.\nThis is a test.", &["--only", "AnA"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("::AnA]").count(), 1, "{stdout}");
    assert!(stdout.contains("Replace with: “a”"), "{stdout}");
    assert!(stdout.ends_with("No lints found.\n"), "{stdout}");
    assert!(output.stderr.is_empty(), "no prompts for piped input");
}

#[test]
fn skips_blank_lines_and_handles_crlf() {
    let output = repl("\r\n   \n\t\nThis is a test.\r\n", &["--only", "AnA"]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"No lints found.\n");
}

#[test]
fn exits_on_empty_input() {
    let output = repl("", &[]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn uses_curated_rules_by_default_and_shows_multiple_suggestions() {
    let output = repl("The colour is nice.\n", &[]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("::SpellCheck]"), "{stdout}");
    assert!(stdout.contains("Replace with: “color”"), "{stdout}");
    assert!(stdout.matches("Replace with:").count() > 1, "{stdout}");
}

#[test]
fn only_filters_rules_and_accepts_a_comma_separated_list() {
    let input = "This is an test with a colour.\n";
    let output = repl(input, &["--only", "AnA"]);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("::AnA]"), "{stdout}");
    assert!(!stdout.contains("::SpellCheck]"), "{stdout}");

    let output = repl(input, &["--only", "AnA,SpellCheck"]);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("::AnA]"), "{stdout}");
    assert!(stdout.contains("::SpellCheck]"), "{stdout}");
}

#[test]
fn respects_dialect() {
    let input = "The colour is nice.\n";
    let american = repl(input, &["--only", "SpellCheck", "--dialect", "us"]);
    assert!(
        String::from_utf8(american.stdout)
            .unwrap()
            .contains("::SpellCheck]")
    );
    let british = repl(input, &["--only", "SpellCheck", "--dialect", "uk"]);
    assert_eq!(british.stdout, b"No lints found.\n");
}

#[test]
fn rejects_invalid_dialect() {
    let output = repl("", &["--dialect", "not-a-dialect"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("Invalid dialect")
    );
}

#[test]
fn warns_once_about_unknown_rules() {
    let output = repl(
        "This is an test.\nThis is an test.\n",
        &["--only", "UnknownRule"],
    );
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        stderr
            .matches("Cannot enable unknown rule 'UnknownRule'")
            .count(),
        1
    );
    assert_eq!(stderr.matches("No rules are enabled").count(), 1);
    assert_eq!(output.stdout, b"No lints found.\nNo lints found.\n");
}

#[test]
fn reports_in_text_order_not_rule_name_order() {
    let output = repl("The colour is an test.\n", &["--only", "AnA,SpellCheck"]);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.find("::SpellCheck]").unwrap() < stdout.find("::AnA]").unwrap());
}

#[test]
fn preserves_unicode_and_leading_whitespace_in_reports() {
    let output = repl("  Café is an test.\n", &["--only", "AnA"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("  Café is an test."), "{stdout}");
    assert!(
        stdout.contains("repl:1:11"),
        "character offsets must not be byte offsets: {stdout}"
    );
    assert!(
        !stdout.contains('\x1b'),
        "--no-color must suppress ANSI escapes"
    );
}

#[test]
fn no_color_environment_disables_colors() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_harper-cli"))
        .args(["repl", "--only", "AnA"])
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"This is an test.\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(!output.stdout.contains(&0x1b));
    assert!(!output.stderr.contains(&0x1b));
    assert!(String::from_utf8(output.stdout).unwrap().contains("::AnA]"));
}

#[test]
fn produces_results_before_eof() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_harper-cli"))
        .args(["--no-color", "repl", "--only", "AnA"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut line = String::new();
        BufReader::new(stdout).read_line(&mut line).unwrap();
        sender.send(line).unwrap();
    });
    stdin.write_all(b"This is a test.\n").unwrap();
    // Keep stdin open: a REPL must not wait for the entire stream to finish.
    let result = receiver.recv_timeout(Duration::from_secs(30));
    if result.is_err() {
        child.kill().unwrap();
    }
    drop(stdin);
    let status = child.wait().unwrap();
    reader.join().unwrap();
    assert_eq!(result.unwrap(), "No lints found.\n");
    assert!(status.success());
}
