use colored::Colorize;
use std::fs;
use tokio::task;

use crate::memory::ProcessMemoryStats;
use crate::utils::{format_size, get_cmd};
use crate::AnyError;

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
        if self.command.len() > 50 {
            self.command.truncate(50);
        }
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
                Some(task::spawn(async move { Process::new(pid) }))
            })
            .collect::<Vec<_>>();

        // Wait for all the processes to finish
        let processes = futures::future::try_join_all(processes).await?;
        // Sort the processes by swap usage
        self.processes = processes
            .into_iter()
            .filter_map(Result::ok)
            .collect();
        self.processes.sort_by_key(|p| p.memory.swap);
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
