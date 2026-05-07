use std::process::Command;

fn harper_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_harper-cli"))
}

#[test]
fn word_provenance_finds_direct_entry() {
    let output = harper_cli()
        .args(["word-provenance", "hello"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("DIRECT"),
        "expected DIRECT provenance for 'hello', got: {stdout}"
    );
}

#[test]
fn word_provenance_finds_affix_generated() {
    // "quickly" should be generated from "quick" via the Y suffix flag
    let output = harper_cli()
        .args(["word-provenance", "quickly"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("AFFIX"),
        "expected AFFIX provenance for 'quickly', got: {stdout}"
    );
    assert!(
        stdout.contains("quick"),
        "expected base word 'quick' for 'quickly', got: {stdout}"
    );
}

#[test]
fn word_provenance_not_found() {
    let output = harper_cli()
        .args(["word-provenance", "xyzzyplugh"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("not found"),
        "expected 'not found' for nonsense word, got: {stdout}"
    );
}

#[test]
fn word_provenance_json_output() {
    let output = harper_cli()
        .args(["word-provenance", "--json", "hello"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("expected valid JSON, got: {stdout}\nerror: {e}"));

    assert_eq!(parsed["query"], "hello");
    assert!(parsed["routes"].is_array());
    assert!(
        parsed["routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["provenance"]["type"] == "direct"),
        "expected at least one direct route"
    );
}

#[test]
fn word_provenance_multiple_words() {
    let output = harper_cli()
        .args(["word-provenance", "hello", "quickly"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("hello") && stdout.contains("quickly"),
        "expected output for both words, got: {stdout}"
    );
}
