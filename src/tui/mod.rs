use std::io::stderr;
use std::os::fd::AsRawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// Enable raw mode on a specific fd (stderr) instead of stdin.
/// Returns the original termios to restore later.
fn enable_raw_mode_on_fd(fd: i32) -> std::io::Result<libc::termios> {
    unsafe {
        let mut orig: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut orig) != 0 {
            return Err(std::io::Error::last_os_error());
        }
        let mut raw = orig;
        libc::cfmakeraw(&mut raw);
        if libc::tcsetattr(fd, libc::TCSANOW, &raw) != 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(orig)
    }
}

/// Restore terminal mode from saved termios.
fn restore_terminal_mode(fd: i32, orig: &libc::termios) {
    unsafe {
        libc::tcsetattr(fd, libc::TCSANOW, orig);
    }
}

use crate::buffer::SharedBuffer;
use crate::format::{self, Format};
use crate::stats::StatsCollector;

pub mod compact;
pub mod fullscreen;

pub struct App {
    pub fullscreen: bool,
    pub format: Format,
    pub format_resolved: bool,
    pub force_json: bool,
    pub force_csv: bool,
    pub no_detect: bool,
    pub running: bool,
    pub done: Arc<AtomicBool>,
}

impl App {
    pub fn new(
        fullscreen: bool,
        force_json: bool,
        force_csv: bool,
        no_detect: bool,
        done: Arc<AtomicBool>,
    ) -> Self {
        Self {
            fullscreen,
            format: Format::PlainText,
            format_resolved: false,
            force_json,
            force_csv,
            no_detect,
            running: true,
            done,
        }
    }

    /// Once we have at least 4 samples, resolve the format once and lock it in.
    pub fn resolve_format(&mut self, samples: &[String]) {
        if !self.format_resolved && samples.len() >= 4 {
            self.format = format::resolve(self.force_json, self.force_csv, self.no_detect, samples);
            self.format_resolved = true;
        }
    }
}

pub fn run_tui(
    buffer: SharedBuffer,
    stats: StatsCollector,
    done: Arc<AtomicBool>,
    fullscreen: bool,
    force_json: bool,
    force_csv: bool,
    no_detect: bool,
) {
    // Enable raw mode on stderr (not stdin, which is the data pipe)
    let stderr_fd = stderr().as_raw_fd();
    let orig_termios = enable_raw_mode_on_fd(stderr_fd).expect("failed to enable raw mode on stderr");
    let mut stderr_handle = stderr();
    execute!(stderr_handle, EnterAlternateScreen).expect("failed to enter alternate screen");

    let backend = CrosstermBackend::new(stderr_handle);
    let mut terminal = Terminal::new(backend).expect("failed to create terminal");

    let mut app = App::new(fullscreen, force_json, force_csv, no_detect, Arc::clone(&done));

    let tick_interval = Duration::from_millis(500);
    let mut last_tick = Instant::now();

    while app.running {
        let elapsed = last_tick.elapsed();
        if elapsed >= tick_interval {
            stats.tick(elapsed.as_secs_f64());
            last_tick = Instant::now();
        }

        let snap = stats.snapshot();
        let samples = buffer.get_samples();

        app.resolve_format(&samples);

        terminal
            .draw(|frame| {
                if app.fullscreen {
                    fullscreen::render(frame, &snap, &samples, &app);
                } else {
                    compact::render(frame, &snap, &samples, &app);
                }
            })
            .expect("failed to draw frame");

        // Poll for keyboard input with a 50ms timeout
        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Char('q') => {
                        app.running = false;
                    }
                    KeyCode::Char('f') => {
                        app.fullscreen = !app.fullscreen;
                    }
                    _ => {}
                }
            }
        }

        // Also exit if the pipeline is done
        if app.done.load(Ordering::Relaxed) && !app.running {
            break;
        }
    }

    // Restore terminal
    restore_terminal_mode(stderr_fd, &orig_termios);
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .expect("failed to leave alternate screen");
    terminal.show_cursor().expect("failed to show cursor");

    // Print summary to stderr
    let snap = stats.snapshot();
    let elapsed = snap.elapsed_secs;
    let rate = if elapsed > 0.0 {
        snap.total_bytes as f64 / elapsed
    } else {
        0.0
    };
    eprintln!(
        "pipeview: {} lines | {} | {:.1}s | {}/s",
        format_number(snap.total_lines),
        format_bytes(snap.total_bytes),
        elapsed,
        format_bytes(rate as u64),
    );
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
