use pipespy::format::{self, Format};

#[test]
fn detects_json_lines() {
    let lines = vec![
        r#"{"id": 1, "name": "alice"}"#.to_string(),
        r#"{"id": 2, "name": "bob"}"#.to_string(),
        r#"{"id": 3, "name": "charlie"}"#.to_string(),
        r#"{"id": 4, "name": "dave"}"#.to_string(),
    ];
    assert_eq!(format::detect(&lines), Format::Json);
}

#[test]
fn detects_csv_data() {
    let lines = vec![
        "id,name,email".to_string(),
        "1,alice,alice@example.com".to_string(),
        "2,bob,bob@example.com".to_string(),
        "3,charlie,charlie@example.com".to_string(),
    ];
    assert_eq!(format::detect(&lines), Format::Csv);
}

#[test]
fn plain_log_lines() {
    let lines = vec![
        "[2026-04-09 10:00:01] INFO Starting server".to_string(),
        "[2026-04-09 10:00:02] WARN High memory".to_string(),
        "[2026-04-09 10:00:03] ERROR Connection lost".to_string(),
    ];
    assert_eq!(format::detect(&lines), Format::PlainText);
}
