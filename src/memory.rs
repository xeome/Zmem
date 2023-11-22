use colored::{ColoredString, Colorize};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::utils::{format_size, get_cmd, parse_value};
use crate::AnyError;

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
    pub swap_on_disk: u64,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// ### Update
    /// Uses `/proc/meminfo` to get the memory stats
    pub fn update(&mut self) -> Result<(), AnyError> {
        let contents = fs::read_to_string("/proc/meminfo")?;
        for line in contents.lines() {
            // Split the line into key and value
            let mut split = line.split_whitespace();
            let (key, value) = (
                split.next().ok_or("bad file format")?,
                split.next().ok_or("bad file format")?,
            );
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
        self.swap_on_disk = self.swap_used - self.zswap;

        Ok(())
    }

    /// ### Display
    /// Displays the memory stats in a human readable format:
    /// ```
    ///            total            used            free          shared      buff/cache       available
    ///Mem:      7.15 GB         4.91 GB       340.04 MB       122.59 MB         1.91 GB         1.98 GB
    ///Swap:     9.77 GB         2.17 GB         7.60 GB                       256.40 MB         7.60 GB
    ///Total:   16.92 GB         7.08 GB         7.93 GB                                         9.58 GB
    ///
    ///            Zswap      Compressed           Ratio
    ///Zswap:    1.68 GB       764.75 MB           2.256
    /// ```
    pub fn display(&self) {
        fn fmt(s: String) -> String {
            format!("{:>13}", s)
        }

        fn print_header() {
            println!(
                "{:>8} {} {} {} {} {} {}",
                "",
                fmt("total".to_string()).bold(),
                fmt("used".to_string()).bold(),
                fmt("free".to_string()).bold(),
                fmt("shared".to_string()).bold(),
                fmt("buff/cache".to_string()).bold(),
                fmt("available".to_string()).bold(),
            );
        }

        print_header();

        fn print_row(
            name: ColoredString,
            total: u64,
            used: u64,
            free: u64,
            shared: u64,
            buffers: u64,
            cached: u64,
            available: u64,
        ) {
            println!(
                "{:<8} {} {} {} {} {} {}",
                name,
                fmt(format_size(total)).green(),
                fmt(format_size(used)).red(),
                fmt(format_size(free)).blue(),
                fmt(format_size(shared)).yellow(),
                fmt(format_size(buffers + cached)).magenta(),
                fmt(format_size(available)).blue()
            );
        }

        print_row(
            "Mem:".blue().bold(),
            self.total,
            self.used,
            self.free,
            self.shared,
            self.buffers,
            self.cached,
            self.available,
        );

        print_row(
            "Swap:".magenta().bold(),
            self.swap_total,
            self.swap_used,
            self.swap_free,
            0,
            0,
            self.swap_cached,
            self.swap_available,
        );

        print_row(
            "Total:".bold().blue(),
            self.totalvmem,
            self.usedvmem,
            self.freevmem,
            self.shared,
            self.buffers,
            self.cached,
            self.availablevmem,
        );

        println!();

        fn print_zswap_header() {
            println!(
                "{:>8} {} {} {} {}",
                "",
                fmt("Zswap".to_string()).bold(),
                fmt("Compressed".to_string()).bold(),
                fmt("Ratio".to_string()).bold(),
                fmt("On Disk".to_string()).bold(),
            );
        }

        print_zswap_header();

        fn print_zswap(name: &str, zswap: u64, compressed: u64, ratio: f64, on_disk: u64) {
            println!(
                "{:<8} {} {} {} {}",
                name.magenta().bold(),
                fmt(format_size(zswap)).green(),
                fmt(format_size(compressed)).red(),
                fmt(format!("{:.3}", ratio)).blue(),
                fmt(format_size(on_disk)).yellow(),
            );
        }

        print_zswap(
            "Zswap:",
            self.zswap,
            self.zswap_compressed,
            self.compression_ratio,
            self.swap_on_disk,
        );

        println!();
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

    /// Update the process memory stats
    /// # Examples
    /// ```
    /// let mut pms = ProcessMemoryStats::new();
    /// pms.update(1)?;
    /// ```
    pub fn update(&mut self, pid: &u32) -> Result<(), AnyError> {
        self.command = get_cmd(*pid)?;
        if self.command.len() > 50 {
            self.command.truncate(50);
        }

        self.pid = *pid;

        // This is the sum of all the smaps data but it is much more performant to get it this way.
        // Since 4.14 and requires CONFIG_PROC_PAGE_MONITOR
        let smaps_file = File::open(format!("/proc/{}/smaps_rollup", pid))?;
        let mut reader = BufReader::new(smaps_file);
        let mut line = String::new(); // Line to be reused, saves allocations, after testing it seems to save 5-10% of the time

        // rss, pss, private_clean + private_dirty (uss), swap
        // Local variables are faster than struct fields, Data locality is important
        let mut mem_values = (0, 0, 0, 0);

        while reader.read_line(&mut line)? > 0 {
            match &line[..10] {
                // lines are hardcoded to be longer than 10 chars in the kernel code so this is "safe"
                "Rss:      " => mem_values.0 = parse_value(&line[5..])?,
                "Pss:      " => mem_values.1 = parse_value(&line[5..])?,
                "Private_Cl" | "Private_Di" => mem_values.2 += parse_value(&line[14..])?,
                "Swap:     " => mem_values.3 = parse_value(&line[6..])?,
                _ => (),
            }

            line.clear();
        }

        self.rss = mem_values.0;
        self.pss = mem_values.1;
        self.uss = mem_values.2;
        self.swap = mem_values.3;

        Ok(())
    }

    pub fn display(&self) {
        let fmt = |s: String| format!("{:>14}", s);

        println!(
            "{:>10} {} {} {} {} {}",
            self.pid,
            fmt(format_size(self.swap)).red(),
            fmt(format_size(self.uss)).green(),
            fmt(format_size(self.pss)).blue(),
            fmt(format_size(self.rss)).cyan(),
            self.command
        );
    }
}
