use colored::Colorize;
use std::fs::File;
use std::io::Read;

#[derive(Clone, Copy)]
pub struct MemoryStats {
    pub total: u64,
    pub free: u64,
    pub available: u64,
    pub used: u64,
    pub shared: u64,
    pub buffers: u64,
    pub cached: u64,
    pub swap_total: u64,
    pub swap_free: u64,
    pub zswap: u64,
    pub zswap_compressed: u64,
    pub swap_cached: u64,
    pub compression_ratio: f64,
    pub swap_used: u64,
    pub swap_available: u64,
}

impl MemoryStats {
    pub fn new() -> Self {
        MemoryStats {
            total: 0,
            free: 0,
            available: 0,
            used: 0,
            shared: 0,
            buffers: 0,
            cached: 0,
            swap_total: 0,
            swap_free: 0,
            swap_used: 0,
            swap_available: 0,
            zswap: 0,
            zswap_compressed: 0,
            swap_cached: 0,
            compression_ratio: 0.0,
        }
    }
    pub fn update(&mut self) {
        let mut file = File::open("/proc/meminfo").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        for line in lines {
            let mut split = line.split_whitespace();
            let key = split.next().unwrap();
            let value = split.next().unwrap();
            match key {
                "MemTotal:" => self.total = value.parse().unwrap(),
                "MemFree:" => self.free = value.parse().unwrap(),
                "MemAvailable:" => self.available = value.parse().unwrap(),
                "MemUsed:" => self.used = value.parse().unwrap(),
                "Shmem:" => self.shared = value.parse().unwrap(),
                "Buffers:" => self.buffers = value.parse().unwrap(),
                "Cached:" => self.cached = value.parse().unwrap(),
                "SwapTotal:" => self.swap_total = value.parse().unwrap(),
                "SwapFree:" => self.swap_free = value.parse().unwrap(),
                "Zswap:" => self.zswap_compressed = value.parse().unwrap(),
                "Zswapped:" => self.zswap = value.parse().unwrap(),
                "SwapCached:" => self.swap_cached = value.parse().unwrap(),
                _ => (),
            }
        }
        self.used = self.total - self.free - self.buffers - self.cached;
        self.swap_used = self.swap_total - self.swap_free;
        self.swap_available = self.swap_total - self.swap_used;
        self.compression_ratio = self.zswap as f64 / self.zswap_compressed as f64;
    }
    // Display the memory stats. in Mb, in a free -m like format. right aligned. Colors are used to highlight the most important stats. Depending on the value of the stat, the color will change.
    //                total        used        free      shared  buff/cache   available
    // Mem:            7321        4861         899          33        1561        2172
    // Swap:           9999        2602        7397
    pub fn display(&self) {
        let total = format!("{:>9}", self.total / 1024);
        let used = format!("{:>14}", self.used / 1024);
        let free = format!("{:>14}", self.free / 1024);
        let shared = format!("{:>14}", self.shared / 1024);
        let cached = format!("{:>14}", self.cached / 1024);
        let available = format!("{:>14}", self.available / 1024);
        let swap_total = format!("{:>8}", self.swap_total / 1024);
        let swap_used = format!("{:>14}", self.swap_used / 1024);
        let swap_free = format!("{:>14}", self.swap_free / 1024);
        let swap_available = format!("{:>14}", self.swap_available / 1024);
        let zswap = format!("{:>7}", self.zswap / 1024);
        let zswap_compressed = format!("{:>14}", self.zswap_compressed / 1024);
        let swap_cached = format!("{:>14}", self.swap_cached / 1024);
        let compression_ratio = format!("{:>14.3}", self.compression_ratio);
        println!(
            "{:>14} {:>14} {:>14} {:>14} {:>14} {:>14}",
            "total".bold(),
            "used".bold(),
            "free".bold(),
            "shared".bold(),
            "buff/cache".bold(),
            "available".bold()
        );
        println!(
            "{} {} {} {} {} {} {}",
            "Mem:".bold().cyan(),
            total.green(),
            used.red(),
            free.cyan(),
            shared,
            cached.yellow(),
            available.blue()
        );
        println!(
            "{} {} {} {} {:>14} {} {}",
            "Swap:".bold().purple(),
            swap_total.green(),
            swap_used.red(),
            swap_free.cyan(),
            "",
            swap_cached.yellow(),
            swap_available.blue()
        );
        println!(
            "\n{:>14} {:>14} {:>14}",
            "Zswap".bold(),
            "Compressed".bold(),
            "Ratio".bold()
        );
        println!(
            "{} {} {} {}",
            "Zswap:".bold().purple(),
            zswap,
            zswap_compressed,
            compression_ratio
        );
    }
}
