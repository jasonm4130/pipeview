use std::process::{Command, Stdio};
use std::io::Write;

#[test]
fn stdout_matches_stdin_exactly() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "--quiet"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn pipespy");

    let input = (1..=1000).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n") + "\n";

    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();

    let output = child.wait_with_output().expect("failed to wait");
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout, input, "stdout must match stdin byte-for-byte");
}

#[test]
fn handles_empty_input() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "--quiet"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn pipespy");

    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait");
    assert!(output.stdout.is_empty(), "empty input should produce empty output");
}

#[test]
fn handles_binary_data() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "--quiet"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn pipespy");

    let input: Vec<u8> = (0..=255u8).cycle().take(4096).collect();
    let mut lined_input = Vec::new();
    for chunk in input.chunks(64) {
        lined_input.extend_from_slice(chunk);
        lined_input.push(b'\n');
    }

    child.stdin.take().unwrap().write_all(&lined_input).unwrap();

    let output = child.wait_with_output().expect("failed to wait");
    assert_eq!(output.stdout, lined_input, "binary data must pass through unchanged");
}
