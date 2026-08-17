use colored::Colorize;
use clap::ValueEnum;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use crate::memory::ProcessMemoryStats;
use crate::utils::{format_size, get_cmd};
use crate::AnyError;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum SortColumn {
    Swap,
    Uss,
    Pss,
    Rss,
}

/// Truncates `s` to at most `max_len` bytes without splitting a multi-byte UTF-8 char.
/// `String::truncate` panics on a non-char-boundary index, which a raw byte-length
/// cutoff on a cmdline (arbitrary bytes from the OS) can easily hit.
fn truncate_char_boundary(s: &mut String, max_len: usize) {
    if s.len() > max_len {
        let mut end = max_len;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        s.truncate(end);
    }
}

pub struct Process {
    pid: u32,
    command: String,
    memory: ProcessMemoryStats,
}

impl Process {
    pub fn new(pid: u32) -> Result<Self, AnyError> {
        let mut process = Self {
            pid,
            command: String::new(),
            memory: ProcessMemoryStats::new(),
        };
        process.update()?;
        Ok(process)
    }

    pub fn update(&mut self) -> Result<(), AnyError> {
        self.memory.update(&self.pid)?;
        self.command = get_cmd(self.pid)?;
        truncate_char_boundary(&mut self.command, 50);
        Ok(())
    }

    pub fn display(&self) {
        let fmt = |s: String| format!("{:>14}", s);

        println!(
            "{:>10} {} {} {} {} {}",
            self.pid,
            fmt(format_size(self.memory.swap)).red(),
            fmt(format_size(self.memory.uss)).green(),
            fmt(format_size(self.memory.pss)).blue(),
            fmt(format_size(self.memory.rss)).cyan(),
            self.command
        );
    }
}

pub struct Processes {
    processes: Vec<Process>,
}

impl Processes {
    pub fn new() -> Self {
        Self { processes: vec![] }
    }

    /// Update the processes
    /// Reads each process's memory stats in parallel across a pool of OS threads sized to
    /// the CPU count. Reading smaps_rollup walks the target's page tables under its mmap
    /// lock, so cost per pid varies a lot (a browser tab vs. a shell); workers pull the
    /// next pid from a shared cursor instead of owning a fixed slice, so one thread stuck
    /// on a few heavy processes doesn't leave the others idle.
    ///
    /// # Examples
    /// ```
    /// let mut processes = Processes::new();
    /// processes.update(SortColumn::Swap)?;
    /// ```
    pub fn update(&mut self, sort_by: SortColumn) -> Result<(), AnyError> {
        let pids: Vec<u32> = fs::read_dir("/proc")?
            .filter_map(|entry| entry.ok()?.file_name().to_string_lossy().parse().ok())
            .collect();

        let worker_count = thread::available_parallelism().map_or(1, |n| n.get());
        let next = AtomicUsize::new(0);

        self.processes = thread::scope(|scope| {
            let handles: Vec<_> = (0..worker_count)
                .map(|_| {
                    let next = &next;
                    let pids = &pids;
                    scope.spawn(move || {
                        let mut found = Vec::new();
                        while let Some(&pid) = pids.get(next.fetch_add(1, Ordering::Relaxed)) {
                            if let Ok(process) = Process::new(pid) {
                                found.push(process);
                            }
                        }
                        found
                    })
                })
                .collect();

            // A panicked worker thread is dropped rather than failing the whole batch.
            handles
                .into_iter()
                .filter_map(|h| h.join().ok())
                .flatten()
                .collect()
        });
        // Sort the processes by selected memory column
        self.processes
            .sort_by_key(|p| process_sort_key(&p.memory, sort_by));
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
            process.display();
        }
    }
}

fn process_sort_key(memory: &ProcessMemoryStats, sort_by: SortColumn) -> u64 {
    match sort_by {
        SortColumn::Swap => memory.swap,
        SortColumn::Uss => memory.uss,
        SortColumn::Pss => memory.pss,
        SortColumn::Rss => memory.rss,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_sort_key_uses_requested_column() {
        let memory = ProcessMemoryStats {
            swap: 1,
            uss: 2,
            pss: 3,
            rss: 4,
        };

        assert_eq!(process_sort_key(&memory, SortColumn::Swap), 1);
        assert_eq!(process_sort_key(&memory, SortColumn::Uss), 2);
        assert_eq!(process_sort_key(&memory, SortColumn::Pss), 3);
        assert_eq!(process_sort_key(&memory, SortColumn::Rss), 4);
    }

    #[test]
    fn truncate_does_not_split_multibyte_char() {
        // Byte 50 lands in the middle of a multi-byte char; must not panic.
        let mut s = "a".repeat(49) + "日本語";
        truncate_char_boundary(&mut s, 50);
        assert!(s.len() <= 50);
        assert!(s.is_char_boundary(s.len()));
    }
}
