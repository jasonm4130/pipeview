mod buffer;
mod cli;
mod format;
mod highlight;
mod pipeline;
mod stats;

use std::thread;

use clap::Parser;
use cli::Args;
use buffer::SharedBuffer;
use stats::StatsCollector;

fn main() {
    let args = Args::parse();

    let buf = SharedBuffer::new(args.buffer);
    let stats = StatsCollector::new();

    // Spawn reader thread
    let reader_buf = buf.clone_handle();
    let reader_stats = stats.clone_handle();
    let reader = thread::spawn(move || {
        pipeline::reader_thread(reader_buf, reader_stats);
    });

    // Spawn writer thread
    let writer_buf = buf.clone_handle();
    let writer = thread::spawn(move || {
        pipeline::writer_thread(writer_buf);
    });

    if args.quiet {
        // Quiet mode: wait for completion, print summary
        reader.join().unwrap();
        writer.join().unwrap();

        let snap = stats.snapshot();
        let bytes_str = format_bytes(snap.total_bytes);
        let elapsed = snap.elapsed_secs;
        let rate = if elapsed > 0.0 { snap.total_bytes as f64 / elapsed } else { 0.0 };
        let rate_str = format_bytes(rate as u64);
        eprintln!(
            "pipeview: {} lines | {} | {:.1}s | {}/s",
            format_number(snap.total_lines),
            bytes_str,
            elapsed,
            rate_str,
        );
    } else {
        // TUI mode: placeholder until Task 8
        reader.join().unwrap();
        writer.join().unwrap();
        let snap = stats.snapshot();
        eprintln!(
            "pipeview: {} lines | {:.1}s (TUI not yet implemented)",
            snap.total_lines,
            snap.elapsed_secs,
        );
    }
}

fn format_number(n: u64) -> String {
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

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
