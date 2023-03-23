use memory::MemoryStats;

mod memory;

fn main() {
    let mut mem = MemoryStats::new();
    mem.update();
    mem.display();
}
