use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "pipeview")]
#[command(version)]
#[command(about = "Real-time pipeline debugger — pv shows bytes, pipeview shows your data")]
pub struct Args {
    /// Start in fullscreen mode
    #[arg(short, long)]
    pub fullscreen: bool,

    /// Show 1 in N records (default: auto based on throughput)
    #[arg(short = 'n', long)]
    pub sample_rate: Option<usize>,

    /// Ring buffer size in bytes (default: 8MB)
    #[arg(short, long, default_value = "8388608")]
    pub buffer: usize,

    /// Skip format detection, treat as plain text
    #[arg(long)]
    pub no_detect: bool,

    /// Force JSON mode
    #[arg(long, conflicts_with = "csv", conflicts_with = "no_detect")]
    pub json: bool,

    /// Force CSV mode
    #[arg(long, conflicts_with = "json", conflicts_with = "no_detect")]
    pub csv: bool,

    /// No TUI, just print summary stats on completion
    #[arg(short, long)]
    pub quiet: bool,
}
