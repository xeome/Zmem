use clap::Parser;

use memory::MemoryStats;
use process::{Processes, SortColumn};

mod memory;
mod process;
mod utils;

type AnyError = Box<dyn std::error::Error + Send + Sync>;

/// Zmem is a Linux memory monitoring program that displays detailed information about virtual memory.
#[derive(Parser, Debug)]
#[command(author, version = env!("ZMEM_VERSION"), about, long_about = None)]
struct Args {
    /// Display per-process memory usage or not
    /// (default: false)
    #[clap(short, long)]
    per_process: bool,
    /// Sort per-process output by memory column
    /// (default: swap)
    #[clap(long, value_enum, default_value_t = SortColumn::Swap)]
    sort_by: SortColumn,
}

fn main() {
    let args = Args::parse();

    let mut mem = MemoryStats::new();
    if let Err(e) = mem.update() {
        println!("error updating memory stats: {}", e);
    }
    mem.display();

    if args.per_process {
        let mut processes = Processes::new();
        if let Err(e) = processes.update(args.sort_by) {
            println!("error updating processes: {}", e);
        }

        processes.display();
    }
}
