use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use clap::Parser;
use pipespy::buffer::SharedBuffer;
use pipespy::cli::Args;
use pipespy::stats::StatsCollector;
use pipespy::{pipeline, tui};

fn main() {
    let args = Args::parse();

    let buf = SharedBuffer::new(args.buffer);
    let stats = StatsCollector::new();
    let done = Arc::new(AtomicBool::new(false));

    // Spawn reader thread
    let reader_buf = buf.clone_handle();
    let reader_stats = stats.clone_handle();
    let reader_done = Arc::clone(&done);
    let reader = thread::spawn(move || {
        pipeline::reader_thread(reader_buf, reader_stats);
        reader_done.store(true, Ordering::Relaxed);
    });

    // Spawn writer thread.
    // In TUI mode, if stdout is a terminal, discard output to avoid mixing
    // data with the TUI. Data is still visible in the TUI sample viewer.
    let stdout_is_tty = std::io::stdout().is_terminal();
    let writer_buf = buf.clone_handle();
    let writer = thread::spawn(move || {
        if !args.quiet && stdout_is_tty {
            pipeline::discard_thread(writer_buf);
        } else {
            pipeline::writer_thread(writer_buf);
        }
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
            "pipespy: {} lines | {} | {:.1}s | {}/s",
            format_number(snap.total_lines),
            bytes_str,
            elapsed,
            rate_str,
        );
    } else {
        // TUI mode
        let tui_buf = buf.clone_handle();
        let tui_stats = stats.clone_handle();
        let tui_done = Arc::clone(&done);
        tui::run_tui(
            tui_buf,
            tui_stats,
            tui_done,
            args.fullscreen,
            args.json,
            args.csv,
            args.no_detect,
        );

        reader.join().unwrap();
        writer.join().unwrap();
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
