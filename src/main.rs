mod buffer;
mod cli;
mod format;
mod stats;

use clap::Parser;
use cli::Args;

fn main() {
    let args = Args::parse();
    eprintln!("pipeview: args={args:?}");
}
