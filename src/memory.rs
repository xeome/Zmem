use super::*;
use colored::Colorize;
use std::{
    fmt::{self, Display},
    fs,
};

#[derive(Default, Clone)]
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
    /// ### Update
    /// Uses `/proc/meminfo` to get the memory stats
    pub fn update(&mut self) -> Result {
        for line in fs::read_to_string("/proc/meminfo")?.lines() {
            // Split the line into key and value
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
}

impl Display for MemoryStats {
    /// ### Display
    /// Displays the memory stats in a human readable format:
    ///
    /// ```txt
    ///             total            used            free          shared      buff/cache       available
    ///Mem:      7.15 GB         4.91 GB       340.04 MB       122.59 MB         1.91 GB         1.98 GB
    ///Swap:     9.77 GB         2.17 GB         7.60 GB                       256.40 MB         7.60 GB
    ///Total:   16.92 GB         7.08 GB         7.93 GB                                         9.58 GB
    ///
    ///            Zswap      Compressed           Ratio
    ///Zswap:    1.68 GB       764.75 MB           2.256
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt = |s: u64| format!("{:>15}", format_size(s));
        let fmt_mem = |s: u64| format!("{:>12}", format_size(s));
        let fmt_swap = |s: u64| format!("{:>11}", format_size(s));
        let fmt_total = |s: u64| format!("{:>10}", format_size(s));
        let fmt_zswap = |s: u64| format!("{:>10}", format_size(s));
        let fmt_ratio = |s: f64| format!("{:>15.3}", s);
        writeln!(
            f,
            "{:>17} {:>15} {:>15} {:>15} {:>15} {:>15}",
            "total".bold(),
            "used".bold(),
            "free".bold(),
            "shared".bold(),
            "buff/cache".bold(),
            "available".bold()
        )?;
        writeln!(
            f,
            "{} {} {} {} {} {} {}",
            "Mem:".bold().cyan(),
            fmt_mem(self.total).green(),
            fmt(self.used).red(),
            fmt(self.free).cyan(),
            fmt(self.shared).yellow(),
            fmt(self.buffers + self.cached).magenta(),
            fmt(self.available).blue()
        )?;
        writeln!(
            f,
            "{} {} {} {} {:>15} {} {}",
            "Swap:".bold().purple(),
            fmt_swap(self.swap_total).green(),
            fmt(self.swap_used).red(),
            fmt(self.swap_free).cyan(),
            "",
            fmt(self.swap_cached).yellow(),
            fmt(self.swap_available).blue()
        )?;
        writeln!(
            f,
            "{} {} {} {} {:>15} {:>15} {}",
            "Total:".bold().blue(),
            fmt_total(self.totalvmem).green(),
            fmt(self.usedvmem).red(),
            fmt(self.freevmem).cyan(),
            "",
            "",
            fmt(self.availablevmem).blue()
        )?;
        writeln!(
            f,
            "\n{:>17} {:>15} {:>15}",
            "Zswap".bold(),
            "Compressed".bold(),
            "Ratio".bold()
        )?;
        writeln!(
            f,
            "{} {} {} {}",
            "Zswap:".bold().purple(),
            fmt_zswap(self.zswap).green(),
            fmt(self.zswap_compressed).red(),
            fmt_ratio(self.compression_ratio).cyan()
        )
    }
}

#[derive(Default, Clone)]
pub struct ProcessMemoryStats {
    pub pid: u32,
    pub command: String,
    pub swap: u64,
    pub uss: u64,
    pub pss: u64,
    pub rss: u64,
}

impl ProcessMemoryStats {
    /// Update the process memory stats
    /// # Examples
    ///
    /// ```no_run
    /// let mut pms = zmem::ProcessMemoryStats::default();
    /// pms.update(1, "cmd".into()).unwrap();
    /// ```
    pub fn update(&mut self, pid: u32, mut cmd: String) -> Result {
        if cmd.len() > 50 {
            cmd.truncate(50);
        }
        self.pid = pid;
        self.command = cmd;
        self.swap = 0;
        self.uss = 0;
        self.pss = 0;
        self.rss = 0;
        for line in fs::read_to_string(format!("/proc/{pid}/smaps"))?.lines() {
            if line.starts_with("Rss:") {
                self.rss += parse_value(&line[5..])?;
            } else if line.starts_with("Pss:") {
                self.pss += parse_value(&line[5..])?;
            } else if line.starts_with("Private_Clean:") || line.starts_with("Private_Dirty:") {
                self.uss += parse_value(&line[14..])?;
            } else if line.starts_with("Swap:") {
                self.swap += parse_value(&line[6..])?;
            }
        }
        Ok(())
    }
}

impl Display for ProcessMemoryStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{:>10} {:>14} {:>14} {:>14} {:>14} {}",
            self.pid,
            format_size(self.swap).red(),
            format_size(self.uss).green(),
            format_size(self.pss).blue(),
            format_size(self.rss).cyan(),
            self.command
        )
    }
}
