use colored::{ColoredString, Colorize};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::utils::{format_size, parse_value};
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
    pub zram: Option<ZramStats>,
}

/// Every zram swap device, summed into one row. kB, to match meminfo.
#[derive(Default, Clone, Copy)]
pub struct ZramStats {
    pub stored: u64,
    pub compressed: u64,
    pub ratio: f64,
    /// Slot count rather than `orig_data_size`: the kernel holds swap slots it has
    /// handed zram no data for (`swapon` and `zramctl` disagree by that much), and
    /// a slot with no data anywhere is not on a disk either.
    pub in_ram: u64,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// ### Update
    /// Uses `/proc/meminfo`, plus `/proc/swaps` and zram sysfs for the on-disk split
    pub fn update(&mut self) -> Result<(), AnyError> {
        self.zram = read_zram_stats();
        let zram_in_ram = self.zram.map_or(0, |z| z.in_ram);
        let contents = fs::read_to_string("/proc/meminfo")?;
        self.update_from_meminfo(&contents, zram_in_ram)
    }

    /// Parses the contents of `/proc/meminfo` and derives the rest of the stats.
    /// Split out from `update` so the derived math is testable without the filesystem.
    fn update_from_meminfo(&mut self, contents: &str, zram_in_ram: u64) -> Result<(), AnyError> {
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
        self.used = self
            .total
            .saturating_sub(self.free)
            .saturating_sub(self.buffers)
            .saturating_sub(self.cached);
        self.swap_used = self.swap_total.saturating_sub(self.swap_free);
        self.swap_available = self.swap_free;
        self.compression_ratio = if self.zswap_compressed == 0 {
            0.0
        } else {
            self.zswap as f64 / self.zswap_compressed as f64
        };
        self.totalvmem = self.total + self.swap_total;
        self.freevmem = self.free + self.swap_free;
        self.usedvmem = self.used + self.swap_used;
        self.availablevmem = self.available + self.swap_available;
        // Disjoint even when zswap fronts zram: zswap_store returns before
        // __swap_writepage, so zram never sees a zswapped page.
        self.swap_on_disk = self
            .swap_used
            .saturating_sub(self.zswap)
            .saturating_sub(zram_in_ram);

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
    ///           Stored      Compressed           Ratio         On Disk
    ///Zswap:    1.68 GB       764.75 MB           2.256         0.00 kB
    ///Zram:     4.20 GB         1.48 GB           2.838
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

        #[allow(clippy::too_many_arguments)]
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
                fmt("Stored".to_string()).bold(),
                fmt("Compressed".to_string()).bold(),
                fmt("Ratio".to_string()).bold(),
                fmt("On Disk".to_string()).bold(),
            );
        }

        print_zswap_header();

        fn print_z(name: &str, stored: u64, compressed: u64, ratio: f64, on_disk: Option<u64>) {
            println!(
                "{:<8} {} {} {} {}",
                name.magenta().bold(),
                fmt(format_size(stored)).green(),
                fmt(format_size(compressed)).red(),
                fmt(format!("{:.3}", ratio)).blue(),
                fmt(on_disk.map_or(String::new(), format_size)).yellow(),
            );
        }

        print_z(
            "Zswap:",
            self.zswap,
            self.zswap_compressed,
            self.compression_ratio,
            Some(self.swap_on_disk),
        );

        if let Some(zram) = self.zram {
            print_z("Zram:", zram.stored, zram.compressed, zram.ratio, None);
        }

        println!();
    }
}

/// zram devices in `/proc/swaps`, paired with their used kB. Devices backing a
/// filesystem aren't listed there, so they're excluded for free.
fn zram_swap_devices(swaps: &str) -> Vec<(&str, u64)> {
    swaps
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let device = fields.next()?;
            let digits = device.strip_prefix("/dev/zram")?;
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            // Type, Size, Used
            let used = fields.nth(2)?.parse().ok()?;
            Some((&device["/dev/".len()..], used))
        })
        .collect()
}

/// `orig_data_size` and `mem_used_total` from `mm_stat`, both bytes. Skips
/// `compr_data_size` between them: it excludes zsmalloc overhead, so it wouldn't
/// compare against meminfo's `Zswap:`.
fn parse_mm_stat(contents: &str) -> Option<(u64, u64)> {
    let mut fields = contents.split_whitespace();
    let orig = fields.next()?.parse().ok()?;
    let mem_used_total = fields.nth(1)?.parse().ok()?;
    Some((orig, mem_used_total))
}

/// `None` when zram isn't in use as swap.
fn read_zram_stats() -> Option<ZramStats> {
    let swaps = fs::read_to_string("/proc/swaps").ok()?;
    let devices = zram_swap_devices(&swaps);
    if devices.is_empty() {
        return None;
    }

    let mut stats = ZramStats::default();
    for (name, used) in devices {
        // Pages evicted to a backing_dev really are on a disk. orig_data_size still
        // counts them (the kernel re-increments pages_stored after writeback), so
        // bd_stat is the only place eviction shows. Absent without
        // CONFIG_ZRAM_WRITEBACK, and in 4K units rather than mm_stat's bytes.
        let written_back = fs::read_to_string(format!("/sys/block/{}/bd_stat", name))
            .ok()
            .and_then(|bd| bd.split_whitespace().next()?.parse::<u64>().ok())
            .unwrap_or(0)
            * 4;
        stats.in_ram += used.saturating_sub(written_back);

        let mm_stat =
            fs::read_to_string(format!("/sys/block/{}/mm_stat", name)).unwrap_or_default();
        if let Some((orig, mem_used_total)) = parse_mm_stat(&mm_stat) {
            stats.stored += (orig / 1024).saturating_sub(written_back);
            stats.compressed += mem_used_total / 1024;
        }
    }

    stats.ratio = if stats.compressed == 0 {
        0.0
    } else {
        stats.stored as f64 / stats.compressed as f64
    };
    Some(stats)
}

#[derive(Default, Clone)]
pub struct ProcessMemoryStats {
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
}
