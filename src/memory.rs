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
        for line in contents.lines() {
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
    pub fn format_size(size: u64) -> String {
        let mut size = size as f64;
        let mut unit = "kB";
        if size > 1024.0 {
            size /= 1024.0;
            unit = "MB";
        }
        if size > 1024.0 {
            size /= 1024.0;
            unit = "GB";
        }
        if size > 1024.0 {
            size /= 1024.0;
            unit = "TB";
        }

        format!("{:.2} {}", size, unit)
    }
    pub fn display(&self) {
        let fmt = |s: u64| format!("{:>15}", Self::format_size(s));
        let fmt_mem = |s: u64| format!("{:>12}", Self::format_size(s));
        let fmt_swap = |s: u64| format!("{:>11}", Self::format_size(s));
        let fmt_total = |s: u64| format!("{:>10}", Self::format_size(s));
        let fmt_zswap = |s: u64| format!("{:>10}", Self::format_size(s));
        let fmt_ratio = |s: f64| format!("{:>15.3}", s);

        println!(
            "{:>17} {:>15} {:>15} {:>15} {:>15} {:>15}",
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
            fmt_mem(self.total).green(),
            fmt(self.used).red(),
            fmt(self.free).cyan(),
            fmt(self.shared).yellow(),
            fmt(self.buffers + self.cached).magenta(),
            fmt(self.available).blue()
        );
        println!(
            "{} {} {} {} {:>15} {} {}",
            "Swap:".bold().purple(),
            fmt_swap(self.swap_total).green(),
            fmt(self.swap_used).red(),
            fmt(self.swap_free).cyan(),
            "",
            fmt(self.swap_cached).yellow(),
            fmt(self.swap_available).blue()
        );
        println!(
            "{} {} {} {} {:>15} {:>15} {}",
            "Total:".bold().blue(),
            fmt_total(self.totalvmem).green(),
            fmt(self.usedvmem).red(),
            fmt(self.freevmem).cyan(),
            "",
            "",
            fmt(self.availablevmem).blue()
        );
        println!(
            "\n{:>17} {:>15} {:>15}",
            "Zswap".bold(),
            "Compressed".bold(),
            "Ratio".bold()
        );
        println!(
            "{} {} {} {}",
            "Zswap:".bold().purple(),
            fmt_zswap(self.zswap).green(),
            fmt(self.zswap_compressed).red(),
            fmt_ratio(self.compression_ratio).cyan()
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

    pub fn get_smaps(pid: u32) -> Result<String, AnyError> {
        let mut file = File::open(format!("/proc/{}/smaps", pid))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub fn update(&mut self, pid: &u32) -> Result<(), AnyError> {
        self.command = Process::get_cmd(*pid)?;
        if self.command.len() > 50 {
            self.command.truncate(50);
        }

        self.pid = *pid;
        let contents = Self::get_smaps(*pid)?;
        let prefixes = ["Swap", "Pss", "Rss", "Private_Clean", "Private_Dirty"];
        for line in contents
            .lines()
            .filter(|line| prefixes.iter().any(|prefix| line.starts_with(prefix)))
        {
            let mut split = line.split_whitespace();
            let key = split.next().ok_or("bad file format")?;
            let value = split.next().ok_or("bad file format")?;
            match key {
                "Swap:" => self.swap += value.parse::<u64>()?,
                "Pss:" => self.pss += value.parse::<u64>()?,
                "Rss:" => self.rss += value.parse::<u64>()?,
                "Private_Clean:" | "Private_Dirty:" => self.uss += value.parse::<u64>()?,

                _ => (),
            }
        }

        Ok(())
    }

    pub fn display(&self) {
        let fmt = |s: String| format!("{:>14}", s);

        println!(
            "{:>10} {} {} {} {} {}",
            self.pid,
            fmt(MemoryStats::format_size(self.swap)).red(),
            fmt(MemoryStats::format_size(self.uss)).green(),
            fmt(MemoryStats::format_size(self.pss)).blue(),
            fmt(MemoryStats::format_size(self.rss)).cyan(),
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
        process.update().unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
        });
        process
    }

    pub fn get_cmd(pid: u32) -> Result<String, AnyError> {
        let mut file = File::open(format!("/proc/{}/cmdline", pid))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents.replace('\0', " "))
    }

    pub fn update(&mut self) -> Result<(), AnyError> {
        self.command = Self::get_cmd(self.pid)?;
        self.memory.update(&self.pid)?;
        Ok(())
    }
}

pub struct Processes {
    processes: Vec<Process>,
}

impl Processes {
    pub fn new() -> Self {
        Self { processes: vec![] }
    }

    fn can_read_file(path: &str) -> bool {
        File::open(path).is_ok()
    }

    pub fn update(&mut self) -> Result<(), AnyError> {
        let mut processes = vec![];
        for entry in fs::read_dir("/proc")? {
            let entry = entry?;
            let pid = entry
                .file_name()
                .into_string()
                .unwrap_or_else(|_| "".to_string());
            if let Ok(pid) = pid.parse::<u32>() {
                // Skip processes with no command, and the ones that we can't get smaps
                if let Ok(command) = Process::get_cmd(pid) {
                    if !command.is_empty() && Self::can_read_file(&format!("/proc/{}/smaps", pid)) {
                        processes.push(Process::new(pid));
                    }
                }
            }
        }
        processes.sort_by(|a, b| a.memory.swap.cmp(&b.memory.swap));
        self.processes = processes;
        Ok(())
    }

    pub fn display(&self) {
        println!(
            "\n{:>10} {:>14} {:>14} {:>14} {:>14} {:>14}",
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
