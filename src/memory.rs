use crate::AnyError;
use colored::Colorize;
use std::fs;
use std::fs::File;
use std::io::Read;

#[derive(Default, Clone, Copy)]
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
    pub totalvmem: u64,
    pub freevmem: u64,
    pub usedvmem: u64,
    pub availablevmem: u64,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self) -> Result<(), AnyError> {
        let mut file = File::open("/proc/meminfo")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let lines: Vec<&str> = contents.lines().collect();
        for line in lines {
            let mut split = line.split_whitespace();
            let key = split.next().ok_or("bad file format")?;
            let value = split.next().ok_or("bad file format")?;
            match key {
                "MemTotal:" => self.total = value.parse()?,
                "MemFree:" => self.free = value.parse()?,
                "MemAvailable:" => self.available = value.parse()?,
                "MemUsed:" => self.used = value.parse()?,
                "Shmem:" => self.shared = value.parse()?,
                "Buffers:" => self.buffers = value.parse()?,
                "Cached:" => self.cached = value.parse()?,
                "SwapTotal:" => self.swap_total = value.parse()?,
                "SwapFree:" => self.swap_free = value.parse()?,
                "Zswap:" => self.zswap_compressed = value.parse()?,
                "Zswapped:" => self.zswap = value.parse()?,
                "SwapCached:" => self.swap_cached = value.parse()?,
                _ => (),
            }
        }
        self.used = self.total - self.free - self.buffers - self.cached;
        self.swap_used = self.swap_total - self.swap_free;
        self.swap_available = self.swap_total - self.swap_used;
        self.compression_ratio = self.zswap as f64 / self.zswap_compressed as f64;
        self.totalvmem = self.total + self.swap_total;
        self.freevmem = self.free + self.swap_free;
        self.usedvmem = self.used + self.swap_used;
        self.availablevmem = self.available + self.swap_available;

        Ok(())
    }

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
            "{} {} {} {} {:>14} {:>14} {}",
            "Total:".bold().blue(),
            format!("{:>7}", self.totalvmem / 1024).green(),
            format!("{:>14}", self.usedvmem / 1024).red(),
            format!("{:>14}", self.freevmem / 1024).cyan(),
            "",
            "",
            format!("{:>14}", self.availablevmem / 1024).blue()
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

#[derive(Default, Clone)]
pub struct ProcessMemoryStats {
    pub pid: u32,
    pub username: String,
    pub command: String,
    pub swap: u64,
    pub uss: u64,
    pub pss: u64,
    pub rss: u64,
}

impl ProcessMemoryStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, pid: u32) -> Result<(), AnyError> {
        let mut file = File::open(format!("/proc/{}/cmdline", pid))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        self.command = contents.replace("\0", " ");
        if self.command.len() > 50 {
            self.command.truncate(50);
        }
        self.pid = pid;
        let mut file = File::open(format!("/proc/{}/smaps", pid))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let lines: Vec<&str> = contents.lines().collect();
        for line in lines {
            let mut split = line.split_whitespace();
            let key = split.next().ok_or("bad file format")?;
            let value = split.next().ok_or("bad file format")?;
            match key {
                "Swap:" => self.swap += value.parse::<u64>()?,
                "Pss:" => self.pss = value.parse()?,
                "Rss:" => self.rss = value.parse()?,
                "Private_Clean:" => self.uss += value.parse::<u64>()?,
                "Private_Dirty:" => self.uss += value.parse::<u64>()?,
                _ => (),
            }
        }

        Ok(())
    }

    pub fn display(&self) {
        let swap = format!("{:>14}", self.swap / 1024);
        let uss = format!("{:>14}", self.uss / 1024);
        let pss = format!("{:>14}", self.pss / 1024);
        let rss = format!("{:>14}", self.rss / 1024);

        println!(
            "{:>14} {} {} {} {} {}",
            self.pid,
            swap.red(),
            uss.blue(),
            pss.green(),
            rss.cyan(),
            self.command
        );
    }
}

pub struct Process {
    pid: u32,
    command: String,
    memory: ProcessMemoryStats,
}

impl Process {
    pub fn new(pid: u32) -> Self {
        let mut process = Self {
            pid,
            command: String::new(),
            memory: ProcessMemoryStats::new(),
        };
        process.update().unwrap();
        process
    }

    pub fn update(&mut self) -> Result<(), AnyError> {
        self.memory.update(self.pid)?;
        let mut file = File::open(format!("/proc/{}/cmdline", self.pid))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        self.command = contents.replace("\0", " ");
        Ok(())
    }
}

pub struct Processes {
    processes: Vec<Process>,
}

impl Processes {
    pub fn new() -> Self {
        let mut processes = Self { processes: vec![] };
        processes.update().unwrap();
        processes
    }

    pub fn update(&mut self) -> Result<(), AnyError> {
        let mut processes = vec![];
        for entry in fs::read_dir("/proc")? {
            let entry = entry?;
            let pid = entry.file_name().into_string().unwrap();
            if let Ok(pid) = pid.parse::<u32>() {
                processes.push(Process::new(pid));
            }
        }
        processes.sort_by(|a, b| a.memory.swap.cmp(&b.memory.swap));
        self.processes = processes;
        Ok(())
    }

    pub fn display(&self) {
        println!(
            "\n{:>14} {:>14} {:>14} {:>14} {:>14} {:>14}",
            "PID".bold(),
            "Swap".bold(),
            "USS".bold(),
            "PSS".bold(),
            "RSS".bold(),
            "COMMAND".bold()
        );
        for process in &self.processes {
            process.memory.display();
        }
    }
}
