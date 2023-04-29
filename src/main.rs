use zmem::*;

fn main() {
    let mut mem = MemoryStats::default();
    let mut processes = Processes::default();

    // let time = std::time::Instant::now();

    std::thread::scope(|s| {
        s.spawn(|| {
            if let Err(err) = mem.update() {
                println!("error updating memory stats: {err}");
            }
            println!("{mem}");
        });
        s.spawn(|| {
            if let Err(err) = processes.update() {
                println!("error updating processes: {err}");
            }
            processes.sort_by_swap();
            println!("{processes}");
        });
    });

    // println!("{:?}", time.elapsed());
}
