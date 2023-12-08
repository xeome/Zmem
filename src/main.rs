use clap::Parser;

use memory::MemoryStats;
use process::Processes;

mod memory;
mod process;
mod utils;

type AnyError = Box<dyn std::error::Error + Send + Sync>;

/// Zmem is a Linux memory monitoring program that displays detailed information about virtual memory.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Display per-process memory usage or not
    /// (default: false)
    #[clap(short, long)]
    per_process: bool,
    summary: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.summary {
        let mut mem = MemoryStats::new();
        if let Err(e) = mem.update() {
            println!("error updating memory stats: {}", e);
        }
        mem.display();
    }

    if args.per_process {
        let mut processes = Processes::new();
        if let Err(e) = processes.update().await {
            println!("error updating processes: {}", e);
        }

        processes.display();
    }
}
