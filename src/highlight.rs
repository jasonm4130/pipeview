use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use crate::format::Format;

/// Dispatches to a format-specific highlighter, or returns plain text.
pub fn highlight_line(line: &str, format: Format) -> Line<'static> {
    match format {
        Format::Json => highlight_json(line),
        Format::Csv => highlight_csv(line),
        Format::PlainText => Line::from(Span::raw(line.to_owned())),
    }
}

/// JSON syntax highlighting using a character-by-character state machine.
///
/// - String keys (before `:`) in Green
/// - String values (after `:`) in Cyan
/// - Numbers in Yellow
/// - Structural chars (`{}[],:`) in White
fn highlight_json(line: &str) -> Line<'static> {
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum State {
        Normal,
        InStringKey,
        InStringValue,
        InNumber,
    }

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut state = State::Normal;
    let mut escaped = false;
    let mut current = String::new();

    // Track whether the next string encountered is a key or value.
    // After `{` or `,` (at the top level), the next string is a key.
    // After `:` the next string is a value.
    let mut next_string_is_key = true;

    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        match state {
            State::Normal => {
                if ch == '"' {
                    // Flush any accumulated whitespace/structural chars
                    if !current.is_empty() {
                        spans.push(Span::styled(
                            current.clone(),
                            Style::default().fg(Color::White),
                        ));
                        current.clear();
                    }
                    // Start a string — decide key vs value
                    current.push(ch);
                    state = if next_string_is_key {
                        State::InStringKey
                    } else {
                        State::InStringValue
                    };
                } else if ch.is_ascii_digit() || (ch == '-' && {
                    // Check that a digit follows the minus
                    i + 1 < len && chars[i + 1].is_ascii_digit()
                }) {
                    // Flush structural/whitespace buffer
                    if !current.is_empty() {
                        spans.push(Span::styled(
                            current.clone(),
                            Style::default().fg(Color::White),
                        ));
                        current.clear();
                    }
                    current.push(ch);
                    state = State::InNumber;
                } else if "{[".contains(ch) {
                    current.push(ch);
                    // After `{` or `[`, the next string is a key
                    next_string_is_key = true;
                } else if ch == ':' {
                    current.push(ch);
                    // After `:`, the next string is a value
                    next_string_is_key = false;
                } else if ch == ',' {
                    current.push(ch);
                    // After `,`, the next string is a key
                    next_string_is_key = true;
                } else {
                    current.push(ch);
                }
            }

            State::InStringKey | State::InStringValue => {
                if escaped {
                    current.push(ch);
                    escaped = false;
                } else if ch == '\\' {
                    current.push(ch);
                    escaped = true;
                } else if ch == '"' {
                    // Close the string
                    current.push(ch);
                    let color = if state == State::InStringKey {
                        Color::Green
                    } else {
                        Color::Cyan
                    };
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(color),
                    ));
                    current.clear();
                    state = State::Normal;
                } else {
                    current.push(ch);
                }
            }

            State::InNumber => {
                if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E'
                    || ch == '+' || ch == '-'
                {
                    current.push(ch);
                } else {
                    // Flush number
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(Color::Yellow),
                    ));
                    current.clear();
                    state = State::Normal;
                    // Re-process current char without advancing i
                    continue;
                }
            }
        }

        i += 1;
    }

    // Flush any remaining buffer
    if !current.is_empty() {
        let color = match state {
            State::InStringKey => Color::Green,
            State::InStringValue => Color::Cyan,
            State::InNumber => Color::Yellow,
            State::Normal => Color::White,
        };
        spans.push(Span::styled(current, Style::default().fg(color)));
    }

    Line::from(spans)
}

/// CSV syntax highlighting — split on commas, rotate field colors, commas in DarkGray.
fn highlight_csv(line: &str) -> Line<'static> {
    const FIELD_COLORS: [Color; 6] = [
        Color::White,
        Color::Cyan,
        Color::Green,
        Color::Yellow,
        Color::Magenta,
        Color::Blue,
    ];

    let mut spans: Vec<Span<'static>> = Vec::new();
    let fields: Vec<&str> = line.split(',').collect();

    for (idx, field) in fields.iter().enumerate() {
        let color = FIELD_COLORS[idx % FIELD_COLORS.len()];
        spans.push(Span::styled(
            field.to_string(),
            Style::default().fg(color),
        ));
        if idx < fields.len() - 1 {
            spans.push(Span::styled(
                ",".to_string(),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_passthrough() {
        let input = "hello world";
        let line = highlight_line(input, Format::PlainText);
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "hello world");
    }

    #[test]
    fn csv_splits_on_commas() {
        let input = "alice,30,seattle";
        let line = highlight_line(input, Format::Csv);
        // 3 fields + 2 commas = 5 spans
        assert_eq!(line.spans.len(), 5);
        assert_eq!(line.spans[0].content, "alice");
        assert_eq!(line.spans[1].content, ",");
        assert_eq!(line.spans[2].content, "30");
        assert_eq!(line.spans[3].content, ",");
        assert_eq!(line.spans[4].content, "seattle");
    }

    #[test]
    fn json_produces_colored_spans() {
        let input = r#"{"name": "Alice", "age": 30}"#;
        let line = highlight_line(input, Format::Json);
        // Must produce multiple spans
        assert!(line.spans.len() > 1, "expected multiple spans, got {}", line.spans.len());

        // Verify at least one Green span (key) and one Cyan span (value)
        let has_green = line.spans.iter().any(|s| {
            s.style.fg == Some(Color::Green)
        });
        let has_cyan = line.spans.iter().any(|s| {
            s.style.fg == Some(Color::Cyan)
        });
        let has_yellow = line.spans.iter().any(|s| {
            s.style.fg == Some(Color::Yellow)
        });

        assert!(has_green, "expected a green (key) span");
        assert!(has_cyan, "expected a cyan (value) span");
        assert!(has_yellow, "expected a yellow (number) span");
    }
}
