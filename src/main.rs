use memory::MemoryStats;
use process::Processes;

mod memory;
mod process;
mod utils;

type AnyError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    let mut mem = MemoryStats::new();
    if let Err(e) = mem.update() {
        println!("error updating memory stats: {}", e);
    }
    mem.display();

    let mut processes = Processes::new();
    if let Err(e) = processes.update().await {
        println!("error updating processes: {}", e);
    }
    processes.display();
}
