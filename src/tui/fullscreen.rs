use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{BarChart, Block, Borders, List, ListItem, Paragraph, Sparkline};

use crate::highlight::highlight_line;
use crate::stats::StatsSnapshot;
use crate::tui::App;

pub fn render(frame: &mut Frame, snap: &StatsSnapshot, samples: &[String], app: &App) {
    let area = frame.area();

    // Top-level vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // Header
            Constraint::Length(5),   // Extended stats bar
            Constraint::Length(8),   // Sparkline
            Constraint::Length(10),  // Histogram
            Constraint::Min(6),      // Records
            Constraint::Length(1),   // Footer
        ])
        .split(area);

    // --- Header ---
    let elapsed_str = format_elapsed(snap.elapsed_secs);
    let header_text = format!(
        " pipeview \u{2502} fullscreen  elapsed: {} \u{2502} q to detach",
        elapsed_str
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));
    frame.render_widget(header, chunks[0]);

    // --- Extended stats bar: 6 bordered boxes ---
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
        ])
        .split(chunks[1]);

    // THROUGHPUT (green)
    let throughput_val = format!("{:.0} lines/s", snap.effective_throughput_lines());
    let throughput_box = Paragraph::new(throughput_val)
        .block(Block::default().title(" THROUGHPUT ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Green));
    frame.render_widget(throughput_box, stats_chunks[0]);

    // BANDWIDTH (blue)
    let bandwidth_val = format_bandwidth(snap.effective_throughput_bytes() as u64);
    let bandwidth_box = Paragraph::new(bandwidth_val)
        .block(Block::default().title(" BANDWIDTH ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Blue));
    frame.render_widget(bandwidth_box, stats_chunks[1]);

    // TOTAL LINES (white)
    let total_val = format_total_lines(snap.total_lines);
    let total_box = Paragraph::new(total_val)
        .block(Block::default().title(" TOTAL LINES ").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    frame.render_widget(total_box, stats_chunks[2]);

    // MIN LINE (cyan)
    let (min_len, max_len, avg_len) = compute_length_stats(&snap.line_lengths);
    let min_val = format!("{} B", min_len);
    let min_box = Paragraph::new(min_val)
        .block(Block::default().title(" MIN LINE ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(min_box, stats_chunks[3]);

    // MAX LINE (yellow)
    let max_val = format!("{} B", max_len);
    let max_box = Paragraph::new(max_val)
        .block(Block::default().title(" MAX LINE ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(max_box, stats_chunks[4]);

    // AVG LINE (magenta)
    let avg_val = format!("{} B", avg_len);
    let avg_box = Paragraph::new(avg_val)
        .block(Block::default().title(" AVG LINE ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Magenta));
    frame.render_widget(avg_box, stats_chunks[5]);

    // --- Sparkline ---
    let sparkline_data: Vec<u64> = if snap.sparkline.len() <= 2 && snap.total_lines > 0 {
        let effective = snap.effective_throughput_lines() as u64;
        vec![effective; 20]
    } else {
        snap.sparkline.iter().map(|&v| v as u64).collect()
    };
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(" throughput history ")
                .borders(Borders::ALL),
        )
        .data(&sparkline_data)
        .style(Style::default().fg(Color::Green));
    frame.render_widget(sparkline, chunks[2]);

    // --- Histogram ---
    let buckets = build_histogram_buckets(&snap.line_lengths);
    let bar_data: Vec<(&str, u64)> = buckets
        .iter()
        .map(|(label, count)| (label.as_str(), *count))
        .collect();
    let histogram = BarChart::default()
        .block(
            Block::default()
                .title(" line length distribution ")
                .borders(Borders::ALL),
        )
        .data(&bar_data)
        .bar_width(8)
        .bar_gap(1)
        .style(Style::default().fg(Color::Cyan))
        .value_style(Style::default().fg(Color::White));
    frame.render_widget(histogram, chunks[3]);

    // --- Records ---
    let records_area = chunks[4];
    let inner_height = records_area.height.saturating_sub(2) as usize;
    let last_n = if samples.len() > inner_height {
        &samples[samples.len() - inner_height..]
    } else {
        samples
    };

    let items: Vec<ListItem> = last_n
        .iter()
        .map(|line| {
            let highlighted = highlight_line(line, app.format);
            ListItem::new(highlighted)
        })
        .collect();

    let records_list = List::new(items)
        .block(Block::default().title(" records ").borders(Borders::ALL));
    frame.render_widget(records_list, records_area);

    // --- Footer ---
    let is_done = app.done.load(std::sync::atomic::Ordering::Relaxed);
    let (pipe_status, dot_color) = if is_done {
        ("pipe done", Color::Yellow)
    } else {
        ("pipe healthy", Color::Green)
    };

    let footer_spans = vec![
        Span::raw(" f compact \u{2502} q quit  "),
        Span::raw(pipe_status),
        Span::raw(" "),
        Span::styled("\u{25cf}", Style::default().fg(dot_color)),
    ];
    let footer = Paragraph::new(Line::from(footer_spans))
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));
    frame.render_widget(footer, chunks[5]);
}

/// Returns (min, max, avg) byte lengths. Returns (0, 0, 0) if slice is empty.
fn compute_length_stats(lengths: &[u64]) -> (u64, u64, u64) {
    if lengths.is_empty() {
        return (0, 0, 0);
    }
    let min = *lengths.iter().min().unwrap();
    let max = *lengths.iter().max().unwrap();
    let sum: u64 = lengths.iter().sum();
    let avg = sum / lengths.len() as u64;
    (min, max, avg)
}

/// Builds 8 histogram buckets from line_lengths.
/// Each bucket covers a range of `max/8` bytes.
/// Trailing empty buckets are trimmed.
/// Returns a Vec of (label, count) pairs.
fn build_histogram_buckets(lengths: &[u64]) -> Vec<(String, u64)> {
    if lengths.is_empty() {
        return Vec::new();
    }

    let max = *lengths.iter().max().unwrap();
    if max == 0 {
        return Vec::new();
    }

    let bucket_size = (max / 8).max(1);
    let mut counts = [0u64; 8];

    for &len in lengths {
        let idx = ((len / bucket_size) as usize).min(7);
        counts[idx] += 1;
    }

    // Find last non-empty bucket
    let last_nonempty = counts.iter().rposition(|&c| c > 0).unwrap_or(0);

    counts[..=last_nonempty]
        .iter()
        .enumerate()
        .map(|(i, &count)| {
            let lo = i as u64 * bucket_size;
            let hi = lo + bucket_size - 1;
            let label = format!("{}-{}", lo, hi);
            (label, count)
        })
        .collect()
}

fn format_elapsed(secs: f64) -> String {
    let total = secs as u64;
    let mm = total / 60;
    let ss = total % 60;
    format!("{:02}:{:02}", mm, ss)
}

fn format_bandwidth(bytes_per_sec: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes_per_sec >= GB {
        format!("{:.1} GB/s", bytes_per_sec as f64 / GB as f64)
    } else if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.1} KB/s", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}

fn format_total_lines(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}
