use colored::Colorize;
use std::fs;
use tokio::task;

use crate::memory::ProcessMemoryStats;
use crate::utils::{can_read_file, get_cmd};
use crate::AnyError;

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

    pub fn update(&mut self) -> Result<(), AnyError> {
        self.command = get_cmd(self.pid)?;
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

    /// Update the processes
    /// Uses a thread pool to get the memory stats for each process that has a command and can be read from /proc
    ///
    /// # Examples
    /// ```
    /// let mut processes = Processes::new();
    /// processes.update()?;
    /// ```
    pub async fn update(&mut self) -> Result<(), AnyError> {
        let processes = fs::read_dir("/proc")?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                // Try to parse the file name as a pid
                let pid = entry.file_name().to_string_lossy().parse::<u32>().ok()?;
                let command = get_cmd(pid).ok()?;
                // Only add the process if it has a command and we can read the smaps_rollup
                if command.is_empty() || !can_read_file(&format!("/proc/{}/smaps_rollup", pid)) {
                    return None;
                }
                Some(task::spawn(async move { Process::new(pid) }))
            })
            .collect::<Vec<_>>();

        // Wait for all the processes to finish
        let mut processes = futures::future::try_join_all(processes).await?;
        // Sort the processes by swap usage
        processes.sort_by_key(|p| p.memory.swap);
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
