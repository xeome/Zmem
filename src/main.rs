use memory::MemoryStats;
use memory::ProcessMemoryStats;

mod memory;

type AnyError = Box<dyn std::error::Error + Send + Sync>;

fn main() {
    let mut mem = MemoryStats::new();
    if let Err(e) = mem.update() {
        println!("error updating memory stats: {}", e);
    }
    mem.display();
}
