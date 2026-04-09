use std::fmt;

/// Represents the detected or forced output format for pipeline data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Json,
    Csv,
    PlainText,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Format::Json => write!(f, "JSON"),
            Format::Csv => write!(f, "CSV"),
            Format::PlainText => write!(f, "Text"),
        }
    }
}

/// Detects the format of the given sample lines.
///
/// Checks up to 32 lines and determines format based on:
/// - Empty/all blank -> PlainText
/// - All non-empty trimmed lines parse as valid JSON -> Json
/// - All lines have the same number of commas (>= 1) -> Csv
/// - Otherwise -> PlainText
pub fn detect(lines: &[String]) -> Format {
    // Check up to 32 lines
    let sample = if lines.len() > 32 {
        &lines[0..32]
    } else {
        lines
    };

    // Filter non-empty trimmed lines
    let non_empty: Vec<&str> = sample
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    // Empty or all blank -> PlainText
    if non_empty.is_empty() {
        return Format::PlainText;
    }

    // Check if all lines are valid JSON
    let all_json = non_empty.iter().all(|line| {
        serde_json::from_str::<serde_json::Value>(line).is_ok()
    });

    if all_json {
        return Format::Json;
    }

    // Check if all lines have the same number of commas (>= 1)
    if let Some(first_comma_count) = non_empty
        .iter()
        .map(|line| line.matches(',').count())
        .next()
    {
        if first_comma_count >= 1
            && non_empty
                .iter()
                .all(|line| line.matches(',').count() == first_comma_count)
        {
            return Format::Csv;
        }
    }

    Format::PlainText
}

/// Resolves the output format based on explicit flags or detection.
///
/// Priority:
/// - If no_detect -> PlainText
/// - If force_json -> Json
/// - If force_csv -> Csv
/// - Otherwise -> detect(samples)
pub fn resolve(
    force_json: bool,
    force_csv: bool,
    no_detect: bool,
    samples: &[String],
) -> Format {
    if no_detect {
        return Format::PlainText;
    }

    if force_json {
        return Format::Json;
    }

    if force_csv {
        return Format::Csv;
    }

    detect(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_json() {
        let lines = vec![
            r#"{"name": "Alice", "age": 30}"#.to_string(),
            r#"{"name": "Bob", "age": 25}"#.to_string(),
            r#"{"name": "Charlie", "age": 35}"#.to_string(),
        ];
        assert_eq!(detect(&lines), Format::Json);
    }

    #[test]
    fn detects_csv() {
        let lines = vec![
            "name,age,city".to_string(),
            "Alice,30,NYC".to_string(),
            "Bob,25,LA".to_string(),
        ];
        assert_eq!(detect(&lines), Format::Csv);
    }

    #[test]
    fn detects_plain_text() {
        let lines = vec![
            "[2026-04-09 10:00:00] INFO: Starting application".to_string(),
            "[2026-04-09 10:00:01] DEBUG: Processing input".to_string(),
            "[2026-04-09 10:00:02] INFO: Complete".to_string(),
        ];
        assert_eq!(detect(&lines), Format::PlainText);
    }

    #[test]
    fn empty_input_is_plain_text() {
        let lines: Vec<String> = vec![];
        assert_eq!(detect(&lines), Format::PlainText);
    }

    #[test]
    fn resolve_force_json() {
        let lines = vec!["not json at all".to_string()];
        assert_eq!(resolve(true, false, false, &lines), Format::Json);
    }

    #[test]
    fn resolve_no_detect() {
        let lines = vec![
            r#"{"valid": "json"}"#.to_string(),
        ];
        assert_eq!(resolve(false, false, true, &lines), Format::PlainText);
    }
}
