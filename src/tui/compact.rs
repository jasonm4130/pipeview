use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline};

use crate::stats::StatsSnapshot;
use crate::tui::App;
use crate::highlight::highlight_line;

pub fn render(frame: &mut Frame, snap: &StatsSnapshot, samples: &[String], app: &App) {
    let area = frame.area();

    // Top-level vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Length(3),  // Stats bar
            Constraint::Length(5),  // Sparkline
            Constraint::Min(4),     // Samples
            Constraint::Length(1),  // Footer
        ])
        .split(area);

    // --- Header ---
    let elapsed_str = format_elapsed(snap.elapsed_secs);
    let header_text = format!(
        " pipeview \u{2502} stdin \u{2192} stdout  elapsed: {} \u{2502} q to detach",
        elapsed_str
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));
    frame.render_widget(header, chunks[0]);

    // --- Stats bar: 4 bordered boxes side by side ---
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);

    // THROUGHPUT (green)
    let throughput_val = format!("{:.0} lines/s", snap.throughput_lines);
    let throughput_box = Paragraph::new(throughput_val)
        .block(Block::default().title(" THROUGHPUT ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Green));
    frame.render_widget(throughput_box, stats_chunks[0]);

    // BANDWIDTH (blue)
    let bandwidth_val = format_bandwidth(snap.throughput_bytes as u64);
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

    // FORMAT (magenta)
    let format_val = app.format.to_string();
    let format_box = Paragraph::new(format_val)
        .block(Block::default().title(" FORMAT ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Magenta));
    frame.render_widget(format_box, stats_chunks[3]);

    // --- Sparkline ---
    let sparkline_data: Vec<u64> = snap
        .sparkline
        .iter()
        .map(|&v| v as u64)
        .collect();
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(" throughput history ")
                .borders(Borders::ALL),
        )
        .data(&sparkline_data)
        .style(Style::default().fg(Color::Green));
    frame.render_widget(sparkline, chunks[2]);

    // --- Samples ---
    let samples_area = chunks[3];
    let inner_height = samples_area.height.saturating_sub(2) as usize; // subtract borders
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

    let samples_list = List::new(items)
        .block(Block::default().title(" records ").borders(Borders::ALL));
    frame.render_widget(samples_list, samples_area);

    // --- Footer ---
    let is_done = app.done.load(std::sync::atomic::Ordering::Relaxed);
    let (pipe_status, dot_color) = if is_done {
        ("pipe done", Color::Yellow)
    } else {
        ("pipe healthy", Color::Green)
    };

    let footer_spans = vec![
        Span::raw(" f fullscreen \u{2502} q quit  "),
        Span::raw(pipe_status),
        Span::raw(" "),
        Span::styled("\u{25cf}", Style::default().fg(dot_color)),
    ];
    let footer = Paragraph::new(Line::from(footer_spans))
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));
    frame.render_widget(footer, chunks[4]);
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
